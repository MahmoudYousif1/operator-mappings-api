use dotenv::dotenv;
use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
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
async fn test_lookup_by_imsi_returns_200() {
    let resp = client()
        .get(format!("{}/operators", &*BASE_URL))
        .query(&[("imsi", "3368900890017")])
        .send()
        .await
        .expect("GET /operators?imsi failed");
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_lookup_by_msisdn_returns_200() {
    let resp = client()
        .get(format!("{}/operators", &*BASE_URL))
        .query(&[("msisdn", "4077000")])
        .send()
        .await
        .expect("GET /operators?msisdn failed");
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_lookup_by_tadig_returns_200() {
    let resp = client()
        .get(format!("{}/operators", &*BASE_URL))
        .query(&[("tadig", "LBN_GEO")])
        .send()
        .await
        .expect("GET /operators?tadig failed");
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_lookup_invalid_msisdn_returns_400() {
    let resp = client()
        .get(format!("{}/operators", &*BASE_URL))
        .query(&[("msisdn", "frgr")])
        .send()
        .await
        .expect("GET /operators?msisdn=frgr failed");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
