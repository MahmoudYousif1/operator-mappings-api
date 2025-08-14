use dotenv::dotenv;
use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
use serde_json::json;
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
        "country": "Ireland",
        "iso2":    "IE",
        "iso3":    "IRL",
        "name":    "IrishCom",
        "e212":    ["27201"],
        "e164":    ["35385"],
        "tadig":   [tadig]
    });

    let resp = client()
        .post(format!("{}/operators", &*BASE_URL))
        .json(&body)
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_success() || resp.status() == StatusCode::CONFLICT,
        "{} setup failed: {}",
        tadig,
        resp.status()
    );
}

#[tokio::test]
async fn test_delete_operator_success() {
    ensure_operator_exists("IRLEC").await;

    let resp = client()
        .delete(format!("{}/operators/IRLEC", &*BASE_URL))
        .send()
        .await
        .unwrap();

    println!("DELETE /operators/IRLEC (1st attempt): {}", resp.status());
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::NO_CONTENT,
        "expected OK or NO_CONTENT on first delete, got {}",
        resp.status()
    );
}

#[tokio::test]
async fn test_delete_operator_twice_returns_not_found() {
    ensure_operator_exists("IRLEC").await;

    client()
        .delete(format!("{}/operators/IRLEC", &*BASE_URL))
        .send()
        .await
        .unwrap();

    let resp2 = client()
        .delete(format!("{}/operators/IRLEC", &*BASE_URL))
        .send()
        .await
        .unwrap();

    println!("DELETE /operators/IRLEC (2nd attempt): {}", resp2.status());
    assert_eq!(resp2.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_operator_not_found() {
    let resp = client()
        .delete(format!("{}/operators/DOES_NOT_EXIST", &*BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
