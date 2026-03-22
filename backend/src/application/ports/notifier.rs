use crate::domain::services::merger::MarketSummary;
use std::error::Error;
use async_trait::async_trait;

#[async_trait]
pub trait Notifier {
    async fn send_summary(&self, summary: &MarketSummary) -> Result<(), Box<dyn Error>>;
}
