#[tokio::main]
async fn main() {
    let token = std::env::args()
        .skip_while(|v| v != "--token")
        .nth(1)
        .unwrap_or_else(|| nexa_g1_loopback_spike::DEFAULT_TOKEN.into());
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 43116))
        .await
        .expect("loopback fixture bind");
    println!("G1 fixture listening on http://127.0.0.1:43116");
    axum::serve(listener, nexa_g1_loopback_spike::app(&token))
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await
        .unwrap()
}
