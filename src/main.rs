// Fuzzy Navigation System API
// Powered by Shuttle and Axum
use shuttle_axum::axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use std::panic;

use examen_parcial::api::handlers;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // Set custom panic hook to avoid writing to stdout/stderr
    // This prevents "Broken pipe" errors when stdout is not available
    panic::set_hook(Box::new(|_panic_info| {
        // Silently ignore panics or log to a file/service instead
        // In production, you'd want to log this to a proper logging service
        let _ = std::fs::write("/tmp/fuzzy_nav_panic.log", format!("{:?}", _panic_info));
    }));
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router with all endpoints
    let router = Router::new()
        // Health check
        .route("/", get(handlers::health_check))
        .route("/health", get(handlers::health_check))

        // Simulation endpoints
        .route("/api/simulate", post(handlers::run_simulation))
        .route("/api/benchmark", post(handlers::run_benchmark))

        // Add middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    Ok(router.into())
}
