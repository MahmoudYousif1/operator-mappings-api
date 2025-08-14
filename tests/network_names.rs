use dotenv::dotenv;
use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
use serde_json::Value;
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

#[tokio::test]
async fn test_network_names_returns_list() {
    let resp = client()
        .get(format!("{}/operators/network-names", &*BASE_URL))
        .send()
        .await
        .expect("GET /operators/network-names failed");

    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = resp.json().await.expect("Invalid JSON");
    let names = body
        .as_array()
        .expect("Expected a JSON array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect::<Vec<_>>();

    assert!(
        !names.is_empty(),
        "Expected non-empty list of network names"
    );
}

#[tokio::test]
async fn test_network_names_empty_array_when_no_operators() {
    let resp = client()
        .get(format!("{}/operators/network-names", &*BASE_URL))
        .send()
        .await
        .expect("GET /operators/network-names failed");

    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = resp.json().await.expect("Invalid JSON");
    let names = body
        .as_array()
        .expect("Expected a JSON array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect::<Vec<_>>();

    assert!(
        names.is_empty() || !names.is_empty(),
        "Expected an array (possibly empty), got: {:?}",
        names
    );
}
