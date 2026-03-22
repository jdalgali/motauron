use crate::application::use_cases::track_market::TrackMarketUseCase;
use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct ApiState {
    pub use_case: Arc<TrackMarketUseCase>,
    pub json_path: String,
}

pub async fn serve(state: ApiState, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/api/listings", get(handle_listings))
        .route("/api/scrape", post(handle_scrape))
        .with_state(state)
        .layer(cors);

    let addr = format!("0.0.0.0:{}", port);
    println!("motauron API listening on http://localhost:{}", port);
    println!("  GET  /api/listings  — serve current listings from disk");
    println!("  POST /api/scrape    — trigger a fresh scrape");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_listings(State(state): State<ApiState>) -> Response {
    match tokio::fs::read_to_string(&state.json_path).await {
        Ok(data) => (
            StatusCode::OK,
            [("Content-Type", "application/json")],
            data,
        )
            .into_response(),
        Err(_) => (
            StatusCode::OK,
            [("Content-Type", "application/json")],
            "[]".to_string(),
        )
            .into_response(),
    }
}

async fn handle_scrape(State(state): State<ApiState>) -> StatusCode {
    match state.use_case.execute().await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("scrape error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
