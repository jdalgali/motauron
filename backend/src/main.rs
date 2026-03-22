mod config;
mod domain;
mod application;
mod infrastructure;

use application::use_cases::track_market::TrackMarketUseCase;
use infrastructure::api::{ApiState, serve};
use infrastructure::notify::ntfy_notifier::{NtfyConfig, NtfyNotifier};
use infrastructure::scrapers::motorradhandel::MotorradhandelScraper;
use infrastructure::store::json_repo::JsonListingRepository;
use infrastructure::store::firestore_repo::FirestoreListingRepository;

use std::sync::Arc;
use std::error::Error;

const LOCAL_DB_PATH: &str = "listings.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let cfg = config::load()?;
    let args: Vec<String> = std::env::args().collect();
    let daemon = args.iter().any(|a| a == "--daemon");
    let serve_mode = args.iter().any(|a| a == "--serve");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .build()?;

    let ntfy_cfg = cfg.notify.ntfy.map(|n| NtfyConfig {
        url: n.url,
        token: n.token,
    });

    let notifier = Box::new(NtfyNotifier::new(
        client.clone(),
        ntfy_cfg,
        cfg.agent.min_alert_score,
    ));

    // Build scrapers from config targets.
    // Default: one target — all motorcycles on motorradhandel.ch.
    let targets = if cfg.targets.is_empty() {
        vec![config::Target::default()]
    } else {
        cfg.targets
    };

    let mut scrapers: Vec<Box<dyn application::ports::scraper::Scraper + Send + Sync>> = Vec::new();
    for t in &targets {
        let url = t.effective_url();
        scrapers.push(Box::new(infrastructure::scrapers::WithGeneration {
            inner: Box::new(MotorradhandelScraper::new(client.clone(), &url)),
            rules: t.generations.clone(),
        }));
    }

    // Auto-detect store:
    //   Firestore — if service-account.json is present or GOOGLE_APPLICATION_CREDENTIALS is set.
    //   JSON file  — otherwise (local dev, no cloud dependency).
    let use_firestore = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").is_ok()
        || std::path::Path::new("service-account.json").exists();

    let repository: Box<dyn application::ports::repository::ListingRepository + Send + Sync> =
        if use_firestore {
            let sa_path = if std::env::var("GOOGLE_APPLICATION_CREDENTIALS").is_ok() {
                None
            } else {
                Some("service-account.json")
            };
            println!("store: Firestore (project motauron-ch)");
            Box::new(FirestoreListingRepository::new("motauron-ch", sa_path).await?)
        } else {
            println!("store: local JSON ({})", LOCAL_DB_PATH);
            Box::new(JsonListingRepository::new(LOCAL_DB_PATH))
        };

    let use_case = Arc::new(TrackMarketUseCase {
        scrapers,
        repository,
        notifier,
    });

    if serve_mode {
        println!("motauron — local serve mode");
        println!("Running initial scrape…");
        if let Err(e) = use_case.execute().await {
            eprintln!("initial scrape error: {}", e);
        }

        let state = ApiState {
            use_case,
            json_path: LOCAL_DB_PATH.to_string(),
        };

        if daemon {
            // Background scrape loop + HTTP server in parallel.
            let uc = state.use_case.clone();
            let interval = std::time::Duration::from_secs(cfg.agent.interval_hours * 3600);
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(interval).await;
                    println!("motauron: background scrape…");
                    if let Err(e) = uc.execute().await {
                        eprintln!("background scrape error: {}", e);
                    }
                }
            });
        }

        serve(state, 3001).await?;
    } else if daemon {
        let interval = std::time::Duration::from_secs(cfg.agent.interval_hours * 3600);
        println!("motauron — daemon mode, every {}h", cfg.agent.interval_hours);
        loop {
            println!("motauron — motorcycle market tracker\n");
            if let Err(e) = use_case.execute().await {
                eprintln!("error: {}", e);
            }
            println!("sleeping {}h…", cfg.agent.interval_hours);
            tokio::time::sleep(interval).await;
        }
    } else {
        println!("motauron — motorcycle market tracker\n");
        if let Err(e) = use_case.execute().await {
            eprintln!("error: {}", e);
        }
    }

    Ok(())
}
