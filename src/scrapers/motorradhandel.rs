use crate::models::motorcycle::MotorcycleListing;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;

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

/// The search model handle — its name is used as the URL slug
#[derive(Deserialize, Debug)]
struct RelSuchMH {
    #[serde(rename = "SuchMH")]
    name: String,
}

#[derive(Deserialize, Debug)]
struct Standort {
    kunde_ort: Option<String>,
    kunde_kanton: Option<String>,
}

fn to_url_slug(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

pub async fn scrape_category(
    client: &Client,
    category: &str,
    url: &str,
) -> Result<Vec<MotorcycleListing>, Box<dyn Error>> {
    let response = client.get(url).send().await?.text().await?;

    let start_marker = "window.__store__";
    let store_idx = match response.find(start_marker) {
        Some(idx) => idx,
        None => {
            std::fs::write("error_page.html", &response).unwrap();
            return Err("JSON start not found — page saved as 'error_page.html'".into());
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

        // Build URL from brand slug + search-model slug + id
        // e.g. https://motorradhandel.ch/en/d/yamaha/tenere-700/8637084
        let brand_slug = to_url_slug(marke);
        let model_slug = item
            .rel_suchmh
            .as_ref()
            .map(|s| to_url_slug(&s.name))
            .unwrap_or_else(|| to_url_slug(modell));
        let url = format!(
            "https://motorradhandel.ch/en/d/{}/{}/{}",
            brand_slug, model_slug, item.id
        );

        let location = item
            .standort
            .as_ref()
            .and_then(|s| s.kunde_ort.clone())
            .unwrap_or_default();
        let kanton = item
            .standort
            .as_ref()
            .and_then(|s| s.kunde_kanton.clone())
            .unwrap_or_default();

        results.push(MotorcycleListing::new(
            item.id,
            today,
            category.to_string(),
            title,
            price,
            mileage,
            year,
            url,
            location,
            kanton,
        ));
    }

    Ok(results)
}
