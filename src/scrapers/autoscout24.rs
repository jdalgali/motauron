// autoscout24.ch / motoscout24.ch browser scraper
//
// Uses chromiumoxide (Chrome DevTools Protocol) to render pages in a real
// browser, bypassing Cloudflare's JS challenge.
//
// Requires Chromium or Chrome:
//   sudo apt install chromium-browser        (Ubuntu / WSL)
//   brew install --cask chromium             (macOS)
//
// Optional debug mode — dumps raw __NEXT_DATA__ JSON to /tmp for inspection:
//   AS24_DEBUG=1 cargo run

use crate::models::motorcycle::MotorcycleListing;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams;
use chrono::NaiveDate;
use futures::StreamExt;
use serde_json::Value;
use std::error::Error;

const LISTINGS_PER_PAGE: usize = 20;

// Injected before any page JS runs — removes the most obvious headless tells.
const STEALTH_JS: &str = r#"
    Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
    Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
    Object.defineProperty(navigator, 'languages', { get: () => ['de-CH', 'de', 'en'] });
    window.chrome = { runtime: {} };
"#;

// --- Public entry point ---

pub async fn scrape_category(
    category: &str,
    make_key: &str,
    model_key: &str,
) -> Result<Vec<MotorcycleListing>, Box<dyn Error>> {
    let config = BrowserConfig::builder()
        .arg("--no-sandbox")
        .arg("--disable-setuid-sandbox")
        .arg("--disable-dev-shm-usage")
        .arg("--disable-gpu")
        .arg("--disable-blink-features=AutomationControlled")
        .arg("--window-size=1920,1080")
        .arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .build()?;

    let (mut browser, mut handler) = Browser::launch(config).await?;

    // The handler drives the CDP connection — must be polled continuously.
    tokio::task::spawn(async move {
        loop {
            if handler.next().await.is_none() {
                break;
            }
        }
    });

    let today = chrono::Local::now().date_naive();
    let mut all_results = Vec::new();
    let mut pg = 1u32;

    loop {
        let url = format!(
            "https://www.autoscout24.ch/lst/{}/{}?atype=M&cy=CH&page={}",
            make_key, model_key, pg
        );

        // Navigate to blank first so we can inject stealth JS before page load
        let page = browser.new_page("about:blank").await?;
        page.execute(
            AddScriptToEvaluateOnNewDocumentParams::builder()
                .source(STEALTH_JS)
                .build()
                .unwrap(),
        )
        .await?;
        page.goto(url).await?;
        page.wait_for_navigation().await?;

        // Give Cloudflare's Turnstile challenge time to auto-solve
        tokio::time::sleep(std::time::Duration::from_secs(6)).await;

        // Check if we're still on a Cloudflare challenge page
        let title: String = page
            .get_title()
            .await
            .ok()
            .flatten()
            .unwrap_or_default();
        if title.contains("Just a moment") {
            eprintln!("  [as24] cloudflare challenge not solved (page: {})", pg);
            let _ = page.close().await;
            break;
        }

        // Extract Next.js server-side data embedded in the page
        let raw: Value = page
            .evaluate(
                "(() => { \
                    const el = document.getElementById('__NEXT_DATA__'); \
                    return el ? JSON.parse(el.textContent) : null; \
                })()",
            )
            .await
            .ok()
            .and_then(|v| v.into_value().ok())
            .unwrap_or(Value::Null);

        if std::env::var("AS24_DEBUG").is_ok() {
            let path = format!("/tmp/as24_p{}.json", pg);
            let _ = std::fs::write(&path, serde_json::to_string_pretty(&raw).unwrap_or_default());
            eprintln!("  [as24 debug] page {} → {}", pg, path);
        }

        let listings = find_listings(&raw);
        if listings.is_empty() {
            let _ = page.close().await;
            break;
        }

        let count = listings.len();
        for item in &listings {
            if let Some(l) = parse_listing(item, category, make_key, model_key, today) {
                all_results.push(l);
            }
        }

        let _ = page.close().await;

        if count < LISTINGS_PER_PAGE {
            break;
        }
        pg += 1;
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    }

    let _ = browser.close().await;
    Ok(all_results)
}

