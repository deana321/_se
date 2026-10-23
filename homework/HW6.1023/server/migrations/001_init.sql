CREATE TABLE IF NOT EXISTS transactions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  kind TEXT NOT NULL CHECK (kind IN ('income','expense')),
  name TEXT NOT NULL,
  amount INTEGER NOT NULL CHECK (amount > 0),
  category TEXT NOT NULL,
  date TEXT NOT NULL,
  note TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS budget (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  monthly_budget INTEGER NOT NULL DEFAULT 20000 CHECK (monthly_budget >= 0)
);
INSERT OR IGNORE INTO budget (id, monthly_budget) VALUES (1, 20000);

CREATE TABLE IF NOT EXISTS ingredients (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  category TEXT NOT NULL DEFAULT '其他',
  quantity REAL NOT NULL CHECK (quantity >= 0),
  unit TEXT NOT NULL,
  purchase_date TEXT NOT NULL,
  expiry_date TEXT NOT NULL,
  note TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS recipes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  category TEXT NOT NULL DEFAULT '其他',
  time_minutes INTEGER NOT NULL DEFAULT 15,
  difficulty TEXT NOT NULL DEFAULT '簡單',
  servings INTEGER NOT NULL DEFAULT 1,
  ingredients_json TEXT NOT NULL DEFAULT '[]',
  steps_json TEXT NOT NULL DEFAULT '[]',
  note TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS meal_plans (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  date TEXT NOT NULL,
  meal_type TEXT NOT NULL CHECK (meal_type IN ('breakfast','lunch','dinner')),
  recipe_id INTEGER NULL REFERENCES recipes(id) ON DELETE SET NULL,
  is_eating_out INTEGER NOT NULL DEFAULT 0,
  note TEXT NOT NULL DEFAULT '',
  UNIQUE(date, meal_type)
);

CREATE TABLE IF NOT EXISTS shopping_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  quantity REAL NOT NULL CHECK (quantity > 0),
  unit TEXT NOT NULL,
  estimated_price INTEGER NOT NULL DEFAULT 0 CHECK (estimated_price >= 0),
  is_purchased INTEGER NOT NULL DEFAULT 0,
  note TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS purchases (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  shopping_item_id INTEGER NULL REFERENCES shopping_items(id) ON DELETE SET NULL,
  name TEXT NOT NULL,
  quantity REAL NOT NULL CHECK (quantity > 0),
  unit TEXT NOT NULL,
  actual_price INTEGER NOT NULL CHECK (actual_price >= 0),
  expiry_date TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cooking_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  recipe_id INTEGER NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
  servings INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL
);
