use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
#[tokio::test]
async fn rejects_missing_auth() {
    let r = nexa_g1_loopback_spike::app("test")
        .oneshot(
            Request::builder()
                .uri("/v1/fixture")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED)
}
#[tokio::test]
async fn rejects_wrong_version() {
    let r = nexa_g1_loopback_spike::app("test")
        .oneshot(
            Request::builder()
                .uri("/v1/fixture")
                .header("authorization", "Bearer test")
                .header("nexa-protocol-version", "2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::UPGRADE_REQUIRED)
}
#[tokio::test]
async fn accepts_authorized_fixture() {
    let r = nexa_g1_loopback_spike::app("test")
        .oneshot(
            Request::builder()
                .uri("/v1/fixture")
                .header("authorization", "Bearer test")
                .header("nexa-protocol-version", "1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK)
}
#[tokio::test]
async fn rejects_websocket_without_approved_origin() {
    let r = nexa_g1_loopback_spike::app("test")
        .oneshot(
            Request::builder()
                .uri("/v1/events?token=test&version=1")
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .header("sec-websocket-version", "13")
                .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED)
}