// --- JSON extraction ---

/// Walk known Next.js data paths to find the listings array.
/// Run with AS24_DEBUG=1 to inspect the raw JSON if none of these match.
fn find_listings(data: &Value) -> Vec<Value> {
    let paths: &[&[&str]] = &[
        &["props", "pageProps", "listings"],
        &["props", "pageProps", "searchResult", "listings"],
        &["props", "pageProps", "vehicleListingSearchResult", "listings"],
        &["props", "pageProps", "initialState", "listings", "items"],
        &["props", "pageProps", "data", "listings"],
        &["props", "pageProps", "data", "results"],
    ];

    for path in paths {
        if let Some(arr) = dig(data, path).and_then(|v| v.as_array()) {
            if !arr.is_empty() {
                return arr.clone();
            }
        }
    }
    vec![]
}

fn dig<'a>(v: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cur = v;
    for key in path {
        cur = cur.get(key)?;
    }
    Some(cur)
}

// --- Listing parsing ---

fn parse_listing(
    item: &Value,
    category: &str,
    make_key: &str,
    model_key: &str,
    today: NaiveDate,
) -> Option<MotorcycleListing> {
    let id = item.get("id").and_then(|v| v.as_u64())?;

    // Price: plain number or { amount, value }
    let price = item
        .get("price")
        .and_then(|p| {
            p.as_f64()
                .or_else(|| p.get("amount").and_then(|a| a.as_f64()))
                .or_else(|| p.get("value").and_then(|a| a.as_f64()))
        })?;

    let mileage = item
        .get("mileage")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let year = item
        .get("firstRegistrationYear")
        .or_else(|| item.get("registrationYear"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u16;

    // Title: prefer versionFullName, fall back to make + model
    let version = item
        .get("versionFullName")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());

    let make_name = dig(item, &["make", "name"])
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let model_name = dig(item, &["model", "name"])
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let title = version
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{} {}", make_name, model_name).trim().to_string());

    if title.is_empty() {
        return None;
    }

    let seller = item.get("seller");

    let seller_name = seller
        .and_then(|s| s.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let city = seller
        .and_then(|s| s.get("city"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let zip = seller
        .and_then(|s| s.get("zipCode").or_else(|| s.get("zip")))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let is_private = seller_name.trim().is_empty();
    let kanton = zip_to_kanton(&zip).to_string();

    let make_slug = dig(item, &["make", "key"])
        .and_then(|v| v.as_str())
        .unwrap_or(make_key);

    let listing_url = format!(
        "https://www.autoscout24.ch/de/motorrad/{}/{}/{}",
        make_slug, model_key, id
    );

    Some(MotorcycleListing::new(
        id,
        today,
        category.to_string(),
        title,
        price as u32,
        mileage,
        year,
        listing_url,
        city,
        kanton,
        is_private,
        seller_name,
    ))
}

/// Maps a Swiss postal code to a canton abbreviation.
fn zip_to_kanton(zip: &str) -> &'static str {
    let Ok(n) = zip.trim().parse::<u32>() else {
        return "";
    };
    match n {
        1200..=1299 => "GE",
        1900..=1999 => "VS",
        1600..=1699 => "FR",
        1000..=1999 => "VD",
        2000..=2399 => "NE",
        2400..=2999 => "JU",
        3000..=3999 => "BE",
        4000..=4059 => "BS",
        4100..=4499 => "BL",
        4500..=4999 => "SO",
        5000..=5999 => "AG",
        6000..=6199 => "LU",
        6200..=6299 => "NW",
        6300..=6499 => "ZG",
        6500..=6999 => "TI",
        7000..=7999 => "GR",
        8200..=8299 => "SH",
        8500..=8599 => "TG",
        8700..=8799 => "SZ",
        8000..=8999 => "ZH",
        9000..=9299 => "SG",
        9300..=9399 => "AR",
        9400..=9699 => "SG",
        _ => "",
    }
}
