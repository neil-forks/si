use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::{http, Router};
use dal::test_harness::{one_time_setup, TestContext};
use sdf::service::signup;
use sdf::service::signup::create_account::CreateAccountResponse;
use sdf::JwtSigningKey;
use tower::ServiceExt;

#[tokio::test]
async fn create_account() {
    one_time_setup().await.expect("cannot setup tests");
    let ctx = TestContext::init().await;
    let (pg, nats, secret_key) = ctx.entries();
    let telemetry = ctx.telemetry();
    let (app, _) = sdf::build_service(
        telemetry,
        pg.clone(),
        nats.clone(),
        JwtSigningKey {
            key: secret_key.clone(),
        },
    )
    .expect("cannot build new server");
    let app: Router = app.into();

    let request = signup::create_account::CreateAccountRequest {
        billing_account_name: "witness".to_string(),
        user_name: "bobo".to_string(),
        user_email: "bobo@tclown.org".to_string(),
        user_password: "bobor7les".to_string(),
    };
    let api_request = Request::builder()
        .method(Method::POST)
        .uri("/api/signup/create_account")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!(&request)).expect("cannot turn request to json"),
        ))
        .expect("cannot create api request");
    let response = app.oneshot(api_request).await.expect("cannot send request");
    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_json: serde_json::Value =
        serde_json::from_slice(&body).expect("response is not valid json");
    let response: CreateAccountResponse =
        serde_json::from_value(body_json).expect("response is not a valid rust struct");
    assert_eq!(response.success, true);
}