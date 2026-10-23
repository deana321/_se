use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let opts = SqliteConnectOptions::from_str(database_url)
        .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    seed_recipes(&pool).await?;
    Ok(pool)
}

async fn seed_recipes(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM recipes")
        .fetch_one(pool)
        .await?;
    if count > 0 {
        return Ok(());
    }
    let seeds: Vec<(&str, &str, i64, &str, &str, &str)> = vec![
        ("番茄炒蛋", "中式", 15, "簡單",
            r#"[{"name":"雞蛋","quantity":3,"unit":"顆"},{"name":"番茄","quantity":2,"unit":"顆"}]"#,
            r#"["番茄切塊，雞蛋打散","熱鍋下油炒蛋盛起","炒番茄後回鍋調味"]"#),
        ("蛋炒飯", "中式", 15, "簡單",
            r#"[{"name":"雞蛋","quantity":2,"unit":"顆"},{"name":"白米","quantity":200,"unit":"g"}]"#,
            r#"["隔夜飯鬆散備用","炒蛋後下飯翻炒","加鹽與蔥花調味"]"#),
        ("香菇炒烏龍", "日式", 20, "簡單",
            r#"[{"name":"烏龍麵","quantity":1,"unit":"包"},{"name":"香菇","quantity":3,"unit":"顆"}]"#,
            r#"["烏龍麵燙熟","香菇切片爆香","下麵與醬油拌炒"]"#),
        ("咖哩飯", "日式", 30, "普通",
            r#"[{"name":"白米","quantity":200,"unit":"g"},{"name":"馬鈴薯","quantity":2,"unit":"顆"}]"#,
            r#"["食材切塊炒香","加水與咖哩塊燉煮","淋在白飯上"]"#),
        ("高麗菜炒肉片", "中式", 20, "普通",
            r#"[{"name":"高麗菜","quantity":300,"unit":"g"},{"name":"豬肉片","quantity":150,"unit":"g"}]"#,
            r#"["肉片醃製","大火炒肉片","下高麗菜快炒調味"]"#),
        ("日式蒸蛋", "日式", 25, "簡單",
            r#"[{"name":"雞蛋","quantity":2,"unit":"顆"},{"name":"高湯","quantity":300,"unit":"ml"}]"#,
            r#"["蛋液與高湯混合過篩","電鍋蒸12分鐘","淋醬油即可"]"#),
    ];
    for (name, cat, time, diff, ing, steps) in seeds {
        sqlx::query(
            "INSERT INTO recipes (name, category, time_minutes, difficulty, servings, ingredients_json, steps_json, note) VALUES (?,?,?,?,1,?,?,?)",
        )
        .bind(name)
        .bind(cat)
        .bind(time)
        .bind(diff)
        .bind(ing)
        .bind(steps)
        .bind("一人份示範食譜")
        .execute(pool)
        .await?;
    }
    Ok(())
}
