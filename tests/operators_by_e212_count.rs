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
async fn test_grouped_by_e212_success() {
    let resp = client()
        .get(format!("{}/operators/grouped-by-e212", &*BASE_URL))
        .send()
        .await
        .expect("GET /operators/grouped-by-e212 failed");
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = resp.json().await.expect("Invalid JSON");

    let small_count = body["small_size_operators"].as_u64().unwrap() as usize;
    let medium_count = body["medium_size_operators"].as_u64().unwrap() as usize;
    let large_count = body["large_size_operators"].as_u64().unwrap() as usize;

    let small_ops = body["small_operators"].as_array().unwrap();
    let medium_ops = body["medium_operators"].as_array().unwrap();
    let large_ops = body["large_operators"].as_array().unwrap();

    assert_eq!(
        small_count,
        small_ops.len(),
        "small_size_operators should match number of small_operators"
    );
    assert_eq!(
        medium_count,
        medium_ops.len(),
        "medium_size_operators should match number of medium_operators"
    );
    assert_eq!(
        large_count,
        large_ops.len(),
        "large_size_operators should match number of large_operators"
    );
}

#[tokio::test]
async fn test_grouped_by_e212_schema() {
    let resp = client()
        .get(format!("{}/operators/grouped-by-e212", &*BASE_URL))
        .send()
        .await
        .expect("GET /operators/grouped-by-e212 failed");
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = resp.json().await.expect("Invalid JSON");

    assert!(body.get("small_size_operators").unwrap().is_number());
    assert!(body.get("medium_size_operators").unwrap().is_number());
    assert!(body.get("large_size_operators").unwrap().is_number());
    assert!(body.get("small_operators").unwrap().is_array());
    assert!(body.get("medium_operators").unwrap().is_array());
    assert!(body.get("large_operators").unwrap().is_array());
}
