use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;

use crate::{
    logic::{aggregate_weekly_needs, to_base_unit, Need},
    models::{MealPlan, MealPlanInput, PurchaseInput, RecipeRow, ShoppingInput, ShoppingItem},
    routes::{bad_request, created},
};

#[derive(Debug, Deserialize, Default)]
pub struct PlanFilter {
    pub start: Option<String>,
    pub end: Option<String>,
}

// ---- Meal plans ----
pub async fn list_plans(
    State(pool): State<SqlitePool>,
    Query(f): Query<PlanFilter>,
) -> Result<Json<Vec<MealPlan>>, axum::response::Response> {
    let mut sql = "SELECT id, date, meal_type, recipe_id, is_eating_out, note FROM meal_plans".to_string();
    let mut conds = vec![];
    if f.start.is_some() {
        conds.push("date >= ?");
    }
    if f.end.is_some() {
        conds.push("date <= ?");
    }
    if !conds.is_empty() {
        sql.push_str(&format!(" WHERE {}", conds.join(" AND ")));
    }
    sql.push_str(" ORDER BY date ASC");
    let mut q = sqlx::query_as::<_, MealPlan>(&sql);
    if let Some(s) = f.start {
        q = q.bind(s);
    }
    if let Some(e) = f.end {
        q = q.bind(e);
    }
    let rows = q
        .fetch_all(&pool)
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
    Ok(Json(rows))
}

