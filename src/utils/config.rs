use actix_web::web;
use serde_yaml;
use std::{env, fs};
use utoipa::openapi::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Debug, Clone)]
pub struct Config {
    pub save_interval_minutes: u64,
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub operator_mappings_file_path: String,
    pub country_borders_file_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            save_interval_minutes: 60,
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: 4,
            operator_mappings_file_path: "./resources/operator_mappings.json".to_string(),
            country_borders_file_path: "./resources/country_borders.csv".to_string(),
        }
    }
}

impl Config {
    fn parse_env_u64(key: &str, default: u64) -> u64 {
        match env::var(key) {
            Ok(s) => s.parse().unwrap_or(default),
            Err(_) => default,
        }
    }

    fn parse_env_u16(key: &str, default: u16) -> u16 {
        match env::var(key) {
            Ok(s) => s.parse().unwrap_or(default),
            Err(_) => default,
        }
    }

    fn parse_env_usize(key: &str, default: usize) -> usize {
        match env::var(key) {
            Ok(s) => s.parse().unwrap_or(default),
            Err(_) => default,
        }
    }

    fn parse_env_string(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    pub fn from_env() -> Self {
        let d = Config::default();
        Config {
            save_interval_minutes: Self::parse_env_u64(
                "SAVE_INTERVAL_MINUTES",
                d.save_interval_minutes,
            ),
            host: Self::parse_env_string("HOST", &d.host),
            port: Self::parse_env_u16("PORT", d.port),
            workers: Self::parse_env_usize("WORKERS", d.workers),
            operator_mappings_file_path: Self::parse_env_string(
                "OPERATOR_MAPPINGS_FILE_PATH",
                &d.operator_mappings_file_path,
            ),
            country_borders_file_path: Self::parse_env_string(
                "COUNTRY_BORDERS_CSV_FILE_PATH",
                &d.country_borders_file_path,
            ),
        }
    }
}

pub fn load() -> Config {
    Config::from_env()
}

pub fn configure_swagger(cfg: &mut web::ServiceConfig) {
    let yaml_path =
        env::var("OPENAPI_YAML_PATH").unwrap_or_else(|_| "./resources/api.yaml".to_string());
    let yaml = fs::read_to_string(&yaml_path).expect("Failed to read OPENAPI_YAML_PATH");
    let spec: OpenApi = serde_yaml::from_str(&yaml).expect("YAML was not valid OpenAPI v3.1");
    cfg.service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", spec));
}
