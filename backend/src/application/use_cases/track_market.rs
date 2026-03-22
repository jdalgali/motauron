use crate::application::ports::scraper::Scraper;
use crate::application::ports::repository::ListingRepository;
use crate::application::ports::notifier::Notifier;
use crate::domain::services::merger::merge_listings;
use std::error::Error;

pub struct TrackMarketUseCase {
    pub scrapers: Vec<Box<dyn Scraper + Send + Sync>>,
    pub repository: Box<dyn ListingRepository + Send + Sync>,
    pub notifier: Box<dyn Notifier + Send + Sync>,
}

impl TrackMarketUseCase {
    pub async fn execute(&self) -> Result<(), Box<dyn Error>> {
        let mut all_scraped = Vec::new();

        // Run all scrapers and combine their results
        for scraper in &self.scrapers {
            match scraper.scrape().await {
                Ok(items) => all_scraped.extend(items),
                Err(e) => eprintln!("Scraper error: {}", e),
            }
        }

        let existing_db = self.repository.load().await.unwrap_or_default();
        
        let (updated_db, summary) = merge_listings(existing_db, all_scraped);
        
        summary.print(); // Assume MarketSummary has a `print` method just like before

        self.repository.save(&updated_db).await?;
        
        if let Err(e) = self.notifier.send_summary(&summary).await {
            eprintln!("notify error: {}", e);
        }
        
        Ok(())
    }
}
