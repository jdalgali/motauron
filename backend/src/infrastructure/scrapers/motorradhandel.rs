use crate::application::ports::scraper::Scraper;
use crate::domain::entities::MotorcycleListing;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;

pub struct MotorradhandelScraper {
    pub client: Client,
    pub url: String,
}

impl MotorradhandelScraper {
    pub fn new(client: Client, url: &str) -> Self {
        Self {
            client,
            url: url.to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
struct MhStore {
    results: Vec<MhListing>,
}

#[derive(Deserialize, Debug)]
struct MhListing {
    id: u64,
    fzg_preis: Option<u32>,
    fzg_km: Option<u32>,
    fzg_1iv: Option<String>,
    kundentyp: Option<u8>,
    rel_marke: Option<RelMarke>,
    rel_modelle: Option<RelModelle>,
    rel_suchmh: Option<RelSuchMH>,
    standort: Option<Standort>,
}

#[derive(Deserialize, Debug)]
struct RelMarke {
    #[serde(rename = "Markenbezeichnung")]
    name: String,
}

#[derive(Deserialize, Debug)]
struct RelModelle {
    #[serde(rename = "Modellbezeichnung")]
    name: String,
}

#[derive(Deserialize, Debug)]
struct RelSuchMH {
    #[serde(rename = "SuchMH")]
    name: String,
}

#[derive(Deserialize, Debug)]
struct Standort {
    kunde_ort: Option<String>,
    kunde_kanton: Option<String>,
    firma_name: Option<String>,
}

fn to_slug(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

/// Derive a stable category key from brand + model, e.g. "yamaha-tenere-700".
fn derive_category(marke: &str, modell: &str) -> String {
    let b = to_slug(marke);
    let m = to_slug(modell);
    if b.is_empty() || m.is_empty() {
        return b + &m;
    }
    format!("{}-{}", b, m)
}

/// Build a page URL: page 1 uses the base URL as-is; subsequent pages append &p=N.
fn page_url(base: &str, page: u32) -> String {
    if page == 1 {
        base.to_string()
    } else {
        format!("{}&p={}", base, page)
    }
}

#[async_trait]
impl Scraper for MotorradhandelScraper {
    async fn scrape(&self) -> Result<Vec<MotorcycleListing>, Box<dyn Error>> {
        let mut all_results: Vec<MotorcycleListing> = Vec::new();
        let mut seen_ids: HashSet<u64> = HashSet::new();

        for page in 1..=100u32 {
            let url = page_url(&self.url, page);
            let page_listings = self.fetch_page(&url).await?;

            if page_listings.is_empty() {
                break;
            }

            let before = all_results.len();
            for listing in page_listings {
                if seen_ids.insert(listing.listing_id) {
                    all_results.push(listing);
                }
            }

            // No new unique listings — we've hit the end.
            if all_results.len() == before {
                break;
            }

            println!(
                "motorradhandel: page {} — {} listings so far",
                page,
                all_results.len()
            );
        }

        println!("motorradhandel: total {} listings", all_results.len());
        Ok(all_results)
    }
}

impl MotorradhandelScraper {
    async fn fetch_page(&self, url: &str) -> Result<Vec<MotorcycleListing>, Box<dyn Error>> {
        let response = self.client.get(url).send().await?.text().await?;

        let start_marker = "window.__store__";
        let store_idx = match response.find(start_marker) {
            Some(idx) => idx,
            None => {
                // Page doesn't exist or is empty — stop pagination.
                return Ok(vec![]);
            }
        };

        let json_start = store_idx
            + response[store_idx..]
                .find('{')
                .ok_or("no JSON object found after marker")?;

        let json_and_rest = &response[json_start..];
        let end_idx = json_and_rest
            .find("</script>")
            .ok_or("JSON end not found")?;

        let json_str = json_and_rest[..end_idx].trim().trim_end_matches(';');
        let store_data: MhStore = serde_json::from_str(json_str)?;

        if store_data.results.is_empty() {
            return Ok(vec![]);
        }

        let today = chrono::Local::now().date_naive();
        let mut results = Vec::new();

        for item in store_data.results {
            let Some(price) = item.fzg_preis else {
                continue;
            };

            let mileage = item.fzg_km.unwrap_or(0);

            let year = item
                .fzg_1iv
                .as_deref()
                .and_then(|s| s.split('-').next())
                .and_then(|y| y.parse::<u16>().ok())
                .unwrap_or(0);

            let marke = item.rel_marke.as_ref().map(|m| m.name.as_str()).unwrap_or("");
            let modell = item.rel_modelle.as_ref().map(|m| m.name.as_str()).unwrap_or("");
            let title = format!("{} {}", marke, modell).trim().to_string();

            if title.is_empty() {
                continue;
            }

            let category = derive_category(marke, modell);

            let brand_slug = to_slug(marke);
            let model_slug = item
                .rel_suchmh
                .as_ref()
                .map(|s| to_slug(&s.name))
                .unwrap_or_else(|| to_slug(modell));
            let url = format!(
                "https://motorradhandel.ch/en/d/{}/{}/{}",
                brand_slug, model_slug, item.id
            );

            let standort = item.standort.as_ref();
            let location = standort.and_then(|s| s.kunde_ort.clone()).unwrap_or_default();
            let kanton = standort.and_then(|s| s.kunde_kanton.clone()).unwrap_or_default();
            let seller_name = standort
                .and_then(|s| s.firma_name.clone())
                .unwrap_or_default();

            let is_private = item.kundentyp == Some(2);

            results.push(MotorcycleListing::new(
                item.id,
                today,
                category,
                title,
                price,
                mileage,
                year,
                url,
                location,
                kanton,
                is_private,
                seller_name,
            ));
        }

        Ok(results)
    }
}
