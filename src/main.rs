use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use std::net::SocketAddr;
use tracing::{event, Level};

#[cfg(feature = "jemalloc")]
mod allocator {
    #[cfg(not(target_env = "msvc"))]
    use tikv_jemallocator::Jemalloc;

    #[cfg(not(target_env = "msvc"))]
    #[global_allocator]
    static GLOBAL: Jemalloc = Jemalloc;
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/widgets", post(create_widget));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    event!(Level::DEBUG, "listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("server run");
}

#[derive(serde::Deserialize, Clone)]
struct Widget {
    payload: serde_json::Value,
}

#[derive(serde::Serialize)]
struct WidgetCreateResponse {
    id: String,
    size: usize,
}

async fn create_widget(Json(widget): Json<Widget>) -> Response {
    (
        StatusCode::CREATED,
        Json(process_widget(widget.clone()).await),
    )
        .into_response()
}

async fn process_widget(widget: Widget) -> WidgetCreateResponse {
    let widget_id = uuid::Uuid::new_v4();
    let bytes = serde_json::to_vec(&widget.payload).unwrap_or_default();
    // An arbitrary sleep to pad the handler latency as a stand-in for a more
    // complex code path.
    // Tweak the duration by setting the `SLEEP_MS` env var.
    tokio::time::sleep(std::time::Duration::from_millis(
        std::env::var("SLEEP_MS")
            .as_deref()
            .unwrap_or("150")
            .parse()
            .expect("invalid SLEEP_MS"),
    ))
    .await;
    WidgetCreateResponse {
        id: widget_id.to_string(),
        size: bytes.len(),
    }
}
