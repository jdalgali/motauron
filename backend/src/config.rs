use serde::Deserialize;
use std::error::Error;

const CONFIG_PATH: &str = "config.toml";

/// The all-motorcycles search URL on motorradhandel.ch.
/// Users can replace this with any filtered search URL from the site.
const DEFAULT_MH_URL: &str =
    "https://motorradhandel.ch/en/all-motorbikes-and-scooters-search-switzerland?q=%7B%22arts%22%3A%5B1%5D%7D";

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub notify: NotifySection,
    #[serde(default)]
    pub targets: Vec<Target>,
}

/// One search URL to scrape from motorradhandel.ch, plus optional generation classification rules.
/// Paste any motorradhandel search URL — the scraper will paginate through all results.
///
/// Both field names are accepted for backwards compatibility:
///   `url = "..."`                — new format
///   `motorradhandel_url = "..."` — old format (also accepted)
/// The old `category` and `motoscout_url` fields are silently ignored.
#[derive(Deserialize, Clone)]
pub struct Target {
    // `url` is the canonical field; `motorradhandel_url` is the legacy alias.
    #[serde(alias = "motorradhandel_url")]
    pub url: Option<String>,

    // Legacy fields — present in old config.toml files, no longer used.
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub motoscout_url: Option<String>,

    #[serde(default)]
    pub generations: Vec<GenerationRule>,
}

impl Target {
    /// Returns the effective scrape URL, falling back to the all-motorcycles default.
    pub fn effective_url(&self) -> String {
        self.url
            .clone()
            .unwrap_or_else(|| DEFAULT_MH_URL.to_string())
    }
}

impl Default for Target {
    fn default() -> Self {
        Self {
            url: Some(DEFAULT_MH_URL.to_string()),
            category: None,
            motoscout_url: None,
            generations: vec![],
        }
    }
}

/// Classifies a listing into a named generation based on year range or title keyword.
/// Rules are evaluated in order; the first match wins.
/// `title_contains` is checked before year ranges so variant names (e.g. "World Raid") win.
#[derive(Deserialize, Clone)]
pub struct GenerationRule {
    pub name: String,
    pub title_contains: Option<String>,
    pub year_from: Option<u16>,
    pub year_to: Option<u16>,
}

#[derive(Deserialize)]
pub struct AgentConfig {
    pub interval_hours: u64,
    pub min_alert_score: i32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            interval_hours: 4,
            min_alert_score: 7,
        }
    }
}

#[derive(Deserialize, Default)]
pub struct NotifySection {
    pub ntfy: Option<NtfySection>,
}

#[derive(Deserialize)]
pub struct NtfySection {
    pub url: String,
    pub token: Option<String>,
}

pub fn load() -> Result<Config, Box<dyn Error>> {
    let mut cfg = if std::path::Path::new(CONFIG_PATH).exists() {
        let raw = std::fs::read_to_string(CONFIG_PATH)?;
        toml::from_str(&raw)?
    } else {
        Config::default()
    };

    // Environment variables override config file — useful for container deployments.
    if let Ok(url) = std::env::var("NTFY_URL") {
        cfg.notify.ntfy = Some(NtfySection {
            url,
            token: std::env::var("NTFY_TOKEN").ok(),
        });
    }

    Ok(cfg)
}
