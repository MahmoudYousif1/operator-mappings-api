use dotenv::dotenv;
use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::env;

static BASE_URL: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("API_TEST_BASE_URL").unwrap_or_else(|_| "http://localhost:8080/api/v1".into())
});

fn client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap()
}

#[tokio::test]
async fn test_group_by_iso3_success() {
    let resp = client()
        .get(format!("{}/operators/by-countries-operators", &*BASE_URL))
        .query(&[("iso3", "arg")])
        .send()
        .await
        .expect("GET /operators/by-countries-operators failed");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.expect("Invalid JSON");

    let iso3 = body["iso3"].as_str().unwrap().to_uppercase();
    let total = body["total"].as_u64().unwrap() as usize;
    let ops = body["operators"].as_array().unwrap();

    assert_eq!(iso3, "ARG");
    assert_eq!(ops.len(), total, "total should match number of operators");
    assert!(total > 0, "expected at least one operator for ARG");
    assert_eq!(ops[0]["iso3"].as_str().unwrap().to_uppercase(), "ARG");
}

#[tokio::test]
async fn test_group_by_iso3_not_found() {
    let resp = client()
        .get(format!("{}/operators/by-countries-operators", &*BASE_URL))
        .query(&[("iso3", "xxx")])
        .send()
        .await
        .expect("GET /operators/by-countries-operators failed");

    match resp.status() {
        StatusCode::NOT_FOUND => {}
        StatusCode::OK => {
            let body: Value = resp.json().await.expect("Invalid JSON");
            let iso3 = body["iso3"].as_str().unwrap().to_uppercase();
            let total = body["total"].as_u64().unwrap() as usize;
            let ops = body["operators"].as_array().unwrap();

            assert_eq!(iso3, "XXX");
            assert_eq!(total, 1, "expected total == 1 for fallback operator");
            assert_eq!(ops.len(), 1, "expected one fallback operator");
            assert_eq!(
                ops[0]["country"].as_str().unwrap(),
                "Unknown",
                "fallback country must be 'Unknown'"
            );
            assert_eq!(
                ops[0]["iso3"].as_str().unwrap().to_uppercase(),
                "XXX",
                "fallback iso3 must match requested"
            );
        }
        other => panic!("unexpected status code: {}", other),
    }
}
