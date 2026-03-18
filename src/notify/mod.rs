pub mod ntfy;
pub mod state;

use crate::store::{PriceChange, StoreSummary};
use crate::models::motorcycle::MotorcycleListing;
use ntfy::Message;
use reqwest::Client;
use state::NotifyState;
use std::error::Error;

// Score threshold to trigger an alert (7 = "good", 15 = "great deal").
// Listings below this threshold are included in the digest but don't
// generate a high-priority push.
const ALERT_SCORE: i32 = 7;

#[derive(Clone)]
pub enum Backend {
    Ntfy(ntfy::NtfyConfig),
    Disabled,
}

#[derive(Clone)]
pub struct NotifyConfig {
    pub backend: Backend,
    pub min_alert_score: i32,
}

impl Default for NotifyConfig {
    fn default() -> Self {
        Self {
            backend: Backend::Disabled,
            min_alert_score: ALERT_SCORE,
        }
    }
}

/// Send a test ping to verify the notification pipeline is working.
pub async fn test_ping(client: &Client, config: &NotifyConfig) -> Result<(), Box<dyn Error>> {
    let Backend::Ntfy(ref cfg) = config.backend else {
        return Err("no notification backend configured — add [notify.ntfy] to config.toml".into());
    };
    ntfy::send(
        client,
        cfg,
        Message {
            title: "openclaw · test".to_string(),
            body: "notification pipeline working".to_string(),
            priority: 3,
            tags: vec!["motorcycle", "white_check_mark"],
            click_url: None,
        },
    )
    .await
}

/// Send notifications for everything interesting in this scrape run.
/// Skips listing+price combos already in `state` to avoid duplicate pushes.
pub async fn dispatch(
    client: &Client,
    config: &NotifyConfig,
    summary: &StoreSummary,
    state: &mut NotifyState,
) -> Result<(), Box<dyn Error>> {
    let Backend::Ntfy(ref cfg) = config.backend else {
        return Ok(());
    };

    // --- deals: new listings at or above the alert threshold ---
    let deals: Vec<&MotorcycleListing> = summary
        .new
        .iter()
        .filter(|l| l.price_score >= config.min_alert_score)
        .filter(|l| !state::already_sent(state, l.listing_id, l.price_chf))
        .collect();

    for l in &deals {
        let seller = if l.is_private { "private" } else { &l.seller_name };
        let body = format!(
            "{} · {}km · {} · CHF {} · {} · {}\n{} ({:+}%, vs {})\n{}",
            l.title,
            l.mileage_km,
            l.year,
            l.price_chf,
            l.location,
            seller,
            l.price_label,
            l.price_score,
            l.score_peers,
            l.url,
        );
        let priority = if l.price_score >= 15 { 4 } else { 3 };
        let tags = if l.price_score >= 15 {
            vec!["motorcycle", "white_check_mark"]
        } else {
            vec!["motorcycle"]
        };

        ntfy::send(
            client,
            cfg,
            Message {
                title: format!("openclaw · new {}", l.price_label),
                body,
                priority,
                tags,
                click_url: Some(l.url.clone()),
            },
        )
        .await?;

        state.insert(l.listing_id, l.price_chf);
    }

    // --- price drops that push a listing into or above the alert threshold ---
    let drops: Vec<&PriceChange> = summary
        .price_changes
        .iter()
        .filter(|pc| {
            let l = &pc.listing;
            let crossed_threshold = pc.old_price > l.price_chf
                && l.price_score >= config.min_alert_score;
            let already = state::already_sent(state, l.listing_id, l.price_chf);
            crossed_threshold && !already
        })
        .collect();

    for pc in &drops {
        let l = &pc.listing;
        let diff = l.price_chf as i64 - pc.old_price as i64;
        let body = format!(
            "{} · {}km · {} · CHF {} → {} ({:+})\n{} ({:+}%, vs {})\n{}",
            l.title,
            l.mileage_km,
            l.year,
            pc.old_price,
            l.price_chf,
            diff,
            l.price_label,
            l.price_score,
            l.score_peers,
            l.url,
        );

        ntfy::send(
            client,
            cfg,
            Message {
                title: format!("openclaw · price drop {}", l.title.to_lowercase()),
                body,
                priority: 3,
                tags: vec!["motorcycle", "arrow_down"],
                click_url: Some(l.url.clone()),
            },
        )
        .await?;

        state.insert(l.listing_id, l.price_chf);
    }

    // --- digest: remaining new listings below the threshold (batched) ---
    let rest: Vec<&MotorcycleListing> = summary
        .new
        .iter()
        .filter(|l| l.price_score < config.min_alert_score)
        .filter(|l| !state::already_sent(state, l.listing_id, l.price_chf))
        .collect();

    if !rest.is_empty() {
        let lines: Vec<String> = rest
            .iter()
            .map(|l| {
                format!(
                    "· {} · {}km · {} · CHF {} · {} ({:+}%)",
                    l.title, l.mileage_km, l.year, l.price_chf, l.price_label, l.price_score
                )
            })
            .collect();

        ntfy::send(
            client,
            cfg,
            Message {
                title: format!("openclaw · {} new listing{}", rest.len(), if rest.len() == 1 { "" } else { "s" }),
                body: lines.join("\n"),
                priority: 2,
                tags: vec!["motorcycle"],
                click_url: None,
            },
        )
        .await?;

        for l in &rest {
            state.insert(l.listing_id, l.price_chf);
        }
    }

    Ok(())
}
