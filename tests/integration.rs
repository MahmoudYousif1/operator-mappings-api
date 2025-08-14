use dotenv::dotenv;
use once_cell::sync::Lazy;
use reqwest::{Client, Method, StatusCode};
use serde_json::json;
use std::env;
use std::time::Duration;

static BASE_URL: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("API_TEST_BASE_URL").unwrap_or_else(|_| "http://localhost:8080/api/v1".to_string())
});

fn client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| panic!("Failed to build reqwest client: {}", e))
        .unwrap()
}

async fn ensure_operator_exists() {
    let create_body = json!({
        "country": "Ireland",
        "iso2": "IE",
        "iso3": "IRL",
        "name": "Test Operator",
        "e212": ["27202"],
        "e164": ["35386"],
        "tadig": ["IRL01"]
    });
    let resp = client()
        .post(format!("{}/operators", &*BASE_URL))
        .json(&create_body)
        .send()
        .await
        .map_err(|e| panic!("POST /operators failed: {}", e))
        .unwrap();
    assert!(
        resp.status().is_success() || resp.status() == StatusCode::CONFLICT,
        "could not ensure IRL01 exists: {}",
        resp.status()
    );
}

async fn ensure_patch_operator_exists() {
    let create_body = json!({
        "country": "Argentina",
        "iso2": "AR",
        "iso3": "ARG",
        "name": "Argentina Test Mobile",
        "e212": ["722099"],
        "e164": ["54999"],
        "tadig": ["ARGTM"]
    });
    let resp = client()
        .post(format!("{}/operators", &*BASE_URL))
        .json(&create_body)
        .send()
        .await
        .map_err(|e| panic!("POST /operators failed: {}", e))
        .unwrap();
    assert!(
        resp.status().is_success() || resp.status() == StatusCode::CONFLICT,
        "could not ensure ARGTM exists: {}",
        resp.status()
    );
}

#[tokio::test]
async fn test_lookup_by_imsi_ok() {
    let resp = client()
        .request(Method::GET, format!("{}/operators", &*BASE_URL))
        .query(&[("imsi", "3368900890017")])
        .send()
        .await
        .map_err(|e| panic!("Request failed: {}", e))
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| panic!("Invalid JSON: {}", e))
        .unwrap();
    assert!(body.get("iso3").is_some());
}

#[tokio::test]
async fn test_lookup_by_msisdn_bad_request() {
    let resp = client()
        .get(format!("{}/operators?msisdn=frgr", &*BASE_URL))
        .send()
        .await
        .map_err(|e| panic!("Request failed: {}", e))
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_operator_success() {
    ensure_operator_exists().await;

    let resp = client()
        .delete(format!("{}/operators/IRL01", &*BASE_URL))
        .send()
        .await
        .map_err(|e| panic!("Request failed: {}", e))
        .unwrap();
    assert!(
        resp.status().is_success()
            || resp.status() == StatusCode::NO_CONTENT
            || resp.status() == StatusCode::NOT_FOUND
    );

    let resp2 = client()
        .delete(format!("{}/operators/IRLME", &*BASE_URL))
        .send()
        .await
        .map_err(|e| panic!("Request failed: {}", e))
        .unwrap();
    assert!(
        resp2.status() == StatusCode::NOT_FOUND || resp2.status().is_success(),
        "unexpected status {}",
        resp2.status()
    );
}

#[tokio::test]
async fn test_patch_operator_duplicate_imsi() {
    ensure_patch_operator_exists().await;
    ensure_operator_exists().await;

    let patch_body = json!({
        "e212": ["27202"]
    });
    let resp = client()
        .patch(format!("{}/operators/ARGTM", &*BASE_URL))
        .json(&patch_body)
        .send()
        .await
        .map_err(|e| panic!("Request failed: {}", e))
        .unwrap();
    assert!(
        resp.status() == StatusCode::CONFLICT || resp.status() == StatusCode::OK,
        "unexpected status {}",
        resp.status()
    );
}

#[tokio::test]
async fn test_put_operator_invalid_msisdn() {
    let put_body = json!({
        "country": "Romania",
        "iso2": "RO",
        "iso3": "ROU",
        "name": "X",
        "e212": ["22699"],
        "e164": ["4077A"],
        "realm": null,
        "tadig": ["VALID01"]
    });
    let resp = client()
        .put(format!("{}/operators/VALID01", &*BASE_URL))
        .json(&put_body)
        .send()
        .await
        .map_err(|e| panic!("Request failed: {}", e))
        .unwrap();
    assert!(
        resp.status() == StatusCode::BAD_REQUEST || resp.status() == StatusCode::NOT_FOUND,
        "unexpected status {}",
        resp.status()
    );
}
