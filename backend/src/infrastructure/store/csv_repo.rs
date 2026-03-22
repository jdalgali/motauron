use crate::application::ports::repository::ListingRepository;
use crate::domain::entities::MotorcycleListing;
use std::collections::HashMap;
use std::error::Error;

pub struct CsvListingRepository {
    pub db_path: String,
}

impl CsvListingRepository {
    pub fn new(db_path: &str) -> Self {
        Self {
            db_path: db_path.to_string(),
        }
    }
}

use async_trait::async_trait;

#[async_trait]
impl ListingRepository for CsvListingRepository {
    async fn load(&self) -> Result<HashMap<u64, MotorcycleListing>, Box<dyn Error>> {
        if !std::path::Path::new(&self.db_path).exists() {
            return Ok(HashMap::new());
        }
        let mut reader = csv::Reader::from_path(&self.db_path)?;
        let map = reader
            .deserialize()
            .filter_map(|r: Result<MotorcycleListing, _>| r.ok())
            .map(|l| (l.listing_id, l))
            .collect();
        Ok(map)
    }

    async fn save(&self, db: &HashMap<u64, MotorcycleListing>) -> Result<(), Box<dyn Error>> {
        let mut rows: Vec<&MotorcycleListing> = db.values().collect();
        rows.sort_by_key(|l| l.listing_id);

        let file = std::fs::File::create(&self.db_path)?;
        let mut buf = std::io::BufWriter::new(file);
        std::io::Write::write_all(&mut buf, b"\xef\xbb\xbf")?;
        let mut writer = csv::Writer::from_writer(buf);
        for listing in rows {
            writer.serialize(listing)?;
        }
        writer.flush()?;
        Ok(())
    }
}
