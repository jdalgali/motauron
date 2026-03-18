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
    fzg_1iv: Option<String>, // First registration date, e.g. "2024-11-27"
    rel_marke: Option<RelMarke>,
    rel_modelle: Option<RelModelle>,
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

pub async fn scrape_category(
    client: &Client,
    category: &str,
    url: &str,
) -> Result<Vec<MotorcycleListing>, Box<dyn Error>> {
    println!("  Scraping: {}", category);

    let response = client.get(url).send().await?.text().await?;

    let start_marker = "window.__store__";
    let store_idx = match response.find(start_marker) {
        Some(idx) => idx,
        None => {
            std::fs::write("error_page.html", &response).unwrap();
            return Err("JSON start not found! Page saved as 'error_page.html'.".into());
        }
    };

    let json_start = store_idx
        + response[store_idx..]
            .find('{')
            .ok_or("No JSON object found after marker")?;

    let json_and_rest = &response[json_start..];
    let end_idx = json_and_rest
        .find("</script>")
        .ok_or("JSON end (</script>) not found")?;

    let json_str = json_and_rest[..end_idx].trim().trim_end_matches(';');
    let store_data: MhStore = serde_json::from_str(json_str)?;

    let today = chrono::Local::now().date_naive();
    let mut results = Vec::new();

    for item in store_data.results {
        let Some(price) = item.fzg_preis else {
            continue;
        };

        let mileage = item.fzg_km.unwrap_or(0);

        // Extract year from first-registration date string "YYYY-MM-DD"
        let year = item
            .fzg_1iv
            .as_deref()
            .and_then(|s| s.split('-').next())
            .and_then(|y| y.parse::<u16>().ok())
            .unwrap_or(0);

        let marke = item.rel_marke.map(|m| m.name).unwrap_or_default();
        let modell = item.rel_modelle.map(|m| m.name).unwrap_or_default();
        let title = format!("{} {}", marke, modell).trim().to_string();

        if title.is_empty() {
            continue;
        }

        let url = format!("https://motorradhandel.ch/vi/{}", item.id);

        results.push(MotorcycleListing::new(
            item.id,
            today,
            category.to_string(),
            title,
            price,
            mileage,
            year,
            url,
        ));
    }

    Ok(results)
}