pub async fn create_plan(
    State(pool): State<SqlitePool>,
    Json(input): Json<MealPlanInput>,
) -> Result<impl axum::response::IntoResponse, axum::response::Response> {
    if !crate::routes::valid_date(&input.date) {
        return Err(bad_request("日期格式須為 YYYY-MM-DD".into()).into_response());
    }
    if !["breakfast", "lunch", "dinner"].contains(&input.meal_type.as_str()) {
        return Err(bad_request("meal_type 不合法".into()).into_response());
    }
    let eating_out = input.is_eating_out.unwrap_or(false);
    if !eating_out && input.recipe_id.is_none() {
        return Err(bad_request("非外食必須指定食譜".into()).into_response());
    }
    let r = sqlx::query(
        "INSERT INTO meal_plans (date, meal_type, recipe_id, is_eating_out, note) VALUES (?,?,?,?,?) ON CONFLICT(date, meal_type) DO UPDATE SET recipe_id=excluded.recipe_id, is_eating_out=excluded.is_eating_out, note=excluded.note",
    )
    .bind(&input.date)
    .bind(&input.meal_type)
    .bind(input.recipe_id)
    .bind(if eating_out { 1 } else { 0 })
    .bind(input.note.clone().unwrap_or_default())
    .execute(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let id = r.last_insert_rowid();
    // upsert 可能回 0，改查
    let row = sqlx::query_as::<_, MealPlan>(
        "SELECT id, date, meal_type, recipe_id, is_eating_out, note FROM meal_plans WHERE date=? AND meal_type=?",
    )
    .bind(&input.date)
    .bind(&input.meal_type)
    .fetch_one(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let _ = id;
    Ok(created(json!(row)))
}

pub async fn update_plan(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(input): Json<MealPlanInput>,
) -> Result<Json<MealPlan>, axum::response::Response> {
    if !crate::routes::valid_date(&input.date) {
        return Err(bad_request("日期格式須為 YYYY-MM-DD".into()).into_response());
    }
    let eating_out = input.is_eating_out.unwrap_or(false);
    let r = sqlx::query(
        "UPDATE meal_plans SET date=?, meal_type=?, recipe_id=?, is_eating_out=?, note=? WHERE id=?",
    )
    .bind(&input.date)
    .bind(&input.meal_type)
    .bind(input.recipe_id)
    .bind(if eating_out { 1 } else { 0 })
    .bind(input.note.clone().unwrap_or_default())
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    if r.rows_affected() == 0 {
        return Err(super::not_found("找不到該菜單").into_response());
    }
    let row = sqlx::query_as::<_, MealPlan>(
        "SELECT id, date, meal_type, recipe_id, is_eating_out, note FROM meal_plans WHERE id=?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    Ok(Json(row))
}

pub async fn delete_plan(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    let r = sqlx::query("DELETE FROM meal_plans WHERE id=?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
    if r.rows_affected() == 0 {
        return Err(super::not_found("找不到該菜單").into_response());
    }
    Ok(Json(json!({ "ok": true, "id": id })))
}

// ---- Shopping ----
pub async fn list_shopping(
    State(pool): State<SqlitePool>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    let rows = sqlx::query_as::<_, ShoppingItem>(
        "SELECT id, name, quantity, unit, estimated_price, is_purchased, note FROM shopping_items ORDER BY id DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let total: i64 = rows
        .iter()
        .map(|r| (r.quantity * r.estimated_price as f64).round() as i64)
        .sum();
    Ok(Json(json!({ "items": rows, "estimated_total": total })))
}

pub async fn create_shopping(
    State(pool): State<SqlitePool>,
    Json(input): Json<ShoppingInput>,
) -> Result<impl axum::response::IntoResponse, axum::response::Response> {
    if input.name.trim().is_empty() {
        return Err(bad_request("名稱不可空白".into()).into_response());
    }
    if input.quantity <= 0.0 {
        return Err(bad_request("數量必須大於零".into()).into_response());
    }
    if !crate::models::INGREDIENT_UNITS.contains(&input.unit.as_str()) {
        return Err(bad_request("單位不合法".into()).into_response());
    }
    let price = input.estimated_price.unwrap_or(0);
    if price < 0 {
        return Err(bad_request("預估單價不得為負".into()).into_response());
    }
    let r = sqlx::query(
        "INSERT INTO shopping_items (name, quantity, unit, estimated_price, is_purchased, note) VALUES (?,?,?,?,0,?)",
    )
    .bind(input.name.trim())
    .bind(input.quantity)
    .bind(&input.unit)
    .bind(price)
    .bind(input.note.clone().unwrap_or_default())
    .execute(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let id = r.last_insert_rowid();
    let row = sqlx::query_as::<_, ShoppingItem>(
        "SELECT id, name, quantity, unit, estimated_price, is_purchased, note FROM shopping_items WHERE id=?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    Ok(created(json!(row)))
}

pub async fn update_shopping(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(input): Json<ShoppingInput>,
) -> Result<Json<ShoppingItem>, axum::response::Response> {
    if input.quantity <= 0.0 {
        return Err(bad_request("數量必須大於零".into()).into_response());
    }
    let r = sqlx::query(
        "UPDATE shopping_items SET name=?, quantity=?, unit=?, estimated_price=?, note=? WHERE id=?",
    )
    .bind(input.name.trim())
    .bind(input.quantity)
    .bind(&input.unit)
    .bind(input.estimated_price.unwrap_or(0))
    .bind(input.note.clone().unwrap_or_default())
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    if r.rows_affected() == 0 {
        return Err(super::not_found("找不到該購物項目").into_response());
    }
    let row = sqlx::query_as::<_, ShoppingItem>(
        "SELECT id, name, quantity, unit, estimated_price, is_purchased, note FROM shopping_items WHERE id=?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    Ok(Json(row))
}

pub async fn delete_shopping(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    let r = sqlx::query("DELETE FROM shopping_items WHERE id=?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
    if r.rows_affected() == 0 {
        return Err(super::not_found("找不到該購物項目").into_response());
    }
    Ok(Json(json!({ "ok": true, "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct GenerateInput {
    pub start: String,
    pub end: String,
}

/// POST /api/shopping-list/generate — 依菜單需求 - 冰箱庫存 = 待買量
pub async fn generate(
    State(pool): State<SqlitePool>,
    Json(input): Json<GenerateInput>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    if !crate::routes::valid_date(&input.start) || !crate::routes::valid_date(&input.end) {
        return Err(bad_request("日期格式須為 YYYY-MM-DD".into()).into_response());
    }
    // 收集菜單 recipe_id
    let plans = sqlx::query_as::<_, MealPlan>(
        "SELECT id, date, meal_type, recipe_id, is_eating_out, note FROM meal_plans WHERE date >= ? AND date <= ? AND is_eating_out = 0 AND recipe_id IS NOT NULL",
    )
    .bind(&input.start)
    .bind(&input.end)
    .fetch_all(&pool)
    .await
    .map_err(|e| bad_request(e.to_string()).into_response())?;
    let mut needs: Vec<Need> = vec![];
    for p in plans {
        if let Some(rid) = p.recipe_id {
            if let Some(row) = sqlx::query_as::<_, RecipeRow>(
                "SELECT id, name, category, time_minutes, difficulty, servings, ingredients_json, steps_json, note FROM recipes WHERE id=?",
            )
            .bind(rid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| bad_request(e.to_string()).into_response())?
            {
                let ings: Vec<crate::models::RecipeIngredient> =
                    serde_json::from_str(&row.ingredients_json).unwrap_or_default();
                for i in ings {
                    needs.push(Need { name: i.name, unit: i.unit, quantity: i.quantity });
                }
            }
        }
    }
    let aggregated = aggregate_weekly_needs(needs);
    // 冰箱可用庫存（未過期）
    let today = crate::routes::today_str();
    let stocks = sqlx::query_as::<_, crate::models::Ingredient>(
        "SELECT id, name, category, quantity, unit, purchase_date, expiry_date, note FROM ingredients WHERE quantity > 0 AND expiry_date >= ?",
    )
    .bind(&today)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    let mut generated = vec![];
    for need in aggregated {
        // 計算同名且可換算的庫存總量（基準單位）
        let mut have_base = 0.0;
        let base_opt = to_base_unit(need.quantity, &need.unit);
        let group = base_opt.clone().map(|(g, _)| g).unwrap_or(need.unit.clone());
        let need_base = base_opt.map(|(_, q)| q).unwrap_or(need.quantity);
        for s in &stocks {
            if s.name != need.name {
                continue;
            }
            if let Some((sg, sq)) = to_base_unit(s.quantity, &s.unit) {
                if sg == group {
                    have_base += sq;
                }
            }
        }
        let mut missing_base = need_base - have_base;
        if missing_base < 0.0 {
            missing_base = 0.0;
        }
        if missing_base > 0.0 {
            generated.push(json!({
                "name": need.name,
                "unit": need.unit,
                "need": need.quantity,
                "have_base": have_base,
                "missing": missing_base,
            }));
            // 自動寫入購物清單（若同名同單位未購買項目已存在則累加）
            let exist = sqlx::query_as::<_, ShoppingItem>(
                "SELECT id, name, quantity, unit, estimated_price, is_purchased, note FROM shopping_items WHERE name=? AND unit=? AND is_purchased=0",
            )
            .bind(&need.name)
            .bind(&need.unit)
            .fetch_optional(&pool)
            .await
            .map_err(|e| bad_request(e.to_string()).into_response())?;
            // missing_base 已是基準單位；若原單位是 kg/L，需轉回原單位數量
            let missing_in_unit = match need.unit.as_str() {
                "kg" => missing_base / 1000.0,
                "L" => missing_base / 1000.0,
                _ => missing_base,
            };
            if let Some(e) = exist {
                sqlx::query("UPDATE shopping_items SET quantity = quantity + ? WHERE id=?")
                    .bind(missing_in_unit)
                    .bind(e.id)
                    .execute(&pool)
                    .await
                    .map_err(|e| bad_request(e.to_string()).into_response())?;
            } else {
                sqlx::query(
                    "INSERT INTO shopping_items (name, quantity, unit, estimated_price, is_purchased, note) VALUES (?,?,?,0,0,'菜單自動產生')",
                )
                .bind(&need.name)
                .bind(missing_in_unit)
                .bind(&need.unit)
                .execute(&pool)
                .await
                .map_err(|e| bad_request(e.to_string()).into_response())?;
            }
        }
    }
    Ok(Json(json!({ "ok": true, "generated": generated })))
}

/// POST /api/purchases — Transaction：購買紀錄 + 冰箱 + 記帳
pub async fn complete_purchase(
    State(pool): State<SqlitePool>,
    Json(input): Json<PurchaseInput>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    if input.name.trim().is_empty() {
        return Err(bad_request("名稱不可空白".into()).into_response());
    }
    if input.quantity <= 0.0 {
        return Err(bad_request("購買數量必須大於零".into()).into_response());
    }
    if input.actual_price < 0 {
        return Err(bad_request("實際金額不得為負".into()).into_response());
    }
    if !crate::models::INGREDIENT_UNITS.contains(&input.unit.as_str()) {
        return Err(bad_request("單位不合法".into()).into_response());
    }
    // 同一購物項目不可重複購買
    if let Some(sid) = input.shopping_item_id {
        let item = sqlx::query_as::<_, ShoppingItem>(
            "SELECT id, name, quantity, unit, estimated_price, is_purchased, note FROM shopping_items WHERE id=?",
        )
        .bind(sid)
        .fetch_optional(&pool)
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
        match item {
            Some(it) => {
                if it.is_purchased == 1 {
                    return Err(
                        bad_request("該購物項目已完成購買，不可重複".into()).into_response()
                    );
                }
            }
            None => return Err(bad_request("找不到購物項目".into()).into_response()),
        }
    }
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
    let today = crate::routes::today_str();
    let expiry = input.expiry_date.clone().unwrap_or_else(|| today.clone());
    if !expiry.is_empty() && !crate::routes::valid_date(&expiry) {
        tx.rollback().await.ok();
        return Err(bad_request("有效期限格式須為 YYYY-MM-DD".into()).into_response());
    }
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    // 1. 購買紀錄
    let pr = sqlx::query(
        "INSERT INTO purchases (shopping_item_id, name, quantity, unit, actual_price, expiry_date, created_at) VALUES (?,?,?,?,?,?,?)",
    )
    .bind(input.shopping_item_id)
    .bind(input.name.trim())
    .bind(input.quantity)
    .bind(&input.unit)
    .bind(input.actual_price)
    .bind(&expiry)
    .bind(&now)
    .execute(&mut *tx)
    .await;
    if let Err(e) = pr {
        tx.rollback().await.ok();
        return Err(bad_request(format!("建立購買紀錄失敗：{e}")).into_response());
    }
    // 2. 冰箱：同名同單位找最早到期批次累加，否則新增
    let exist = sqlx::query_as::<_, crate::models::Ingredient>(
        "SELECT id, name, category, quantity, unit, purchase_date, expiry_date, note FROM ingredients WHERE name=? AND unit=? ORDER BY expiry_date ASC LIMIT 1",
    )
    .bind(input.name.trim())
    .bind(&input.unit)
    .fetch_optional(&mut *tx)
    .await;
    match exist {
        Ok(Some(row)) => {
            if sqlx::query("UPDATE ingredients SET quantity = quantity + ? WHERE id=?")
                .bind(input.quantity)
                .bind(row.id)
                .execute(&mut *tx)
                .await
                .is_err()
            {
                tx.rollback().await.ok();
                return Err(bad_request("更新冰箱庫存失敗".into()).into_response());
            }
        }
        Ok(None) => {
            if sqlx::query(
                "INSERT INTO ingredients (name, category, quantity, unit, purchase_date, expiry_date, note) VALUES (?,?,?,?,?,?,?)",
            )
            .bind(input.name.trim())
            .bind("其他")
            .bind(input.quantity)
            .bind(&input.unit)
            .bind(&today)
            .bind(&expiry)
            .bind("採購新增")
            .execute(&mut *tx)
            .await
            .is_err()
            {
                tx.rollback().await.ok();
                return Err(bad_request("新增冰箱庫存失敗".into()).into_response());
            }
        }
        Err(e) => {
            tx.rollback().await.ok();
            return Err(bad_request(e.to_string()).into_response());
        }
    }
    // 3. 記帳：食材支出
    if sqlx::query(
        "INSERT INTO transactions (kind, name, amount, category, date, note) VALUES ('expense',?,?, '食材',?,?)",
    )
    .bind(format!("採購：{}", input.name.trim()))
    .bind(input.actual_price)
    .bind(&today)
    .bind("購物清單連動記帳")
    .execute(&mut *tx)
    .await
    .is_err()
    {
        tx.rollback().await.ok();
        return Err(bad_request("新增記帳失敗".into()).into_response());
    }
    // 4. 購物清單狀態
    if let Some(sid) = input.shopping_item_id {
        if sqlx::query("UPDATE shopping_items SET is_purchased=1 WHERE id=?")
            .bind(sid)
            .execute(&mut *tx)
            .await
            .is_err()
        {
            tx.rollback().await.ok();
            return Err(bad_request("更新購物狀態失敗".into()).into_response());
        }
    }
    tx.commit()
        .await
        .map_err(|e| bad_request(e.to_string()).into_response())?;
    Ok(Json(json!({ "ok": true })))
}
