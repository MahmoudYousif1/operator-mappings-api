use dotenv::dotenv;
use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use std::env;

static BASE_URL: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("API_TEST_BASE_URL").unwrap_or_else(|_| "http://localhost:8080/api/v1".to_string())
});

fn client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap()
}

async fn ensure_operator_exists(tadig: &str) {
    let body = json!({
        "country": "Argentina",
        "iso2":     "AR",
        "iso3":     "ARG",
        "name":     "Argentina Test Mobile",
        "e212":     ["722099"],
        "e164":     ["54999"],
        "tadig":    [tadig]
    });
    let resp = client()
        .post(format!("{}/operators", &*BASE_URL))
        .json(&body)
        .send()
        .await
        .expect("POST /operators failed");
    assert!(
        resp.status().is_success() || resp.status() == StatusCode::CONFLICT,
        "{} setup failed: {}",
        tadig,
        resp.status()
    );
}

#[tokio::test]
async fn test_find_roaming_partners_success() {
    ensure_operator_exists("ARGTM").await;
    let resp = client()
        .get(format!("{}/operators/roaming-partners", &*BASE_URL))
        .query(&[("tadig", "ARGTM")])
        .send()
        .await
        .expect("GET /roaming-partners failed");
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.map_err(|e| panic!("{}", e)).unwrap();
    let mut got: Vec<String> = body["bordering_countries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    got.sort();
    let mut expected = vec![
        "Bolivia (Plurinational State Of)".to_string(),
        "Brazil".to_string(),
        "Chile".to_string(),
        "Paraguay".to_string(),
        "Uruguay".to_string(),
    ];
    expected.sort();
    assert_eq!(got, expected);
    let partners = body["partners"].as_array().unwrap();
    let message = body["message"].as_str().unwrap();
    let count: usize = message
        .trim_start_matches("Found ")
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(partners.len(), count);
}

#[tokio::test]
async fn test_find_roaming_partners_empty_tadig() {
    let resp = client()
        .get(format!("{}/operators/roaming-partners", &*BASE_URL))
        .query(&[("tadig", "")])
        .send()
        .await
        .expect("GET empty tadig failed");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_find_roaming_partners_tadig_not_found() {
    let resp = client()
        .get(format!("{}/operators/roaming-partners", &*BASE_URL))
        .query(&[("tadig", "DOES_NOT_EXIST")])
        .send()
        .await
        .expect("GET unknown tadig failed");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_find_roaming_partners_border_not_found() {
    let no_borders = json!({
        "country": "Narnia",
        "iso2":     "NA",
        "iso3":     "NAR",
        "name":     "Narnia Telecom",
        "e212":     ["00000"],
        "e164":     ["00000"],
        "tadig":    ["NARN01"]
    });
    client()
        .post(format!("{}/operators", &*BASE_URL))
        .json(&no_borders)
        .send()
        .await
        .expect("POST NARN01 failed");
    let resp = client()
        .get(format!("{}/operators/roaming-partners", &*BASE_URL))
        .query(&[("tadig", "NARN01")])
        .send()
        .await
        .expect("GET NARN01 failed");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
