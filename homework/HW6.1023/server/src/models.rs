use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: i64,
    pub kind: String,
    pub name: String,
    pub amount: i64,
    pub category: String,
    pub date: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    pub kind: String,
    pub name: String,
    pub amount: i64,
    pub category: String,
    pub date: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Ingredient {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub quantity: f64,
    pub unit: String,
    pub purchase_date: String,
    pub expiry_date: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientInput {
    pub name: String,
    pub category: Option<String>,
    pub quantity: f64,
    pub unit: String,
    pub purchase_date: String,
    pub expiry_date: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredient {
    pub name: String,
    pub quantity: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecipeRow {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub time_minutes: i64,
    pub difficulty: String,
    pub servings: i64,
    pub ingredients_json: String,
    pub steps_json: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub time_minutes: i64,
    pub difficulty: String,
    pub servings: i64,
    pub ingredients: Vec<RecipeIngredient>,
    pub steps: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInput {
    pub name: String,
    pub category: Option<String>,
    pub time_minutes: Option<i64>,
    pub difficulty: Option<String>,
    pub servings: Option<i64>,
    pub ingredients: Vec<RecipeIngredient>,
    pub steps: Vec<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MealPlan {
    pub id: i64,
    pub date: String,
    pub meal_type: String,
    pub recipe_id: Option<i64>,
    pub is_eating_out: i64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealPlanInput {
    pub date: String,
    pub meal_type: String,
    pub recipe_id: Option<i64>,
    pub is_eating_out: Option<bool>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ShoppingItem {
    pub id: i64,
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub estimated_price: i64,
    pub is_purchased: i64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingInput {
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub estimated_price: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseInput {
    pub shopping_item_id: Option<i64>,
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub actual_price: i64,
    pub expiry_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookingInput {
    pub recipe_id: i64,
    pub servings: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}

pub const EXPENSE_CATEGORIES: &[&str] = &[
    "餐飲", "食材", "交通", "生活用品", "娛樂", "房租", "水電", "其他",
];
pub const INGREDIENT_UNITS: &[&str] =
    &["顆", "包", "盒", "瓶", "片", "g", "kg", "ml", "L"];
