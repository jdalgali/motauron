use crate::application::ports::repository::ListingRepository;
use crate::domain::entities::MotorcycleListing;
use async_trait::async_trait;
use firestore::{FirestoreDb, FirestoreDbOptions};
use std::collections::HashMap;
use std::error::Error;

pub struct FirestoreListingRepository {
    pub db: FirestoreDb,
    pub collection: String,
}

impl FirestoreListingRepository {
    pub async fn new(project_id: &str, service_account_path: &str) -> Result<Self, Box<dyn Error>> {
        unsafe {
            std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", service_account_path);
        }
        let db = FirestoreDb::with_options(
            FirestoreDbOptions::new(project_id.to_string())
        ).await?;
        Ok(Self {
            db,
            collection: "listings".to_string(),
        })
    }
}

#[async_trait]
impl ListingRepository for FirestoreListingRepository {
    async fn load(&self) -> Result<HashMap<u64, MotorcycleListing>, Box<dyn Error>> {
        let docs: Vec<MotorcycleListing> = self.db.fluent()
            .select()
            .from(self.collection.as_str())
            .obj()
            .query()
            .await?;

        let mut map = HashMap::new();
        for doc in docs {
            map.insert(doc.listing_id, doc.clone());
        }
        Ok(map)
    }

    async fn save(&self, listings: &HashMap<u64, MotorcycleListing>) -> Result<(), Box<dyn Error>> {
        for listing in listings.values() {
            self.db.fluent()
                .update()
                .in_col(&self.collection)
                .document_id(listing.listing_id.to_string())
                .object(listing)
                .execute::<MotorcycleListing>()
                .await?;
        }
        Ok(())
    }
}
