pub mod motorradhandel;

use crate::application::ports::scraper::Scraper;
use crate::config::GenerationRule;
use crate::domain::entities::MotorcycleListing;
use async_trait::async_trait;
use std::error::Error;

/// Wraps any scraper and stamps a `generation` onto each listing using config-defined rules.
pub struct WithGeneration {
    pub inner: Box<dyn Scraper + Send + Sync>,
    pub rules: Vec<GenerationRule>,
}

fn detect_generation(rules: &[GenerationRule], year: u16, title: &str) -> Option<String> {
    let title_lower = title.to_lowercase();
    // Title keyword rules take priority (variant names like "World Raid" override year ranges)
    for rule in rules {
        if let Some(kw) = &rule.title_contains {
            if title_lower.contains(&kw.to_lowercase()) {
                return Some(rule.name.clone());
            }
        }
    }
    // Then year-range rules
    for rule in rules {
        if rule.title_contains.is_some() {
            continue;
        }
        let from_ok = rule.year_from.map_or(true, |y| year >= y);
        let to_ok = rule.year_to.map_or(true, |y| year <= y);
        if from_ok && to_ok {
            return Some(rule.name.clone());
        }
    }
    None
}

#[async_trait]
impl Scraper for WithGeneration {
    async fn scrape(&self) -> Result<Vec<MotorcycleListing>, Box<dyn Error>> {
        let mut listings = self.inner.scrape().await?;
        for listing in &mut listings {
            listing.generation = detect_generation(&self.rules, listing.year, &listing.title);
        }
        Ok(listings)
    }
}
