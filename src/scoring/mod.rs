use crate::models::motorcycle::MotorcycleListing;
use std::collections::HashMap;

/// CHF per km used for mileage adjustment within a peer group.
const CHF_PER_KM: f64 = 0.7;

/// Dealers are expected to charge ~7% more than private sellers
/// (warranty, prep, margin). A dealer at median price = "good".
/// A private seller at median price = "fair".
const DEALER_PREMIUM: f64 = 1.07;

pub fn score_category(listings: &mut [MotorcycleListing]) {
    // Group by (title, year) first for variant-aware comparison.
    // Falls back to year-only if the variant group has fewer than 2 listings.
    let mut title_year_groups: HashMap<(String, u16), Vec<usize>> = HashMap::new();
    let mut year_groups: HashMap<u16, Vec<usize>> = HashMap::new();

    for (i, l) in listings.iter().enumerate() {
        title_year_groups
            .entry((l.title.to_lowercase(), l.year))
            .or_default()
            .push(i);
        year_groups.entry(l.year).or_default().push(i);
    }

    for i in 0..listings.len() {
        let key = (listings[i].title.to_lowercase(), listings[i].year);
        let year = listings[i].year;

        let peers = if title_year_groups[&key].len() >= 2 {
            title_year_groups[&key].clone()
        } else if year_groups[&year].len() >= 2 {
            year_groups[&year].clone()
        } else {
            listings[i].price_label = "n/a".to_string();
            listings[i].score_peers = 1;
            continue;
        };

        let prices: Vec<u32> = peers.iter().map(|&j| listings[j].price_chf).collect();
        let kms: Vec<u32> = peers.iter().map(|&j| listings[j].mileage_km).collect();
        let median_price = median(&prices) as f64;
        let median_km = median(&kms) as f64;

        // Cap km delta to avoid outsized adjustments from outlier peers
        let km_delta = (median_km - listings[i].mileage_km as f64).clamp(-5_000.0, 5_000.0);
        let mileage_adjusted = median_price + km_delta * CHF_PER_KM;

        let loc_factor = canton_multiplier(&listings[i].kanton);
        // Dealers are expected to charge more — adjust the bar upward for them
        let seller_factor = if listings[i].is_private { 1.0 } else { DEALER_PREMIUM };
        let expected = mileage_adjusted * loc_factor * seller_factor;

        let delta_pct = ((expected - listings[i].price_chf as f64) / expected * 100.0).round() as i32;

        listings[i].price_score = delta_pct;
        listings[i].price_label = label(delta_pct).to_string();
        listings[i].score_peers = peers.len().min(255) as u8;
    }
}

fn median(values: &[u32]) -> u32 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

fn canton_multiplier(kanton: &str) -> f64 {
    match kanton.to_uppercase().as_str() {
        "ZH" => 1.08,
        "GE" => 1.07,
        "ZG" => 1.06,
        "VD" => 1.05,
        "BS" | "BL" => 1.04,
        "AG" | "SO" => 1.02,
        "BE" | "LU" | "TG" | "SH" | "SG" => 1.00,
        "FR" | "NE" => 0.97,
        "TI" | "GR" => 0.96,
        "VS" | "JU" | "UR" | "OW" | "NW" | "SZ" | "GL" | "AR" | "AI" => 0.94,
        _ => 1.00,
    }
}

fn label(delta_pct: i32) -> &'static str {
    match delta_pct {
        15.. => "great deal",
        7..=14 => "good",
        -6..=6 => "fair",
        -14..=-7 => "overpriced",
        _ => "expensive",
    }
}
