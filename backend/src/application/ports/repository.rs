use crate::domain::entities::MotorcycleListing;
use std::collections::HashMap;
use std::error::Error;
use async_trait::async_trait;

#[async_trait]
pub trait ListingRepository {
    async fn load(&self) -> Result<HashMap<u64, MotorcycleListing>, Box<dyn Error>>;
    async fn save(&self, listings: &HashMap<u64, MotorcycleListing>) -> Result<(), Box<dyn Error>>;
}
