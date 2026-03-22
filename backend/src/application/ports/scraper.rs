use crate::domain::entities::MotorcycleListing;
use std::error::Error;
use async_trait::async_trait;

#[async_trait]
pub trait Scraper {
    async fn scrape(&self) -> Result<Vec<MotorcycleListing>, Box<dyn Error>>;
}
