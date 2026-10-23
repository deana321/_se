use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExpiryStatus {
    Normal,
    Soon,
    Expired,
}

/// UT-01: 剩餘預算 = 預算 - 支出（整數 NT$）
pub fn remaining_budget(monthly_budget: i64, total_expense: i64) -> i64 {
    monthly_budget - total_expense
}

/// UT-02: 金額驗證（必須 > 0）
pub fn validate_amount(amount: i64) -> Result<(), String> {
    if amount <= 0 {
        return Err("金額必須大於零".to_string());
    }
    Ok(())
}

pub fn validate_quantity(qty: f64, allow_zero: bool) -> Result<(), String> {
    if qty.is_nan() || qty.is_infinite() {
        return Err("數量不合法".to_string());
    }
    if allow_zero {
        if qty < 0.0 {
            return Err("庫存不得小於零".to_string());
        }
    } else if qty <= 0.0 {
        return Err("數量必須大於零".to_string());
    }
    Ok(())
}

/// UT-03: 有效期限判斷（後端唯一真相）
/// 正常：到期日 - 今天 > 3 天；即將到期：0~3 天；已過期：到期日 < 今天
pub fn expiry_status(expiry: NaiveDate, today: NaiveDate) -> ExpiryStatus {
    let diff = (expiry - today).num_days();
    if diff < 0 {
        ExpiryStatus::Expired
    } else if diff <= 3 {
        ExpiryStatus::Soon
    } else {
        ExpiryStatus::Normal
    }
}

/// UT-04 / UT-05: 扣除庫存
pub fn deduct_stock(current: f64, use_amount: f64) -> Result<f64, String> {
    validate_quantity(use_amount, false)?;
    if use_amount > current {
        return Err(format!("庫存不足：現有 {current}，需要 {use_amount}"));
    }
    let left = current - use_amount;
    if left < 0.0 {
        return Err("庫存不得小於零".to_string());
    }
    Ok(left)
}

/// UT-06: 購物總金額（數量 * 單價，加總，整數）
pub fn shopping_total(items: &[(f64, i64)]) -> i64 {
    items
        .iter()
        .map(|(qty, price)| (*qty * (*price as f64)).round() as i64)
        .sum()
}

/// 單位換算：g<->kg, ml<->L；其餘單位不可互換
/// 回傳換算為基準單位後的數量：重量基準 g，體積基準 ml
pub fn to_base_unit(qty: f64, unit: &str) -> Option<(String, f64)> {
    match unit {
        "g" => Some(("g".to_string(), qty)),
        "kg" => Some(("g".to_string(), qty * 1000.0)),
        "ml" => Some(("ml".to_string(), qty)),
        "L" => Some(("ml".to_string(), qty * 1000.0)),
        "顆" | "包" | "盒" | "瓶" | "片" => Some((unit.to_string(), qty)),
        _ => None,
    }
}

pub fn units_compatible(a: &str, b: &str) -> bool {
    match (to_base_unit(1.0, a), to_base_unit(1.0, b)) {
        (Some((ga, _)), Some((gb, _))) => ga == gb,
        _ => false,
    }
}

/// UT-07: 每週食材需求統計（同名+可換算單位才合併）
#[derive(Debug, Clone)]
pub struct Need {
    pub name: String,
    pub unit: String,
    pub quantity: f64,
}

