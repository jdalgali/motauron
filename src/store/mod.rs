use crate::models::motorcycle::{ListingStatus, MotorcycleListing};
use std::collections::{HashMap, HashSet};
use std::error::Error;

const DB_PATH: &str = "listings_db.csv";

pub struct StoreSummary {
    pub new: Vec<MotorcycleListing>,
    pub updated: usize,
    pub sold: Vec<MotorcycleListing>,
    pub relisted: Vec<MotorcycleListing>,
}

impl StoreSummary {
    pub fn print(&self) {
        println!("\n--- Run Summary ---");
        println!("  Updated (still active): {}", self.updated);

        if self.new.is_empty() {
            println!("  New listings:           0");
        } else {
            println!("  New listings:           {}", self.new.len());
            for l in &self.new {
                println!(
                    "    [NEW]  {} | {}km | {} | CHF {} | {}",
                    l.title, l.mileage_km, l.year, l.price_chf, l.url
                );
            }
        }

        if self.sold.is_empty() {
            println!("  Marked as sold:         0");
        } else {
            println!("  Marked as sold:         {}", self.sold.len());
            for l in &self.sold {
                println!(
                    "    [SOLD] {} | {}km | {} | CHF {} | last seen {}",
                    l.title, l.mileage_km, l.year, l.price_chf, l.last_seen
                );
            }
        }

        if self.relisted.is_empty() {
            println!("  Relisted bikes:         0");
        } else {
            println!("  Relisted bikes:         {}", self.relisted.len());
            for l in &self.relisted {
                println!(
                    "    [RELIST] {} | {}km | {} | CHF {} | prev id {} | {}",
                    l.title,
                    l.mileage_km,
                    l.year,
                    l.price_chf,
                    l.previous_listing_id.unwrap_or(0),
                    l.url
                );
            }
        }

        println!("\nDatabase saved to {}", DB_PATH);
    }
}

pub fn merge_and_save(scraped: Vec<MotorcycleListing>) -> Result<StoreSummary, Box<dyn Error>> {
    // Load existing database
    let mut db: HashMap<u64, MotorcycleListing> = if std::path::Path::new(DB_PATH).exists() {
        let mut reader = csv::Reader::from_path(DB_PATH)?;
        reader
            .deserialize()
            .filter_map(|r: Result<MotorcycleListing, _>| r.ok())
            .map(|l| (l.listing_id, l))
            .collect()
    } else {
        HashMap::new()
    };

    // Build fingerprint → listing_id index for relist detection.
    // Note: if two DB entries share a fingerprint (identical model/year/km),
    // only one is indexed — ambiguous cases won't be flagged as relists.
    let fingerprint_index: HashMap<String, u64> = db
        .values()
        .map(|l| (l.fingerprint.clone(), l.listing_id))
        .collect();

    // Track which categories and IDs are present in this run
    let scraped_categories: HashSet<String> = scraped.iter().map(|l| l.category.clone()).collect();
    let current_ids: HashSet<u64> = scraped.iter().map(|l| l.listing_id).collect();

    let mut summary = StoreSummary {
        new: Vec::new(),
        updated: 0,
        sold: Vec::new(),
        relisted: Vec::new(),
    };

    for mut listing in scraped {
        if let Some(existing) = db.get_mut(&listing.listing_id) {
            // Known listing — refresh last_seen and price (price may change)
            existing.last_seen = listing.last_seen;
            existing.price_chf = listing.price_chf;
            existing.status = ListingStatus::Active;
            summary.updated += 1;
        } else {
            // New listing ID — check fingerprint for relist
            if let Some(&prev_id) = fingerprint_index.get(&listing.fingerprint) {
                listing.status = ListingStatus::Relisted;
                listing.previous_listing_id = Some(prev_id);
                summary.relisted.push(listing.clone());
            } else {
                summary.new.push(listing.clone());
            }
            db.insert(listing.listing_id, listing);
        }
    }

    // Only mark as sold within the categories we scraped this run.
    // Listings in other categories are untouched — we didn't check them.
    for listing in db.values_mut() {
        if listing.status == ListingStatus::Active
            && scraped_categories.contains(&listing.category)
            && !current_ids.contains(&listing.listing_id)
        {
            listing.status = ListingStatus::Sold;
            summary.sold.push(listing.clone());
        }
    }

    // Write back to CSV sorted by listing_id for stable, diffable output
    let mut rows: Vec<&MotorcycleListing> = db.values().collect();
    rows.sort_by_key(|l| l.listing_id);

    let mut writer = csv::Writer::from_path(DB_PATH)?;
    for listing in rows {
        writer.serialize(listing)?;
    }
    writer.flush()?;

    Ok(summary)
}
