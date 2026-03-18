use crate::models::motorcycle::{ListingStatus, MotorcycleListing};
use crate::scoring;
use std::collections::{HashMap, HashSet};
use std::error::Error;

const DB_PATH: &str = "listings_db.csv";

pub struct PriceChange {
    pub listing: MotorcycleListing,
    pub old_price: u32,
}

pub struct StoreSummary {
    pub new: Vec<MotorcycleListing>,
    pub updated: usize,
    pub sold: Vec<MotorcycleListing>,
    pub relisted: Vec<MotorcycleListing>,
    pub price_changes: Vec<PriceChange>,
    pub total_tracked: usize,
}

impl StoreSummary {
    pub fn print(&self) {
        println!();
        println!("  new           {}", self.new.len());
        for l in &self.new {
            let seller = if l.is_private { "private" } else { &l.seller_name };
            println!(
                "    + {} · {}km (~{}km/yr) · {} · chf {} · {} · {} · {} ({:+}%, vs {})",
                l.title.to_lowercase(),
                l.mileage_km,
                l.annual_km(),
                l.year,
                l.price_chf,
                l.location,
                seller,
                l.price_label,
                l.price_score,
                l.score_peers,
            );
            println!("      {}", l.url);
        }

        println!("  price changes {}", self.price_changes.len());
        for pc in &self.price_changes {
            let l = &pc.listing;
            let diff = l.price_chf as i64 - pc.old_price as i64;
            let arrow = if diff < 0 { "↓" } else { "↑" };
            println!(
                "    {} {} · chf {} → {} ({:+}) · {} · {}",
                arrow,
                l.title.to_lowercase(),
                pc.old_price,
                l.price_chf,
                diff,
                l.location,
                l.url,
            );
        }

        println!("  sold          {}", self.sold.len());
        for l in &self.sold {
            let days = (l.last_seen - l.first_seen).num_days();
            println!(
                "    - {} · {}km · {} · chf {} · {} · {} days on market",
                l.title.to_lowercase(),
                l.mileage_km,
                l.year,
                l.price_chf,
                l.location,
                days,
            );
        }

        println!("  relisted      {}", self.relisted.len());
        for l in &self.relisted {
            let seller = if l.is_private { "private" } else { &l.seller_name };
            println!(
                "    ~ {} · {}km · {} · chf {} · {} · {} · {} ({:+}%, vs {}) · was id {}",
                l.title.to_lowercase(),
                l.mileage_km,
                l.year,
                l.price_chf,
                l.location,
                seller,
                l.price_label,
                l.price_score,
                l.score_peers,
                l.previous_listing_id.unwrap_or(0),
            );
            println!("      {}", l.url);
        }

        println!("  updated       {}", self.updated);
        println!();
        println!("  tracking  {} listings — {}", self.total_tracked, DB_PATH);
    }
}

pub fn merge_and_save(scraped: Vec<MotorcycleListing>) -> Result<StoreSummary, Box<dyn Error>> {
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

    let fingerprint_index: HashMap<String, u64> = db
        .values()
        .map(|l| (l.fingerprint.clone(), l.listing_id))
        .collect();

    let scraped_categories: HashSet<String> = scraped.iter().map(|l| l.category.clone()).collect();
    let current_ids: HashSet<u64> = scraped.iter().map(|l| l.listing_id).collect();

    let mut summary = StoreSummary {
        new: Vec::new(),
        updated: 0,
        sold: Vec::new(),
        relisted: Vec::new(),
        price_changes: Vec::new(),
        total_tracked: 0,
    };

    for mut listing in scraped {
        if let Some(existing) = db.get_mut(&listing.listing_id) {
            // Detect price change before overwriting
            if listing.price_chf != existing.price_chf {
                summary.price_changes.push(PriceChange {
                    listing: existing.clone(),
                    old_price: existing.price_chf,
                });
            }
            existing.last_seen = listing.last_seen;
            existing.price_chf = listing.price_chf;
            existing.status = ListingStatus::Active;
            // Refresh mutable fields that can change between runs
            existing.is_private = listing.is_private;
            existing.seller_name = listing.seller_name;
            summary.updated += 1;
        } else {
            // original_price_chf is already set in MotorcycleListing::new
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

    for listing in db.values_mut() {
        if listing.status == ListingStatus::Active
            && scraped_categories.contains(&listing.category)
            && !current_ids.contains(&listing.listing_id)
        {
            listing.status = ListingStatus::Sold;
            summary.sold.push(listing.clone());
        }
    }

    // Re-score all active listings in scraped categories with fresh market context
    for category in &scraped_categories {
        let mut group: Vec<MotorcycleListing> = db
            .values()
            .filter(|l| &l.category == category && l.status == ListingStatus::Active)
            .cloned()
            .collect();

        scoring::score_category(&mut group);

        for scored in group {
            if let Some(l) = db.get_mut(&scored.listing_id) {
                l.price_score = scored.price_score;
                l.price_label = scored.price_label;
                l.score_peers = scored.score_peers;
            }
        }
    }

    // Refresh summary entries with scored versions from db
    for l in summary.new.iter_mut().chain(summary.relisted.iter_mut()) {
        if let Some(scored) = db.get(&l.listing_id) {
            l.price_score = scored.price_score;
            l.price_label = scored.price_label.clone();
            l.score_peers = scored.score_peers;
        }
    }

    // Update price_changes entries with fresh scores too
    for pc in summary.price_changes.iter_mut() {
        if let Some(scored) = db.get(&pc.listing.listing_id) {
            pc.listing.price_chf = scored.price_chf;
            pc.listing.price_score = scored.price_score;
            pc.listing.price_label = scored.price_label.clone();
        }
    }

    summary.total_tracked = db.len();

    let mut rows: Vec<&MotorcycleListing> = db.values().collect();
    rows.sort_by_key(|l| l.listing_id);

    // Write UTF-8 BOM so Excel/LibreOffice auto-detects the encoding
    let file = std::fs::File::create(DB_PATH)?;
    let mut buf = std::io::BufWriter::new(file);
    std::io::Write::write_all(&mut buf, b"\xef\xbb\xbf")?;
    let mut writer = csv::Writer::from_writer(buf);
    for listing in rows {
        writer.serialize(listing)?;
    }
    writer.flush()?;

    Ok(summary)
}
