use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
};
use tower::ServiceExt;

async fn websocket_request(origin: &str, token: &str, version: &str) -> (StatusCode, Vec<String>) {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, nexa_g1_loopback_spike::app("test"))
            .await
            .unwrap();
    });
    let mut request = format!("ws://{address}/v1/events?token={token}&version={version}")
        .into_client_request()
        .unwrap();
    request
        .headers_mut()
        .insert("Origin", origin.parse().unwrap());
    let result = connect_async(request).await;
    let output = match result {
        Ok((mut socket, response)) => {
            let mut replies = vec![socket
                .next()
                .await
                .unwrap()
                .unwrap()
                .into_text()
                .unwrap()
                .to_string()];
            for input in [r#"{"type":"cancel","request_id":"fixture"}"#, "not-json"] {
                socket.send(Message::Text(input.into())).await.unwrap();
                replies.push(
                    socket
                        .next()
                        .await
                        .unwrap()
                        .unwrap()
                        .into_text()
                        .unwrap()
                        .to_string(),
                );
            }
            socket
                .send(Message::Text("x".repeat(257).into()))
                .await
                .unwrap();
            replies.push(
                socket
                    .next()
                    .await
                    .unwrap()
                    .unwrap()
                    .into_text()
                    .unwrap()
                    .to_string(),
            );
            (response.status(), replies)
        }
        Err(tokio_tungstenite::tungstenite::Error::Http(response)) => {
            (response.status(), Vec::new())
        }
        Err(error) => panic!("unexpected websocket result: {error}"),
    };
    server.abort();
    output
}

#[tokio::test]
async fn enforces_http_authorization_and_version() {
    let app = nexa_g1_loopback_spike::app("test");
    for (origin, authorization, version, expected) in [
        (
            "http://tauri.localhost",
            None,
            None,
            StatusCode::UNAUTHORIZED,
        ),
        (
            "http://tauri.localhost",
            Some("Bearer test"),
            Some("2"),
            StatusCode::UPGRADE_REQUIRED,
        ),
        (
            "https://attacker.example",
            Some("Bearer test"),
            Some("1"),
            StatusCode::FORBIDDEN,
        ),
        (
            "http://tauri.localhost",
            Some("Bearer test"),
            Some("1"),
            StatusCode::OK,
        ),
    ] {
        let mut request = Request::builder()
            .uri("/v1/fixture")
            .header("origin", origin);
        if let Some(value) = authorization {
            request = request.header("authorization", value);
        }
        if let Some(value) = version {
            request = request.header("nexa-protocol-version", value);
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }
}

#[tokio::test]
async fn accepts_windows_origin_and_handles_cancel_and_untrusted_messages() {
    let (status, replies) = websocket_request("http://tauri.localhost", "test", "1").await;
    assert_eq!(status, StatusCode::SWITCHING_PROTOCOLS);
    assert_eq!(
        replies,
        [
            "Event connection ready.",
            "Cancellation acknowledged.",
            "Malformed event ignored.",
            "Malformed event ignored."
        ]
    );
}

#[tokio::test]
async fn rejects_wrong_websocket_origin_token_and_version() {
    for (origin, token, version) in [
        ("https://attacker.example", "test", "1"),
        ("http://tauri.localhost", "wrong", "1"),
        ("http://tauri.localhost", "test", "2"),
    ] {
        let (status, replies) = websocket_request(origin, token, version).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(replies.is_empty());
    }
}
