mod models;
mod scrapers;
mod store;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🦅 OpenClaw starting...");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    // Add more (name, url) pairs here to track additional models
    let categories: &[(&str, &str)] = &[
        (
            "Tenere_700",
            "https://motorradhandel.ch/en/all-motorbikes-and-scooters-search-switzerland?q=%7B%22arts%22%3A%5B1%5D,%22categories%22%3A%5B4%5D,%22brands%22%3A%5B197%5D,%22models%22%3A%5B595%5D%7D",
        ),
        // ("Africa_Twin", "https://..."),
    ];

    let mut all_listings = Vec::new();

    for (category, url) in categories {
        match scrapers::motorradhandel::scrape_category(&client, category, url).await {
            Ok(listings) => {
                println!("  Found {} listings for {}", listings.len(), category);
                all_listings.extend(listings);
            }
            Err(e) => eprintln!("  Error scraping {}: {}", category, e),
        }
    }

    let summary = store::merge_and_save(all_listings)?;
    summary.print();

    Ok(())
}
