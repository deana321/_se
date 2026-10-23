use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;

use crate::{
    logic::{remaining_budget, validate_amount},
    models::{Transaction, TransactionInput, EXPENSE_CATEGORIES},
    routes::{bad_request, created, today_str, valid_date},
};

#[derive(Debug, Deserialize, Default)]
pub struct TxFilter {
    pub start: Option<String>,
    pub end: Option<String>,
    pub category: Option<String>,
    pub q: Option<String>,
    pub kind: Option<String>,
}

pub async fn list(
    State(pool): State<SqlitePool>,
    Query(f): Query<TxFilter>,
) -> Result<Json<Vec<Transaction>>, axum::response::Response> {
    let rows = sqlx::query_as::<_, Transaction>(
        "SELECT id, kind, name, amount, category, date, note FROM transactions ORDER BY date DESC, id DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let out: Vec<Transaction> = rows
        .into_iter()
        .filter(|t| {
            if let Some(ref s) = f.start {
                if t.date < *s {
                    return false;
                }
            }
            if let Some(ref e) = f.end {
                if t.date > *e {
                    return false;
                }
            }
            if let Some(ref c) = f.category {
                if !c.is_empty() && t.category != *c {
                    return false;
                }
            }
            if let Some(ref k) = f.kind {
                if !k.is_empty() && t.kind != *k {
                    return false;
                }
            }
            if let Some(ref q) = f.q {
                if !q.is_empty() && !(t.name.contains(q) || t.note.contains(q)) {
                    return false;
                }
            }
            true
        })
        .collect();
    Ok(Json(out))
}

fn validate_input(input: &TransactionInput) -> Result<(), String> {
    if input.kind != "income" && input.kind != "expense" {
        return Err("kind 必須是 income 或 expense".into());
    }
    if input.name.trim().is_empty() {
        return Err("項目名稱不可空白".into());
    }
    validate_amount(input.amount)?;
    if input.category.trim().is_empty() {
        return Err("分類不可空白".into());
    }
    if input.kind == "expense"
        && !EXPENSE_CATEGORIES.contains(&input.category.as_str())
    {
        return Err("支出分類不合法".into());
    }
    if !valid_date(&input.date) {
        return Err("日期格式須為 YYYY-MM-DD".into());
    }
    Ok(())
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(input): Json<TransactionInput>,
) -> Result<impl axum::response::IntoResponse, axum::response::Response> {
    validate_input(&input).map_err(|e| bad_request(e).into_response())?;
    let note = input.note.clone().unwrap_or_default();
    let r = sqlx::query(
        "INSERT INTO transactions (kind, name, amount, category, date, note) VALUES (?,?,?,?,?,?)",
    )
    .bind(&input.kind)
    .bind(input.name.trim())
    .bind(input.amount)
    .bind(&input.category)
    .bind(&input.date)
    .bind(&note)
    .execute(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let id = r.last_insert_rowid();
    let row = sqlx::query_as::<_, Transaction>(
        "SELECT id, kind, name, amount, category, date, note FROM transactions WHERE id = ?",
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
    Json(input): Json<TransactionInput>,
) -> Result<Json<Transaction>, axum::response::Response> {
    validate_input(&input).map_err(|e| bad_request(e).into_response())?;
    let note = input.note.clone().unwrap_or_default();
    let r = sqlx::query(
        "UPDATE transactions SET kind=?, name=?, amount=?, category=?, date=?, note=? WHERE id=?",
    )
    .bind(&input.kind)
    .bind(input.name.trim())
    .bind(input.amount)
    .bind(&input.category)
    .bind(&input.date)
    .bind(&note)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    if r.rows_affected() == 0 {
        return Err(super::not_found("找不到該筆紀錄").into_response());
    }
    let row = sqlx::query_as::<_, Transaction>(
        "SELECT id, kind, name, amount, category, date, note FROM transactions WHERE id = ?",
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
) -> Result<impl axum::response::IntoResponse, axum::response::Response> {
    let r = sqlx::query("DELETE FROM transactions WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
    if r.rows_affected() == 0 {
        return Err(super::not_found("找不到該筆紀錄").into_response());
    }
    Ok((StatusCode::OK, Json(json!({ "ok": true, "id": id }))))
}

pub async fn get_budget(
    State(pool): State<SqlitePool>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    let b: i64 = sqlx::query_scalar("SELECT monthly_budget FROM budget WHERE id = 1")
        .fetch_one(&pool)
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
    Ok(Json(json!({ "monthly_budget": b })))
}

#[derive(Debug, serde::Deserialize)]
pub struct BudgetInput {
    pub monthly_budget: i64,
}

pub async fn put_budget(
    State(pool): State<SqlitePool>,
    Json(input): Json<BudgetInput>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    if input.monthly_budget < 0 {
        return Err(bad_request("預算不得為負數".into()).into_response());
    }
    sqlx::query("UPDATE budget SET monthly_budget = ? WHERE id = 1")
        .bind(input.monthly_budget)
        .execute(&pool)
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
    Ok(Json(json!({ "monthly_budget": input.monthly_budget })))
}

pub async fn statistics(
    State(pool): State<SqlitePool>,
    Query(q): Query<TxFilter>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    let today = today_str();
    let month = q
        .start
        .clone()
        .map(|s| s.get(0..7).unwrap_or(&today[0..7]).to_string())
        .unwrap_or_else(|| today.get(0..7).unwrap_or("2026-10").to_string());
    let rows = sqlx::query_as::<_, Transaction>(
        "SELECT id, kind, name, amount, category, date, note FROM transactions WHERE substr(date,1,7) = ? ORDER BY date ASC",
    )
    .bind(&month)
    .fetch_all(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let mut income: i64 = 0;
    let mut expense: i64 = 0;
    let mut by_cat = std::collections::HashMap::new();
    let mut by_day = std::collections::HashMap::new();
    for t in &rows {
        if t.kind == "income" {
            income += t.amount;
        } else {
            expense += t.amount;
            *by_cat.entry(t.category.clone()).or_insert(0i64) += t.amount;
        }
        if t.kind == "expense" {
            *by_day.entry(t.date.clone()).or_insert(0i64) += t.amount;
        }
    }
    let budget: i64 = sqlx::query_scalar("SELECT monthly_budget FROM budget WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap_or(20000);
    let remaining = remaining_budget(budget, expense);
    let usage = if budget > 0 {
        expense as f64 / budget as f64
    } else {
        0.0
    };
    Ok(Json(json!({
        "month": month,
        "total_income": income,
        "total_expense": expense,
        "remaining": remaining,
        "budget": budget,
        "usage_rate": usage,
        "over80": usage >= 0.8 && usage < 1.0,
        "over100": usage >= 1.0,
        "by_category": by_cat,
        "by_day": by_day,
    })))
}

pub async fn dashboard(
    State(pool): State<SqlitePool>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    let today = today_str();
    let month = today.get(0..7).unwrap_or("2026-10").to_string();
    let expense: Option<i64> = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount),0) FROM transactions WHERE kind='expense' AND substr(date,1,7)=?",
    )
    .bind(&month)
    .fetch_one(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let expense = expense.unwrap_or(0);
    let budget: i64 = sqlx::query_scalar("SELECT monthly_budget FROM budget WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap_or(20000);
    let ing_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM ingredients WHERE quantity > 0")
            .fetch_one(&pool)
            .await
            .unwrap_or(0);
    let soon_rows = sqlx::query_as::<_, crate::models::Ingredient>(
        "SELECT id, name, category, quantity, unit, purchase_date, expiry_date, note FROM ingredients WHERE quantity > 0",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    let today_d = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap();
    let mut soon = 0;
    let mut soon_list = vec![];
    for ing in soon_rows {
        if let Ok(exp) = chrono::NaiveDate::parse_from_str(&ing.expiry_date, "%Y-%m-%d") {
            let st = crate::logic::expiry_status(exp, today_d);
            if st != crate::logic::ExpiryStatus::Normal {
                soon += 1;
                soon_list.push(ing);
            }
        }
    }
    // 今日菜單
    let plans = sqlx::query_as::<_, crate::models::MealPlan>(
        "SELECT id, date, meal_type, recipe_id, is_eating_out, note FROM meal_plans WHERE date = ?",
    )
    .bind(&today)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    Ok(Json(json!({
        "today": today,
        "month_expense": expense,
        "remaining": remaining_budget(budget, expense),
        "budget": budget,
        "ingredient_count": ing_count,
        "expiring_count": soon,
        "expiring_list": soon_list,
        "today_plans": plans,
    })))
}
