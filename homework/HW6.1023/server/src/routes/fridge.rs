use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;

use crate::{
    logic::{expiry_status, validate_quantity},
    models::{Ingredient, IngredientInput, INGREDIENT_UNITS},
    routes::{bad_request, created, valid_date},
};

#[derive(Debug, Deserialize, Default)]
pub struct IngFilter {
    pub q: Option<String>,
    pub category: Option<String>,
}

pub async fn list(
    State(pool): State<SqlitePool>,
    Query(f): Query<IngFilter>,
) -> Result<Json<Vec<serde_json::Value>>, axum::response::Response> {
    let rows = sqlx::query_as::<_, Ingredient>(
        "SELECT id, name, category, quantity, unit, purchase_date, expiry_date, note FROM ingredients ORDER BY expiry_date ASC, id DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let today = crate::routes::today_str();
    let today_d = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap();
    let mut out = vec![];
    for r in rows {
        if let Some(ref q) = f.q {
            if !q.is_empty() && !r.name.contains(q) {
                continue;
            }
        }
        if let Some(ref c) = f.category {
            if !c.is_empty() && r.category != *c {
                continue;
            }
        }
        let status = chrono::NaiveDate::parse_from_str(&r.expiry_date, "%Y-%m-%d")
            .map(|exp| expiry_status(exp, today_d))
            .unwrap_or(crate::logic::ExpiryStatus::Normal);
        out.push(json!({
            "id": r.id, "name": r.name, "category": r.category,
            "quantity": r.quantity, "unit": r.unit,
            "purchase_date": r.purchase_date, "expiry_date": r.expiry_date,
            "note": r.note, "status": status,
        }));
    }
    Ok(Json(out))
}

fn validate_input(input: &IngredientInput) -> Result<(), String> {
    if input.name.trim().is_empty() {
        return Err("食材名稱不可空白".into());
    }
    validate_quantity(input.quantity, true)?;
    if !INGREDIENT_UNITS.contains(&input.unit.as_str()) {
        return Err("單位不合法".into());
    }
    if !valid_date(&input.purchase_date) || !valid_date(&input.expiry_date) {
        return Err("日期格式須為 YYYY-MM-DD".into());
    }
    Ok(())
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(input): Json<IngredientInput>,
) -> Result<impl axum::response::IntoResponse, axum::response::Response> {
    validate_input(&input).map_err(|e| bad_request(e).into_response())?;
    let r = sqlx::query(
        "INSERT INTO ingredients (name, category, quantity, unit, purchase_date, expiry_date, note) VALUES (?,?,?,?,?,?,?)",
    )
    .bind(input.name.trim())
    .bind(input.category.clone().unwrap_or("其他".into()))
    .bind(input.quantity)
    .bind(&input.unit)
    .bind(&input.purchase_date)
    .bind(&input.expiry_date)
    .bind(input.note.clone().unwrap_or_default())
    .execute(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let id = r.last_insert_rowid();
    let row = sqlx::query_as::<_, Ingredient>(
        "SELECT id, name, category, quantity, unit, purchase_date, expiry_date, note FROM ingredients WHERE id=?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    Ok(created(json!(row)))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(input): Json<IngredientInput>,
) -> Result<Json<Ingredient>, axum::response::Response> {
    validate_input(&input).map_err(|e| bad_request(e).into_response())?;
    let r = sqlx::query(
        "UPDATE ingredients SET name=?, category=?, quantity=?, unit=?, purchase_date=?, expiry_date=?, note=? WHERE id=?",
    )
    .bind(input.name.trim())
    .bind(input.category.clone().unwrap_or("其他".into()))
    .bind(input.quantity)
    .bind(&input.unit)
    .bind(&input.purchase_date)
    .bind(&input.expiry_date)
    .bind(input.note.clone().unwrap_or_default())
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    if r.rows_affected() == 0 {
        return Err(super::not_found("找不到該食材").into_response());
    }
    let row = sqlx::query_as::<_, Ingredient>(
        "SELECT id, name, category, quantity, unit, purchase_date, expiry_date, note FROM ingredients WHERE id=?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    Ok(Json(row))
}

pub async fn remove(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    let r = sqlx::query("DELETE FROM ingredients WHERE id=?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
    if r.rows_affected() == 0 {
        return Err(super::not_found("找不到該食材").into_response());
    }
    Ok(Json(json!({ "ok": true, "id": id })))
}

#[derive(Debug, serde::Deserialize)]
pub struct AdjustInput {
    pub delta: f64,
}

pub async fn adjust(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(input): Json<AdjustInput>,
) -> Result<Json<Ingredient>, axum::response::Response> {
    let row = sqlx::query_as::<_, Ingredient>(
        "SELECT id, name, category, quantity, unit, purchase_date, expiry_date, note FROM ingredients WHERE id=?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let row = row.ok_or_else(|| super::not_found("找不到該食材").into_response())?;
    let next = row.quantity + input.delta;
    if next < 0.0 {
        return Err(bad_request("庫存不得小於零".into()).into_response());
    }
    sqlx::query("UPDATE ingredients SET quantity=? WHERE id=?")
        .bind(next)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
    let updated = sqlx::query_as::<_, Ingredient>(
        "SELECT id, name, category, quantity, unit, purchase_date, expiry_date, note FROM ingredients WHERE id=?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    Ok(Json(updated))
}
