use crate::application::ports::notifier::Notifier;
use crate::domain::entities::MotorcycleListing;
use crate::domain::services::merger::{MarketSummary, PriceChange};
use async_trait::async_trait;
use chrono::Local;
use reqwest::Client;
use std::collections::HashMap;
use std::error::Error;

const STATE_PATH: &str = "notified.json";

#[derive(Clone)]
pub struct NtfyConfig {
    pub url: String,
    pub token: Option<String>,
}

pub struct NtfyNotifier {
    pub client: Client,
    pub config: Option<NtfyConfig>,
    pub min_alert_score: i32,
}

impl NtfyNotifier {
    pub fn new(client: Client, config: Option<NtfyConfig>, min_alert_score: i32) -> Self {
        Self {
            client,
            config,
            min_alert_score,
        }
    }

    fn load_state(&self) -> HashMap<u64, u32> {
        let Ok(raw) = std::fs::read_to_string(STATE_PATH) else {
            return HashMap::new();
        };
        serde_json::from_str(&raw).unwrap_or_default()
    }

    fn save_state(&self, state: &HashMap<u64, u32>) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(STATE_PATH, json)?;
        Ok(())
    }

    async fn send_msg(
        &self,
        cfg: &NtfyConfig,
        title: String,
        body: String,
        priority: u8,
        tags: Vec<&'static str>,
        click_url: Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        let mut req = self.client
            .post(&cfg.url)
            .header("Title", title)
            .header("Priority", priority.to_string())
            .header("Tags", tags.join(","))
            .body(body);

        if let Some(url) = click_url {
            req = req.header("Click", url);
        }
        if let Some(token) = &cfg.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        req.send().await?.error_for_status()?;
        Ok(())
    }
}

#[async_trait]
impl Notifier for NtfyNotifier {
    async fn send_summary(&self, summary: &MarketSummary) -> Result<(), Box<dyn Error>> {
        let Some(cfg) = &self.config else {
            return Ok(());
        };

        let mut state = self.load_state();

        let deals: Vec<&MotorcycleListing> = summary
            .new
            .iter()
            .filter(|l| l.price_score >= self.min_alert_score)
            .filter(|l| state.get(&l.listing_id) != Some(&l.price_chf))
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

            self.send_msg(cfg, format!("openclaw · new {}", l.price_label), body, priority, tags, Some(l.url.clone())).await?;
            state.insert(l.listing_id, l.price_chf);
        }

        let drops: Vec<&PriceChange> = summary
            .price_changes
            .iter()
            .filter(|pc| {
                let l = &pc.listing;
                let crossed_threshold = pc.old_price > l.price_chf && l.price_score >= self.min_alert_score;
                let already = state.get(&l.listing_id) == Some(&l.price_chf);
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

            self.send_msg(cfg, format!("openclaw · price drop {}", l.title.to_lowercase()), body, 3, vec!["motorcycle", "arrow_down"], Some(l.url.clone())).await?;
            state.insert(l.listing_id, l.price_chf);
        }

        let rest: Vec<&MotorcycleListing> = summary
            .new
            .iter()
            .filter(|l| l.price_score < self.min_alert_score)
            .filter(|l| state.get(&l.listing_id) != Some(&l.price_chf))
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

            let title = format!("openclaw · {} new listing{}", rest.len(), if rest.len() == 1 { "" } else { "s" });
            self.send_msg(cfg, title, lines.join("\n"), 2, vec!["motorcycle"], None).await?;

            for l in &rest {
                state.insert(l.listing_id, l.price_chf);
            }
        }

        let now = Local::now().format("%d.%m.%Y %H:%M");
        let deal_lines: Vec<String> = summary
            .top_deals
            .iter()
            .map(|l| {
                format!(
                    "· {} · {}km · {} · CHF {} · {} ({:+}%)",
                    l.title, l.mileage_km, l.year, l.price_chf, l.price_label, l.price_score
                )
            })
            .collect();
            
        let heartbeat_body = if deal_lines.is_empty() {
            format!("tracking {} listings", summary.total_tracked)
        } else {
            format!(
                "tracking {} listings\n\ntop deals:\n{}",
                summary.total_tracked,
                deal_lines.join("\n")
            )
        };

        self.send_msg(cfg, format!("openclaw · {}", now), heartbeat_body, 1, vec!["motorcycle"], None).await?;

        self.save_state(&state)?;

        Ok(())
    }
}
