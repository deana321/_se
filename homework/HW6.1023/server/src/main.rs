use axum::{
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};

mod db;
mod logic;
mod models;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://sololife.db".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);
    let pool = db::create_pool(&database_url).await?;
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        // transactions
        .route("/api/transactions", get(routes::transactions::list).post(routes::transactions::create))
        .route(
            "/api/transactions/:id",
            put(routes::transactions::update).delete(routes::transactions::remove),
        )
        .route("/api/budget", get(routes::transactions::get_budget).put(routes::transactions::put_budget))
        .route("/api/statistics", get(routes::transactions::statistics))
        .route("/api/dashboard", get(routes::transactions::dashboard))
        // fridge
        .route("/api/ingredients", get(routes::fridge::list).post(routes::fridge::create))
        .route(
            "/api/ingredients/:id",
            put(routes::fridge::update).delete(routes::fridge::remove),
        )
        .route("/api/ingredients/:id/adjust", post(routes::fridge::adjust))
        // recipes
        .route("/api/recipes", get(routes::recipes::list).post(routes::recipes::create))
        .route("/api/recipes/recommendations", get(routes::recipes::recommendations))
        .route(
            "/api/recipes/:id",
            get(routes::recipes::get_one)
                .put(routes::recipes::update)
                .delete(routes::recipes::remove),
        )
        .route("/api/cooking/complete", post(routes::recipes::complete_cooking))
        // meal plans
        .route("/api/meal-plans", get(routes::plans::list_plans).post(routes::plans::create_plan))
        .route(
            "/api/meal-plans/:id",
            put(routes::plans::update_plan).delete(routes::plans::delete_plan),
        )
        // shopping
        .route(
            "/api/shopping-list",
            get(routes::plans::list_shopping).post(routes::plans::create_shopping),
        )
        .route(
            "/api/shopping-list/:id",
            put(routes::plans::update_shopping).delete(routes::plans::delete_shopping),
        )
        .route("/api/shopping-list/generate", post(routes::plans::generate))
        .route("/api/purchases", post(routes::plans::complete_purchase))
        // test reports + test-only reset
        .route("/api/test-reports", get(test_reports))
        .route("/api/test/reset", post(test_reset))
        .layer(cors)
        .with_state(pool);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("SoloLife server on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn test_reports() -> axum::Json<serde_json::Value> {
    use std::path::Path;
    let mut out = serde_json::Map::new();
    for key in ["unit", "system", "e2e"] {
        let p = format!("reports/{key}.json");
        if Path::new(&p).exists() {
            match std::fs::read_to_string(&p) {
                Ok(s) => {
                    out.insert(
                        key.to_string(),
                        serde_json::from_str(&s).unwrap_or(serde_json::Value::Null),
                    );
                }
                Err(_) => {
                    out.insert(key.to_string(), serde_json::Value::Null);
                }
            }
        } else {
            out.insert(key.to_string(), serde_json::Value::Null);
        }
    }
    axum::Json(serde_json::Value::Object(out))
}

async fn test_reset(
    axum::extract::State(pool): axum::extract::State<SqlitePool>,
) -> impl axum::response::IntoResponse {
    if std::env::var("ALLOW_TEST_RESET").unwrap_or_default() != "1" {
        return (axum::http::StatusCode::FORBIDDEN, "not allowed").into_response();
    }
    for t in [
        "cooking_logs",
        "purchases",
        "shopping_items",
        "meal_plans",
        "transactions",
        "ingredients",
    ] {
        sqlx::query(&format!("DELETE FROM {t}"))
            .execute(&pool)
            .await
            .ok();
    }
    (axum::http::StatusCode::OK, axum::Json(serde_json::json!({"ok": true}))).into_response()
}
