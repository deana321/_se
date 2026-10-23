use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;

use crate::{
    logic::{expiry_status, match_recipe, Need, StockLine},
    models::{CookingInput, Recipe, RecipeInput, RecipeIngredient, RecipeRow},
    routes::{bad_request, created, today_str},
};

fn row_to_recipe(r: RecipeRow) -> Recipe {
    let ingredients: Vec<RecipeIngredient> =
        serde_json::from_str(&r.ingredients_json).unwrap_or_default();
    let steps: Vec<String> = serde_json::from_str(&r.steps_json).unwrap_or_default();
    Recipe {
        id: r.id,
        name: r.name,
        category: r.category,
        time_minutes: r.time_minutes,
        difficulty: r.difficulty,
        servings: r.servings,
        ingredients,
        steps,
        note: r.note,
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct RecipeFilter {
    pub q: Option<String>,
}

pub async fn list(
    State(pool): State<SqlitePool>,
    Query(f): Query<RecipeFilter>,
) -> Result<Json<Vec<Recipe>>, axum::response::Response> {
    let rows = sqlx::query_as::<_, RecipeRow>(
        "SELECT id, name, category, time_minutes, difficulty, servings, ingredients_json, steps_json, note FROM recipes ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let mut out: Vec<Recipe> = rows.into_iter().map(row_to_recipe).collect();
    if let Some(q) = f.q {
        if !q.is_empty() {
            out.retain(|r| r.name.contains(&q));
        }
    }
    Ok(Json(out))
}

pub async fn get_one(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Recipe>, axum::response::Response> {
    let row = sqlx::query_as::<_, RecipeRow>(
        "SELECT id, name, category, time_minutes, difficulty, servings, ingredients_json, steps_json, note FROM recipes WHERE id=?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    match row {
        Some(r) => Ok(Json(row_to_recipe(r))),
        None => Err(super::not_found("找不到該食譜").into_response()),
    }
}

fn validate_recipe(input: &RecipeInput) -> Result<(), String> {
    if input.name.trim().is_empty() {
        return Err("料理名稱不可空白".into());
    }
    if input.ingredients.is_empty() {
        return Err("至少需要一項食材".into());
    }
    for ing in &input.ingredients {
        if ing.name.trim().is_empty() {
            return Err("食材名稱不可空白".into());
        }
        if ing.quantity <= 0.0 {
            return Err(format!("食材 {} 數量必須大於零", ing.name));
        }
        if !crate::models::INGREDIENT_UNITS.contains(&ing.unit.as_str()) {
            return Err(format!("食材 {} 單位不合法", ing.name));
        }
    }
    if input.steps.is_empty() {
        return Err("至少需要一個步驟".into());
    }
    Ok(())
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(input): Json<RecipeInput>,
) -> Result<impl axum::response::IntoResponse, axum::response::Response> {
    validate_recipe(&input).map_err(|e| bad_request(e).into_response())?;
    let ing_json = serde_json::to_string(&input.ingredients).unwrap();
    let steps_json = serde_json::to_string(&input.steps).unwrap();
    let r = sqlx::query(
        "INSERT INTO recipes (name, category, time_minutes, difficulty, servings, ingredients_json, steps_json, note) VALUES (?,?,?,?,?,?,?,?)",
    )
    .bind(input.name.trim())
    .bind(input.category.clone().unwrap_or("其他".into()))
    .bind(input.time_minutes.unwrap_or(15))
    .bind(input.difficulty.clone().unwrap_or("簡單".into()))
    .bind(input.servings.unwrap_or(1))
    .bind(&ing_json)
    .bind(&steps_json)
    .bind(input.note.clone().unwrap_or_default())
    .execute(&pool)
    .await
    .map_err(|e| bad_request(format!("新增食譜失敗：{e}")).into_response())?;
    let id = r.last_insert_rowid();
    let row = sqlx::query_as::<_, RecipeRow>(
        "SELECT id, name, category, time_minutes, difficulty, servings, ingredients_json, steps_json, note FROM recipes WHERE id=?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    Ok(created(json!(row_to_recipe(row))))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(input): Json<RecipeInput>,
) -> Result<Json<Recipe>, axum::response::Response> {
    validate_recipe(&input).map_err(|e| bad_request(e).into_response())?;
    let ing_json = serde_json::to_string(&input.ingredients).unwrap();
    let steps_json = serde_json::to_string(&input.steps).unwrap();
    let r = sqlx::query(
        "UPDATE recipes SET name=?, category=?, time_minutes=?, difficulty=?, servings=?, ingredients_json=?, steps_json=?, note=? WHERE id=?",
    )
    .bind(input.name.trim())
    .bind(input.category.clone().unwrap_or("其他".into()))
    .bind(input.time_minutes.unwrap_or(15))
    .bind(input.difficulty.clone().unwrap_or("簡單".into()))
    .bind(input.servings.unwrap_or(1))
    .bind(&ing_json)
    .bind(&steps_json)
    .bind(input.note.clone().unwrap_or_default())
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    if r.rows_affected() == 0 {
        return Err(super::not_found("找不到該食譜").into_response());
    }
    get_one(State(pool), Path(id)).await
}

pub async fn remove(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    let r = sqlx::query("DELETE FROM recipes WHERE id=?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
    if r.rows_affected() == 0 {
        return Err(super::not_found("找不到該食譜").into_response());
    }
    Ok(Json(json!({ "ok": true, "id": id })))
}

async fn load_stocks(pool: &SqlitePool) -> Vec<StockLine> {
    let rows = sqlx::query_as::<_, crate::models::Ingredient>(
        "SELECT id, name, category, quantity, unit, purchase_date, expiry_date, note FROM ingredients WHERE quantity > 0",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let today = today_str();
    let today_d = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap();
    rows.into_iter()
        .map(|r| {
            let expired = chrono::NaiveDate::parse_from_str(&r.expiry_date, "%Y-%m-%d")
                .map(|exp| expiry_status(exp, today_d) == crate::logic::ExpiryStatus::Expired)
                .unwrap_or(false);
            let soon = chrono::NaiveDate::parse_from_str(&r.expiry_date, "%Y-%m-%d")
                .map(|exp| expiry_status(exp, today_d) == crate::logic::ExpiryStatus::Soon)
                .unwrap_or(false);
            let _ = soon;
            StockLine { name: r.name, quantity: r.quantity, unit: r.unit, expired }
        })
        .collect()
}

fn uses_soon_ingredient(recipe: &Recipe, stocks: &[crate::models::Ingredient]) -> bool {
    let today = today_str();
    let today_d = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap();
    for req in &recipe.ingredients {
        for s in stocks {
            if s.name == req.name && s.unit == req.unit {
                if let Ok(exp) = chrono::NaiveDate::parse_from_str(&s.expiry_date, "%Y-%m-%d") {
                    if expiry_status(exp, today_d) == crate::logic::ExpiryStatus::Soon {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub async fn recommendations(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<serde_json::Value>>, axum::response::Response> {
    let rows = sqlx::query_as::<_, RecipeRow>(
        "SELECT id, name, category, time_minutes, difficulty, servings, ingredients_json, steps_json, note FROM recipes ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let recipes: Vec<Recipe> = rows.into_iter().map(row_to_recipe).collect();
    let raw_stocks = sqlx::query_as::<_, crate::models::Ingredient>(
        "SELECT id, name, category, quantity, unit, purchase_date, expiry_date, note FROM ingredients WHERE quantity > 0",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    let stocks = load_stocks(&pool).await;
    let mut out = vec![];
    for r in recipes {
        let required: Vec<Need> = r
            .ingredients
            .iter()
            .map(|i| Need { name: i.name.clone(), unit: i.unit.clone(), quantity: i.quantity })
            .collect();
        let (can_make, shortages) = match_recipe(&required, &stocks);
        let uses_soon = uses_soon_ingredient(&r, &raw_stocks);
        let rank = if can_make && uses_soon {
            0
        } else if can_make {
            1
        } else {
            2
        };
        out.push(json!({
            "recipe": r,
            "can_make": can_make,
            "uses_soon_expiring": uses_soon,
            "shortages": shortages,
            "missing_count": shortages.len(),
            "rank": rank,
        }));
    }
    out.sort_by_key(|v| v["rank"].as_i64().unwrap_or(9));
    Ok(Json(out))
}

/// POST /api/cooking/complete — Transaction 扣除庫存
pub async fn complete_cooking(
    State(pool): State<SqlitePool>,
    Json(input): Json<CookingInput>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    let recipe_row = sqlx::query_as::<_, RecipeRow>(
        "SELECT id, name, category, time_minutes, difficulty, servings, ingredients_json, steps_json, note FROM recipes WHERE id=?",
    )
    .bind(input.recipe_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let recipe_row = recipe_row.ok_or_else(|| super::not_found("找不到該食譜").into_response())?;
    let recipe = row_to_recipe(recipe_row);
    let servings = input.servings.unwrap_or(1).max(1) as f64;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;

    // 檢查並扣除：優先使用較早到期、未過期批次
    for req in &recipe.ingredients {
        let need_qty = req.quantity * servings;
        // 找出該食材所有未過期批次（FIFO by expiry）
        let today = today_str();
        let batches = sqlx::query_as::<_, crate::models::Ingredient>(
            "SELECT id, name, category, quantity, unit, purchase_date, expiry_date, note FROM ingredients WHERE name=? AND unit=? AND quantity > 0 AND expiry_date >= ? ORDER BY expiry_date ASC, id ASC",
        )
        .bind(&req.name)
        .bind(&req.unit)
        .bind(&today)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
        let total: f64 = batches.iter().map(|b| b.quantity).sum();
        if total < need_qty {
            tx.rollback().await.ok();
            return Err(bad_request(format!(
                "食材不足：{} 需要 {need_qty}{}，現有 {total}{}",
                req.name, req.unit, req.unit
            ))
            .into_response());
        }
        let mut left_need = need_qty;
        for b in batches {
            if left_need <= 0.0 {
                break;
            }
            let take = left_need.min(b.quantity);
            let new_qty = b.quantity - take;
            sqlx::query("UPDATE ingredients SET quantity=? WHERE id=?")
                .bind(new_qty)
                .bind(b.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| bad_request(e.to_string()).into_response())?;
            left_need -= take;
        }
    }
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    sqlx::query("INSERT INTO cooking_logs (recipe_id, servings, created_at) VALUES (?,?,?)")
        .bind(recipe.id)
        .bind(servings as i64)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
    tx.commit()
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
    Ok(Json(json!({ "ok": true, "recipe_id": recipe.id, "servings": servings as i64 })))
}
