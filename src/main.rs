mod models;
mod scrapers;
mod scoring;
mod store;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("openclaw — motorcycle market tracker");
    println!();

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    let mut all_listings = Vec::new();

    // --- motorradhandel.ch ---
    // Category key (stable DB id) → search URL
    let mh_categories: &[(&str, &str)] = &[
        (
            "Tenere_700",
            "https://motorradhandel.ch/en/all-motorbikes-and-scooters-search-switzerland?q=%7B%22arts%22%3A%5B1%5D,%22categories%22%3A%5B4%5D,%22brands%22%3A%5B197%5D,%22models%22%3A%5B595%5D%7D",
        ),
        // ("Africa_Twin", "https://..."),
    ];

    for (category, url) in mh_categories {
        let display = category.replace('_', " ").to_lowercase();
        match scrapers::motorradhandel::scrape_category(&client, category, url).await {
            Ok(listings) => {
                println!("  motorradhandel · {} — {} found", display, listings.len());
                all_listings.extend(listings);
            }
            Err(e) => eprintln!("  motorradhandel · {} — failed: {}", display, e),
        }
    }

    // --- autoscout24.ch ---
    // Requires AS24_CLIENT_ID and AS24_CLIENT_SECRET environment variables.
    // Category key → (make_key, model_key) as used by the autoscout24 API.
    // Get make/model keys from: GET /public/v1/reference-data (once credentialed)
    let as24_categories: &[(&str, &str, &str)] = &[
        ("Tenere_700", "yamaha", "tenere-700"),
        // ("Africa_Twin", "honda", "africa-twin"),
    ];

    for (category, make_key, model_key) in as24_categories {
        let display = category.replace('_', " ").to_lowercase();
        match scrapers::autoscout24::scrape_category(category, make_key, model_key).await {
            Ok(listings) => {
                println!("  autoscout24    · {} — {} found", display, listings.len());
                all_listings.extend(listings);
            }
            Err(e) => eprintln!("  autoscout24    · {} — skipped: {}", display, e),
        }
    }

    let summary = store::merge_and_save(all_listings)?;
    summary.print();

    Ok(())
}
