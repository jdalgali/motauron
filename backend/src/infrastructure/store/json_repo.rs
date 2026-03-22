use crate::application::ports::repository::ListingRepository;
use crate::domain::entities::MotorcycleListing;
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;

pub struct JsonListingRepository {
    pub path: String,
}

impl JsonListingRepository {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

#[async_trait]
impl ListingRepository for JsonListingRepository {
    async fn load(&self) -> Result<HashMap<u64, MotorcycleListing>, Box<dyn Error>> {
        if !std::path::Path::new(&self.path).exists() {
            return Ok(HashMap::new());
        }
        let data = tokio::fs::read_to_string(&self.path).await?;
        let list: Vec<MotorcycleListing> = serde_json::from_str(&data)?;
        Ok(list.into_iter().map(|l| (l.listing_id, l)).collect())
    }

    async fn save(&self, listings: &HashMap<u64, MotorcycleListing>) -> Result<(), Box<dyn Error>> {
        let mut rows: Vec<&MotorcycleListing> = listings.values().collect();
        rows.sort_by_key(|l| l.listing_id);
        let data = serde_json::to_string_pretty(&rows)?;
        tokio::fs::write(&self.path, data).await?;
        Ok(())
    }
}
