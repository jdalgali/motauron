use chrono::{Datelike, NaiveDate};
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
    pub listing_id: u64,
    pub fingerprint: String,
    pub status: ListingStatus,
    pub first_seen: NaiveDate,
    pub last_seen: NaiveDate,
    pub category: String,
    pub title: String,
    pub price_chf: u32,
    #[serde(default)]
    pub original_price_chf: Option<u32>, // price when first seen — never updated
    pub year: u16,
    pub mileage_km: u32,
    pub url: String,
    pub previous_listing_id: Option<u64>,
    pub location: String,
    pub kanton: String,
    pub is_private: bool,
    pub seller_name: String,
    pub price_score: i32,
    pub price_label: String,
    pub score_peers: u8, // number of listings the score is based on
}

impl MotorcycleListing {
    /// Mileage rounding is tiered:
    ///   <1000km  → nearest 100km
    ///   ≥1000km  → nearest 1000km
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
        location: String,
        kanton: String,
        is_private: bool,
        seller_name: String,
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
            original_price_chf: Some(price_chf),
            year,
            mileage_km,
            url,
            previous_listing_id: None,
            location,
            kanton,
            is_private,
            seller_name,
            price_score: 0,
            price_label: String::new(),
            score_peers: 0,
        }
    }

    /// Years the bike has been on the road (min 1 to avoid division by zero)
    pub fn age_years(&self) -> u16 {
        let current_year = chrono::Local::now().date_naive().year() as u16;
        (current_year.saturating_sub(self.year)).max(1)
    }

    pub fn annual_km(&self) -> u32 {
        self.mileage_km / self.age_years() as u32
    }
}
