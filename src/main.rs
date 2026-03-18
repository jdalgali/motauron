mod config;
mod models;
mod notify;
mod scrapers;
mod scoring;
mod store;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = config::load()?;
    let daemon = std::env::args().any(|a| a == "--daemon");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .build()?;

    if daemon {
        let interval = std::time::Duration::from_secs(cfg.agent.interval_hours * 3600);
        println!("openclaw agent — running every {}h", cfg.agent.interval_hours);
        loop {
            run_cycle(&client, &cfg).await;
            println!("sleeping {}h until next cycle …", cfg.agent.interval_hours);
            tokio::time::sleep(interval).await;
        }
    } else {
        run_cycle(&client, &cfg).await;
    }

    Ok(())
}

async fn run_cycle(client: &reqwest::Client, cfg: &config::Config) {
    println!("openclaw — motorcycle market tracker");
    println!();

    let mut all_listings = Vec::new();

    // --- motorradhandel.ch ---
    // Category key (stable DB id) → search URL
    let mh_categories: &[(&str, &str)] = &[
        (
            "Tenere_700",
            "https://motorradhandel.ch/en/all-motorbikes-and-scooters-search-switzerland?q=%7B%22arts%22%3A%5B1%5D,%22categories%22%3A%5B4%5D,%22brands%22%3A%5B197%5D,%22models%22%3A%5B595%5D%7D",
        ),
        (
            "MT09",
            "https://motorradhandel.ch/en/all-motorbikes-and-scooters-search-switzerland?q=%7B%22arts%22%3A%5B1%5D%2C%22brands%22%3A%5B197%5D%2C%22models%22%3A%5B198%5D%7D",
        ),
        // ("Africa_Twin", "https://motorradhandel.ch/en/all-motorbikes-and-scooters-search-switzerland?q=%7B%22arts%22%3A%5B1%5D%2C%22brands%22%3A%5B68%5D%2C%22models%22%3A%5B264%5D%7D"),
    ];

    for (category, url) in mh_categories {
        let display = category.replace('_', " ").to_lowercase();
        match scrapers::motorradhandel::scrape_category(client, category, url).await {
            Ok(listings) => {
                println!("  motorradhandel · {} — {} found", display, listings.len());
                all_listings.extend(listings);
            }
            Err(e) => eprintln!("  motorradhandel · {} — failed: {}", display, e),
        }
    }

    // --- autoscout24.ch ---
    // Browser scraper (chromiumoxide) — currently blocked by Cloudflare Turnstile.
    // Uncomment once SMG API credentials are obtained or bypass is solved.
    //
    // let as24_categories: &[(&str, &str, &str)] = &[
    //     ("Tenere_700", "yamaha", "tenere-700"),
    //     ("MT09", "yamaha", "mt-09"),
    // ];
    // for (category, make_key, model_key) in as24_categories { ... }

    let summary = match store::merge_and_save(all_listings) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  store error: {}", e);
            return;
        }
    };
    summary.print();

    let notify_cfg = config::notify_config(cfg);
    let mut notify_state = notify::state::load();
    if let Err(e) = notify::dispatch(client, &notify_cfg, &summary, &mut notify_state).await {
        eprintln!("  notify error: {}", e);
    }
    if let Err(e) = notify::state::save(&notify_state) {
        eprintln!("  notify state save error: {}", e);
    }
}