pub fn aggregate_weekly_needs(items: Vec<Need>) -> Vec<Need> {
    let mut map: HashMap<(String, String), f64> = HashMap::new();
    // 先轉基準單位合併（重量/體積類），離散單位按原單位合併
    let mut base: HashMap<(String, String), f64> = HashMap::new();
    for n in items {
        if let Some((group, bqty)) = to_base_unit(n.quantity, &n.unit) {
            let key = (n.name.clone(), group.clone());
            *base.entry(key).or_insert(0.0) += bqty;
        } else {
            let key = (format!("{}#{}", n.name, n.unit), n.unit.clone());
            *map.entry(key).or_insert(0.0) += n.quantity;
        }
    }
    let mut out: Vec<Need> = Vec::new();
    for ((name, group), qty) in base {
        // 重量回填 g，體積回填 ml（保持一致換算規則）
        out.push(Need {
            name,
            unit: group,
            quantity: qty,
        });
    }
    for ((compound, _), qty) in map {
        let parts: Vec<&str> = compound.split('#').collect();
        out.push(Need {
            name: parts[0].to_string(),
            unit: parts[1].to_string(),
            quantity: qty,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name).then(a.unit.cmp(&b.unit)));
    out
}

/// UT-08: 料理食材比對
#[derive(Debug, Clone)]
pub struct StockLine {
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub expired: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Shortage {
    pub name: String,
    pub unit: String,
    pub need: f64,
    pub have: f64,
    pub missing: f64,
}

pub fn match_recipe(required: &[Need], stocks: &[StockLine]) -> (bool, Vec<Shortage>) {
    let mut shortages = Vec::new();
    // 可用庫存：排除已過期，轉基準單位加總
    let mut avail: HashMap<(String, String), f64> = HashMap::new();
    // 離散單位可用量
    let mut avail_discrete: HashMap<(String, String), f64> = HashMap::new();
    for s in stocks.iter().filter(|s| !s.expired) {
        if let Some((group, bqty)) = to_base_unit(s.quantity, &s.unit) {
            if group == "g" || group == "ml" {
                *avail.entry((s.name.clone(), group)).or_insert(0.0) += bqty;
            } else {
                *avail_discrete
                    .entry((s.name.clone(), s.unit.clone()))
                    .or_insert(0.0) += s.quantity;
            }
        }
    }
    let mut ok = true;
    for r in required {
        if let Some((group, bqty)) = to_base_unit(r.quantity, &r.unit) {
            if group == "g" || group == "ml" {
                let have = *avail.get(&(r.name.clone(), group.clone())).unwrap_or(&0.0);
                if have < bqty {
                    ok = false;
                    shortages.push(Shortage {
                        name: r.name.clone(),
                        unit: group.clone(),
                        need: bqty,
                        have,
                        missing: bqty - have,
                    });
                }
            } else {
                let have = *avail_discrete
                    .get(&(r.name.clone(), r.unit.clone()))
                    .unwrap_or(&0.0);
                if have < r.quantity {
                    ok = false;
                    shortages.push(Shortage {
                        name: r.name.clone(),
                        unit: r.unit.clone(),
                        need: r.quantity,
                        have,
                        missing: r.quantity - have,
                    });
                }
            }
        } else {
            ok = false;
            shortages.push(Shortage {
                name: r.name.clone(),
                unit: r.unit.clone(),
                need: r.quantity,
                have: 0.0,
                missing: r.quantity,
            });
        }
    }
    (ok, shortages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    // UT-01
    #[test]
    fn ut01_remaining_budget() {
        assert_eq!(remaining_budget(10000, 3000), 7000);
        assert_eq!(remaining_budget(20000, 20000), 0);
        assert_eq!(remaining_budget(10000, 12000), -2000); // 超支為負
    }

    // UT-02
    #[test]
    fn ut02_validate_amount() {
        assert!(validate_amount(150).is_ok());
        assert!(validate_amount(0).is_err());
        assert!(validate_amount(-100).is_err());
    }

    // UT-03
    #[test]
    fn ut03_expiry_status() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 23).unwrap();
        let normal = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();
        let soon1 = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
        let soon0 = NaiveDate::from_ymd_opt(2026, 10, 23).unwrap();
        let expired = NaiveDate::from_ymd_opt(2026, 10, 22).unwrap();
        assert_eq!(expiry_status(normal, today), ExpiryStatus::Normal);
        assert_eq!(expiry_status(soon1, today), ExpiryStatus::Soon);
        assert_eq!(expiry_status(soon0, today), ExpiryStatus::Soon);
        assert_eq!(expiry_status(expired, today), ExpiryStatus::Expired);
    }

    // UT-04
    #[test]
    fn ut04_deduct_stock_ok() {
        assert_eq!(deduct_stock(6.0, 2.0).unwrap(), 4.0);
        assert_eq!(deduct_stock(4.0, 4.0).unwrap(), 0.0);
    }

    // UT-05
    #[test]
    fn ut05_deduct_stock_insufficient() {
        assert!(deduct_stock(2.0, 5.0).is_err());
        assert!(deduct_stock(0.0, 1.0).is_err());
        assert!(deduct_stock(5.0, 0.0).is_err());
        assert!(deduct_stock(5.0, -1.0).is_err());
    }

    // UT-06
    #[test]
    fn ut06_shopping_total() {
        assert_eq!(shopping_total(&[(2.0, 30), (1.0, 60)]), 120);
        assert_eq!(shopping_total(&[]), 0);
        assert_eq!(shopping_total(&[(3.0, 0)]), 0);
    }

    // UT-07
    #[test]
    fn ut07_aggregate_weekly_needs() {
        let items = vec![
            Need { name: "雞蛋".into(), unit: "顆".into(), quantity: 2.0 },
            Need { name: "雞蛋".into(), unit: "顆".into(), quantity: 1.0 },
            Need { name: "牛奶".into(), unit: "ml".into(), quantity: 200.0 },
            Need { name: "牛奶".into(), unit: "L".into(), quantity: 0.5 },
        ];
        let agg = aggregate_weekly_needs(items);
        let egg = agg.iter().find(|n| n.name == "雞蛋").unwrap();
        assert_eq!(egg.quantity, 3.0);
        let milk = agg.iter().find(|n| n.name == "牛奶").unwrap();
        assert_eq!(milk.quantity, 700.0); // 200ml + 500ml
        // 不同單位不可混：顆 vs g 不合併
        let mixed = vec![
            Need { name: "鹽".into(), unit: "g".into(), quantity: 10.0 },
            Need { name: "鹽".into(), unit: "顆".into(), quantity: 1.0 },
        ];
        let agg2 = aggregate_weekly_needs(mixed);
        assert_eq!(agg2.len(), 2);
    }

    // UT-08
    #[test]
    fn ut08_match_recipe() {
        let req = vec![Need { name: "雞蛋".into(), unit: "顆".into(), quantity: 2.0 }];
        let enough = vec![StockLine { name: "雞蛋".into(), quantity: 6.0, unit: "顆".into(), expired: false }];
        let (ok, shortage) = match_recipe(&req, &enough);
        assert!(ok && shortage.is_empty());

        let few = vec![StockLine { name: "雞蛋".into(), quantity: 1.0, unit: "顆".into(), expired: false }];
        let (ok2, shortage2) = match_recipe(&req, &few);
        assert!(!ok2 && shortage2.len() == 1 && shortage2[0].missing == 1.0);

        // 過期庫存不計入
        let expired_stock = vec![StockLine { name: "雞蛋".into(), quantity: 10.0, unit: "顆".into(), expired: true }];
        let (ok3, _) = match_recipe(&req, &expired_stock);
        assert!(!ok3);
    }
}
