use crate::app_state::model::{AppState, CountryBordersMap};
use crate::utils::models::Operator;
use csv::StringRecord;
use log::{info, warn};
use std::{collections::HashMap, env};
use tokio::fs;

pub async fn load_operator_mappings() -> Result<AppState, String> {
    let path = env::var("OPERATOR_MAPPINGS_FILE_PATH").unwrap_or_else(|e| {
        let default = "./resources/operator_mappings.json".to_string();
        warn!(
            "OPERATOR_MAPPINGS_FILE_PATH not set ({:?}), defaulting to {}",
            e, default
        );
        default
    });

    let data = fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read '{}': {}", path, e))?;

    let list: Vec<Operator> =
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    info!("Loaded {} operators from {}", list.len(), path);
    Ok(AppState::new(list, path, HashMap::new()))
}

pub async fn load_country_borders() -> Result<CountryBordersMap, String> {
    let path = env::var("COUNTRY_BORDERS_CSV_FILE_PATH").unwrap_or_else(|e| {
        let default = "./resources/country_borders.csv".to_string();
        warn!(
            "COUNTRY_BORDERS_CSV_FILE_PATH not set ({:?}), defaulting to {}",
            e, default
        );
        default
    });

    let raw_csv = fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read '{}': {}", path, e))?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(raw_csv.as_bytes());

    let mut exact_borders: CountryBordersMap = HashMap::new();

    for result in rdr.records() {
        let record: StringRecord = result.map_err(|e| format!("CSV parse error: {}", e))?;
        if record.len() < 4 {
            continue;
        }
        let base_country = record[1].trim().to_string();
        let border_country = record[3].trim().to_string();
        exact_borders
            .entry(base_country)
            .or_default()
            .push(border_country);
    }

    Ok(exact_borders)
}
