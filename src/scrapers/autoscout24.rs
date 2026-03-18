// autoscout24.ch scraper
//
// Uses the official REST API at https://api.autoscout24.ch (OpenAPI spec:
// https://github.com/smg-automotive/autoscout24-api-specs)
//
// Requires OAuth2 client credentials from SMG Automotive.
// Set these environment variables before running:
//
//   AS24_CLIENT_ID=your_client_id
//   AS24_CLIENT_SECRET=your_client_secret
//
// Contact info@autoscout24.ch or https://b2b.autoscout24.ch to request access.

use crate::models::motorcycle::MotorcycleListing;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

const TOKEN_URL: &str = "https://api.autoscout24.ch/public/v1/clients/oauth/token";
const SEARCH_URL: &str = "https://api.autoscout24.ch/public/v1/listings/search";
const AUDIENCE: &str = "https://api.autoscout24.ch";
const PAGE_SIZE: u32 = 20;

// --- OAuth ---

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

async fn fetch_token(
    client: &Client,
    client_id: &str,
    client_secret: &str,
) -> Result<String, Box<dyn Error>> {
    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("audience", AUDIENCE),
    ];

    let resp: TokenResponse = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp.access_token)
}

// --- Search request structs ---

#[derive(Serialize)]
struct SearchRequest<'a> {
    query: SearchQuery<'a>,
    pagination: Pagination,
}

#[derive(Serialize)]
struct SearchQuery<'a> {
    #[serde(rename = "vehicleCategories")]
    vehicle_categories: Vec<&'a str>,
    #[serde(rename = "makeModelVersions")]
    make_model_versions: Vec<MakeModelVersion<'a>>,
}

#[derive(Serialize)]
struct MakeModelVersion<'a> {
    #[serde(rename = "makeKey")]
    make_key: &'a str,
    #[serde(rename = "modelKey")]
    model_key: &'a str,
}

#[derive(Serialize)]
struct Pagination {
    page: u32,
    size: u32,
}

// --- Search response structs ---

#[derive(Deserialize)]
struct SearchResponse {
    content: Vec<As24Listing>,
    #[serde(rename = "totalPages", default)]
    total_pages: u32,
}

#[derive(Deserialize)]
struct As24Listing {
    id: u64,
    price: Option<f64>,
    mileage: Option<u32>,
    #[serde(rename = "firstRegistrationYear")]
    first_registration_year: Option<u16>,
    make: Option<As24Make>,
    model: Option<As24Model>,
    #[serde(rename = "versionFullName")]
    version_full_name: Option<String>,
    seller: Option<As24Seller>,
}

#[derive(Deserialize)]
struct As24Make {
    key: String,
    name: String,
}

#[derive(Deserialize)]
struct As24Model {
    name: String,
}

#[derive(Deserialize)]
struct As24Seller {
    name: Option<String>,
    city: Option<String>,
    #[serde(rename = "zipCode")]
    zip_code: Option<String>,
}

// --- Public scraper function ---

/// `make_key` and `model_key` are the API keys as returned by the reference-data endpoint.
/// Examples: make_key = "yamaha", model_key = "tenere-700"
pub async fn scrape_category(
    client: &Client,
    category: &str,
    make_key: &str,
    model_key: &str,
) -> Result<Vec<MotorcycleListing>, Box<dyn Error>> {
    let client_id = std::env::var("AS24_CLIENT_ID")
        .map_err(|_| "AS24_CLIENT_ID not set — see src/scrapers/autoscout24.rs for setup")?;
    let client_secret = std::env::var("AS24_CLIENT_SECRET")
        .map_err(|_| "AS24_CLIENT_SECRET not set — see src/scrapers/autoscout24.rs for setup")?;

    let token = fetch_token(client, &client_id, &client_secret).await?;

    let today = chrono::Local::now().date_naive();
    let mut results = Vec::new();
    let mut page = 0u32;

    loop {
        let body = SearchRequest {
            query: SearchQuery {
                vehicle_categories: vec!["motorcycle"],
                make_model_versions: vec![MakeModelVersion { make_key, model_key }],
            },
            pagination: Pagination { page, size: PAGE_SIZE },
        };

        let resp: SearchResponse = client
            .post(SEARCH_URL)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        for item in resp.content {
            let Some(price) = item.price else { continue };

            // Prefer versionFullName (e.g. "Yamaha Ténéré 700 World Raid"),
            // fall back to make + model name
            let title = item
                .version_full_name
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| {
                    let marke = item.make.as_ref().map(|m| m.name.as_str()).unwrap_or("");
                    let modell = item.model.as_ref().map(|m| m.name.as_str()).unwrap_or("");
                    format!("{} {}", marke, modell).trim().to_string()
                });

            if title.is_empty() {
                continue;
            }

            let mileage = item.mileage.unwrap_or(0);
            let year = item.first_registration_year.unwrap_or(0);

            let seller_name = item
                .seller
                .as_ref()
                .and_then(|s| s.name.clone())
                .unwrap_or_default();
            let city = item
                .seller
                .as_ref()
                .and_then(|s| s.city.clone())
                .unwrap_or_default();
            let zip = item
                .seller
                .as_ref()
                .and_then(|s| s.zip_code.clone())
                .unwrap_or_default();

            // Private sellers have no company name on file
            let is_private = seller_name.trim().is_empty();
            let kanton = zip_to_kanton(&zip).to_string();

            let make_slug = item.make.as_ref().map(|m| m.key.as_str()).unwrap_or(make_key);
            let listing_url = format!(
                "https://www.autoscout24.ch/de/motorrad/{}/{}/{}",
                make_slug, model_key, item.id
            );

            results.push(MotorcycleListing::new(
                item.id,
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
            ));
        }

        if resp.total_pages == 0 || page + 1 >= resp.total_pages {
            break;
        }
        page += 1;
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    }

    Ok(results)
}

/// Maps a Swiss postal code to a canton abbreviation.
/// Covers the main ranges — good enough for the location scoring multiplier.
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
