use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ListingStatus {
    Active,
    Sold,
    Relisted,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MotorcycleListing {
    pub listing_id: u64,              // Site's own ad ID — stable primary key
    pub fingerprint: String,          // Content hash for relist detection
    pub status: ListingStatus,
    pub first_seen: NaiveDate,
    pub last_seen: NaiveDate,
    pub category: String,
    pub title: String,
    pub price_chf: u32,
    pub year: u16,
    pub mileage_km: u32,
    pub url: String,
    pub previous_listing_id: Option<u64>, // Set when this is a detected relist
}

impl MotorcycleListing {
    /// Stable fingerprint identifying the physical motorcycle regardless of listing ID.
    ///
    /// Mileage rounding is tiered to avoid collisions on nearly-new bikes:
    ///   <1000km  → nearest 100km  (10km and 200km are different buckets)
    ///   ≥1000km  → nearest 1000km (absorbs minor real-world discrepancies on relists)
    pub fn build_fingerprint(title: &str, year: u16, mileage_km: u32) -> String {
        let normalized = title.trim().to_lowercase();
        let rounded_km = if mileage_km < 1_000 {
            (mileage_km / 100) * 100
        } else {
            (mileage_km / 1_000) * 1_000
        };
        format!("{}|{}|{}", normalized, year, rounded_km)
    }

    pub fn new(
        listing_id: u64,
        today: NaiveDate,
        category: String,
        title: String,
        price_chf: u32,
        mileage_km: u32,
        year: u16,
        url: String,
    ) -> Self {
        let fingerprint = Self::build_fingerprint(&title, year, mileage_km);
        Self {
            listing_id,
            fingerprint,
            status: ListingStatus::Active,
            first_seen: today,
            last_seen: today,
            category,
            title,
            price_chf,
            mileage_km,
            year,
            url,
            previous_listing_id: None,
        }
    }
}
