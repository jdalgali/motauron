// Tracks which listings have already been pushed so we don't re-notify on
// every run when nothing has changed.
//
// Format: JSON map of listing_id (as string) → price_chf at notification time.
// A new notification fires when the listing is new to this map OR the price
// has changed since the last notification.

use std::collections::HashMap;
use std::error::Error;

const STATE_PATH: &str = "notified.json";

pub type NotifyState = HashMap<u64, u32>;

pub fn load() -> NotifyState {
    let Ok(raw) = std::fs::read_to_string(STATE_PATH) else {
        return HashMap::new();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save(state: &NotifyState) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(STATE_PATH, json)?;
    Ok(())
}

/// Returns true if this listing+price combo was already notified.
pub fn already_sent(state: &NotifyState, listing_id: u64, price_chf: u32) -> bool {
    state.get(&listing_id) == Some(&price_chf)
}
