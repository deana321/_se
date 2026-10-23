use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::models::ApiError;

pub mod fridge;
pub mod plans;
pub mod recipes;
pub mod transactions;

pub fn err(status: StatusCode, msg: impl Into<String>) -> impl IntoResponse {
    let body = Json(ApiError { error: msg.into() });
    (status, body).into_response()
}

pub fn ok_json(v: serde_json::Value) -> impl IntoResponse {
    (StatusCode::OK, Json(v)).into_response()
}

pub fn not_found(msg: impl Into<String>) -> impl IntoResponse {
    err(StatusCode::NOT_FOUND, msg.into())
}

pub fn bad_request(msg: String) -> impl IntoResponse {
    err(StatusCode::BAD_REQUEST, msg)
}

pub fn today_str() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

pub fn month_prefix(date_or_today: &str) -> String {
    date_or_today.get(0..7).unwrap_or(date_or_today).to_string()
}

pub fn json_err(msg: String) -> axum::response::Response {
    err(StatusCode::BAD_REQUEST, msg).into_response()
}

/// 統一驗證日期 YYYY-MM-DD
pub fn valid_date(s: &str) -> bool {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
}

pub fn created(v: serde_json::Value) -> impl IntoResponse {
    (StatusCode::CREATED, Json(v)).into_response()
}

pub fn _unused() {
    let _ = json!({});
}
