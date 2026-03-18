use crate::models::motorcycle::MotorcycleListing;
use std::collections::HashSet;

/// CHF per km used for mileage adjustment within a year group.
/// Rough real-world depreciation for used Swiss motorcycles.
const CHF_PER_KM: f64 = 0.7;

/// Score all listings in a slice against their year-group peers.
/// Scores are stored directly on each listing.
pub fn score_category(listings: &mut [MotorcycleListing]) {
    let years: HashSet<u16> = listings.iter().map(|l| l.year).collect();

    for year in years {
        let indices: Vec<usize> = listings
            .iter()
            .enumerate()
            .filter(|(_, l)| l.year == year)
            .map(|(i, _)| i)
            .collect();

        if indices.len() < 2 {
            for &i in &indices {
                listings[i].price_label = "n/a".to_string();
            }
            continue;
        }

        let mut prices: Vec<u32> = indices.iter().map(|&i| listings[i].price_chf).collect();
        prices.sort_unstable();
        let median_price = prices[prices.len() / 2] as f64;

        let mut kms: Vec<u32> = indices.iter().map(|&i| listings[i].mileage_km).collect();
        kms.sort_unstable();
        let median_km = kms[kms.len() / 2] as f64;

        for &idx in &indices {
            let l = &listings[idx];

            // Expected price: start from median, adjust for mileage deviation
            let km_delta = median_km - l.mileage_km as f64;
            let mileage_adjusted = median_price + km_delta * CHF_PER_KM;

            // Location multiplier: accounts for regional price differences
            let loc_factor = canton_multiplier(&l.kanton);
            let expected = mileage_adjusted * loc_factor;

            // Positive score = cheaper than expected (good deal)
            // Negative score = more expensive than expected
            let delta_pct =
                ((expected - l.price_chf as f64) / expected * 100.0).round() as i32;

            listings[idx].price_score = delta_pct;
            listings[idx].price_label = label(delta_pct).to_string();
        }
    }
}

/// Regional price multipliers based on cost-of-living and market dynamics.
/// Zürich and Geneva sellers price higher; Valais and Jura tend lower.
fn canton_multiplier(kanton: &str) -> f64 {
    match kanton.to_uppercase().as_str() {
        "ZH" => 1.08,
        "GE" => 1.07,
        "VD" => 1.05,
        "ZG" => 1.06,
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
