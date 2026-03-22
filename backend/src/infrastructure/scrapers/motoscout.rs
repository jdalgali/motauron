use crate::application::ports::scraper::Scraper;
use crate::domain::entities::MotorcycleListing;
use async_trait::async_trait;
use headless_chrome::{Browser, LaunchOptions};
use std::ffi::OsStr;
use scraper::{Html, Selector};
use std::error::Error;
use std::time::Duration;

pub struct MotoscoutScraper {
    pub category: String,
    pub url: String,
}

impl MotoscoutScraper {
    pub fn new(category: &str, url: &str) -> Self {
        Self {
            category: category.to_string(),
            url: url.to_string(),
        }
    }

    fn parse_price(s: &str) -> Option<u32> {
        // Example: "CHF 10'490.–" -> 10490
        let cleaned: String = s.chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        cleaned.parse().ok()
    }

    fn parse_mileage(s: &str) -> u32 {
        // Example: "5'310 km" -> 5310
        let cleaned: String = s.chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        cleaned.parse().unwrap_or(0)
    }

    fn extract_id(url: &str) -> String {
        // Example: "/de/d/yamaha-xsr-125-legacy-20287260" -> "20287260"
        url.split('-').last().unwrap_or(url).to_string()
    }
}

#[async_trait]
impl Scraper for MotoscoutScraper {
    async fn scrape(&self) -> Result<Vec<MotorcycleListing>, Box<dyn Error>> {
        println!("Motoscout: Launching browser...");

        // Use CHROME_PATH env var if set (e.g. in Docker: /usr/bin/chromium).
        // Falls back to headless_chrome auto-detection when unset (local dev).
        let chrome_path = std::env::var("CHROME_PATH").ok().map(std::path::PathBuf::from);

        let browser = Browser::new(
            LaunchOptions::default_builder()
                .headless(true)
                .path(chrome_path)
                .args(vec![
                    OsStr::new("--no-sandbox"),
                    OsStr::new("--disable-dev-shm-usage"),
                ])
                .build()?,
        )?;

        let tab = browser.new_tab()?;
        
        println!("Motoscout: Navigating to {}...", self.url);
        tab.navigate_to(&self.url)?;
        
        // Wait for Cloudflare challenge + React render
        // This is a bit slow but safer
        tokio::time::sleep(Duration::from_secs(15)).await;

        // Try to click cookie consent if it blocks rendering
        if let Ok(accept_btn) = tab.wait_for_element("#onetrust-accept-btn-handler") {
            let _ = accept_btn.click();
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        let html = tab.get_content()?;
        let document = Html::parse_document(&html);
        
        let article_selector = Selector::parse("article").unwrap();
        let h2_selector = Selector::parse("h2").unwrap();
        let p_selector = Selector::parse("p").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        let today = chrono::Local::now().date_naive();
        let mut results = Vec::new();

        for article in document.select(&article_selector) {
            let title = article.select(&h2_selector)
                .next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default()
                .trim()
                .to_string();

            if title.is_empty() { continue; }

            let detail_link = article.select(&a_selector)
                .filter_map(|e| e.value().attr("href"))
                .find(|href| href.contains("/de/d/"))
                .unwrap_or("");

            if detail_link.is_empty() { continue; }

            let id = Self::extract_id(detail_link);
            let url = format!("https://www.motoscout24.ch{}", detail_link);

            let mut price = 0;
            let mut mileage = 0;
            let mut year = 0;
            let mut location = String::new();
            let mut seller_name = String::new();

            for p in article.select(&p_selector) {
                let text = p.text().collect::<String>();
                if text.contains("CHF") {
                    price = Self::parse_price(&text).unwrap_or(0);
                } else if text.contains("km") {
                    mileage = Self::parse_mileage(&text);
                } else if text.len() == 4 && text.chars().all(|c| c.is_ascii_digit()) {
                    year = text.parse().unwrap_or(0);
                } else if text.contains("PS") || text.contains("kW") {
                    // Skip power info
                } else if text.contains('/') {
                    // Skip "1 / 7" image counter
                } else if !text.trim().is_empty() {
                    // Last few P tags are usually seller name and location
                    // This is opportunistic
                    if location.is_empty() {
                        seller_name = text.trim().to_string();
                        location = "CH".to_string(); // Fallback
                    } else {
                         // Swap if we find something that looks more like a location
                    }
                }
            }
            
            // Try to refine location
            // Usually the last P tag has "8810 Horgen" style
            let paragraphs: Vec<_> = article.select(&p_selector).map(|p| p.text().collect::<String>()).collect();
            if paragraphs.len() >= 2 {
                location = paragraphs.last().unwrap().trim().to_string();
                seller_name = paragraphs[paragraphs.len()-2].trim().to_string();
            }

            if price == 0 { continue; }

            results.push(MotorcycleListing::new(
                id.parse().unwrap_or(0),
                today,
                self.category.clone(),
                title,
                price,
                mileage,
                year,
                url,
                location,
                String::new(), // kanton unknown
                false, // privacy unknown
                seller_name,
            ));
        }

        println!("Motoscout: Successfully extracted {} listings from {}", results.len(), self.category);

        Ok(results)
    }
}
