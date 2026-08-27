use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
pub const VERSION: &str = "1";
pub const DEFAULT_TOKEN: &str = "g1-local-fixture";
const ORIGINS: [&str; 2] = ["http://127.0.0.1:4173", "tauri://localhost"];
#[derive(Clone)]
pub struct AppState {
    token: Arc<str>,
}
#[derive(Serialize)]
struct Fixture<'a> {
    message: &'a str,
    protocol_version: &'a str,
}
#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
}
#[derive(Deserialize)]
struct WsQuery {
    token: String,
    version: String,
}
pub fn app(token: &str) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(ORIGINS.map(|v| v.parse::<HeaderValue>().unwrap()))
        .allow_methods([Method::GET])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            "nexa-protocol-version".parse().unwrap(),
        ]);
    Router::new()
        .route("/v1/fixture", get(fixture))
        .route("/v1/events", get(events))
        .with_state(AppState {
            token: Arc::from(token),
        })
        .layer(cors)
}
fn authorized(headers: &HeaderMap, state: &AppState) -> Result<(), Response> {
    let bearer = format!("Bearer {}", state.token);
    if headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        != Some(&bearer)
    {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorBody {
                error: "authorization rejected",
            }),
        )
            .into_response());
    }
    if headers
        .get("nexa-protocol-version")
        .and_then(|v| v.to_str().ok())
        != Some(VERSION)
    {
        return Err((
            StatusCode::UPGRADE_REQUIRED,
            Json(ErrorBody {
                error: "unsupported protocol version",
            }),
        )
            .into_response());
    }
    Ok(())
}
async fn fixture(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Fixture<'static>>, Response> {
    authorized(&headers, &state)?;
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    Ok(Json(Fixture {
        message: "Deterministic fixture complete.",
        protocol_version: VERSION,
    }))
}
async fn events(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WsQuery>,
) -> Response {
    let origin_ok = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| ORIGINS.contains(&v));
    if !origin_ok || query.token != state.token.as_ref() || query.version != VERSION {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorBody {
                error: "websocket origin, token, or version rejected",
            }),
        )
            .into_response();
    }
    ws.on_upgrade(socket)
}
async fn socket(mut socket: WebSocket) {
    if socket
        .send(Message::Text("Event connection ready.".into()))
        .await
        .is_err()
    {
        return;
    }
    while let Some(Ok(message)) = socket.recv().await {
        match message {
            Message::Text(text) if text.len() <= 256 => {
                let reply = if serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|v| v.get("type").and_then(|v| v.as_str()).map(str::to_owned))
                    .as_deref()
                    == Some("cancel")
                {
                    "Cancellation acknowledged."
                } else {
                    "Malformed event ignored."
                };
                if socket.send(Message::Text(reply.into())).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => break,
            _ => {
                let _ = socket
                    .send(Message::Text("Malformed event ignored.".into()))
                    .await;
            }
        }
    }
}
