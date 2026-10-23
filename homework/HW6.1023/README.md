# SoloLife｜一人生活智慧管理系統

> 韓系簡約風・獨居者生活管理・React + Rust(Axum) + SQLite・單元/系統/E2E 三種自動化測試

## 1. 專案介紹

SoloLife 整合**生活記帳、智慧冰箱、一人料理、每週菜單、智慧購物清單、採購記帳連動**與**測試中心**。
所有資料永久存於 SQLite（SQLx + Migration），前端一律經 RESTful API 存取後端，不使用假資料。

## 2. 功能介紹

- 首頁 Dashboard：本月支出/剩餘預算/冰箱總數/即將到期、支出圓餅+每日長條、到期提醒、今日菜單、料理推薦
- 生活記帳：收支 CRUD、日期/類別/關鍵字篩選、每月預算（80% 提醒/100% 超支）、統計（收入/支出/剩餘/類別/每日）
- 智慧冰箱：食材 CRUD、搜尋/分類、+1/-1 增減（不低於 0）、到期三態（正常/即將到期/已過期，後端函式判定）、多批次保存、FIFO 消耗
- 一人料理：6 道預設一人份食譜、CRUD、推薦排序（可做→用即期→缺料少）、缺少品項與數量、完成料理 Transaction 扣庫存
- 每週菜單：週一～週日 × 早/午/晚、可選外食、需求自動合併（g/kg、ml/L 換算，其餘不混算）、只計算不扣庫存
- 購物清單：手動 CRUD、預估總金額、依菜單自動產生（需求−可用庫存）、完成購買 Transaction（購買紀錄+冰箱+記帳+狀態）+ 防重複購買
- 測試中心：讀取 `reports/*.json` 顯示單元/系統/E2E 總數/通過/失敗/通過率/時間/明細；無檔案顯示「尚未執行測試」

## 3. 使用技術

- 前端：React 18 + TypeScript + Vite + Tailwind + React Router + Lucide + Recharts + axios
- 後端：Rust + Axum 0.7 + Tokio + Serde + SQLx(sqlite) + chrono + tower-http(cors)
- DB：SQLite（`server/migrations/001_init.sql` 自動遷移）
- 測試：`cargo test`｜Python pytest + httpx｜Playwright + TypeScript（Chromium headless）

## 4. 系統架構

```
瀏覽器 (Vite :5173) ──REST/JSON──> Axum API (:3000) ──SQLx──> sololife.db
        │                              │  routes/: transactions | fridge | recipes | plans
        └── /api/test-reports <── reports/*.json（腳本由測試輸出產生）
```

## 5. 安裝環境

- Node.js 18+、Python 3.10+、Rust（rustup，含 cargo）
- Windows：`winget install Rustlang.Rustup` 後重開終端機；或 https://rustup.rs/
- 驗證：`rustc --version`、`node --version`、`python --version`

## 6. 啟動方法

```powershell
# 後端（專案根）
$env:DATABASE_URL="sqlite://sololife.db"; $env:PORT="3000"
cd server; cargo run
# 前端（另開終端）
cd client; npm install; npm run dev   # http://localhost:5173
# 前端連後端：client/.env 或 VITE_API_URL，預設 http://localhost:3000
```

## 7. 資料庫初始化

- 首次 `cargo run` 即執行 `migrations/001_init.sql`（建表 + 預設預算 20000 + 6 道食譜）
- 測試隔離：系統測試用獨立 DB，例如 `$env:DATABASE_URL="sqlite://test.db"` 啟動 server；E2E 用時間戳唯一名稱；正式 `sololife.db` 與 `test.db` 不混用（皆已 gitignore）

## 8. API 說明（JSON，錯誤格式 `{"error": "..."}`）

| 方法 | 路徑 | 說明 |
|---|---|---|
| GET/POST | /api/transactions | 查詢（含 start/end/category/kind/q）/新增（201）|
| PUT/DELETE | /api/transactions/:id | 更新/刪除 |
| GET/PUT | /api/budget | 查詢/設定月預算 |
| GET | /api/statistics | 本月收入/支出/剩餘/類別/每日/使用率 |
| GET | /api/dashboard | 首頁聚合 |
| GET/POST | /api/ingredients | 查詢（含 q/category + status）/新增（201）|
| PUT/DELETE | /api/ingredients/:id | 更新/刪除 |
| POST | /api/ingredients/:id/adjust | delta 增減（防負）|
| GET/POST | /api/recipes | 查詢/新增 |
| GET/PUT/DELETE | /api/recipes/:id | 單筆/更新/刪除 |
| GET | /api/recipes/recommendations | 推薦排序 + shortages |
| POST | /api/cooking/complete | {recipe_id}，Transaction FIFO 扣庫存 |
| GET/POST | /api/meal-plans?start&end | 查詢/新增（upsert 同日同餐）|
| PUT/DELETE | /api/meal-plans/:id | 更新/刪除 |
| GET/POST | /api/shopping-list | 查詢（含預估總額）/新增 |
| PUT/DELETE | /api/shopping-list/:id | 更新/刪除 |
| POST | /api/shopping-list/generate | {start,end} 菜單→購物 |
| POST | /api/purchases | {shopping_item_id?,name,quantity,unit,actual_price,expiry_date?} Transaction 四步 |
| GET | /api/test-reports | reports/unit,system,e2e.json |
| POST | /api/test/reset | 僅 ALLOW_TEST_RESET=1 時清空測試資料 |

狀態碼：成功 200、新增 201、參數錯誤 400、找不到 404。

## 9. 單元測試執行方式

```powershell
cd server; cargo test
python ../scripts/export-unit.py   # 產生 reports/unit.json
```
覆蓋 UT-01 剩餘預算、UT-02 金額驗證、UT-03 到期判定、UT-04/05 扣庫存與不足、UT-06 購物總額、UT-07 週需求合併、UT-08 食譜比對（皆在 `server/src/logic.rs`，不依賴 HTTP/正式 DB）。

## 10. 系統測試執行方式

```powershell
# 終端機 A（測試 DB）
$env:DATABASE_URL="sqlite://test.db"; $env:PORT="3000"; $env:ALLOW_TEST_RESET="1"
cd server; cargo run
# 終端機 B
cd tests/system; python -m pip install -r requirements.txt; pytest -v
python ../../scripts/export-system.py   # 產生 reports/system.json
```
ST-01 新增支出、ST-02 修改、ST-03 刪除、ST-04 新增食材、ST-05 推薦、ST-06 完成料理扣庫存、ST-07 購買同步、ST-08 非法資料 Rollback＋防重複。每案驗 Status＋JSON＋副作用，可重複執行。

## 11. E2E 測試執行方式

```powershell
# 需後端 :3000 + 前端 :5173 同時啟動
cd tests/e2e; npm install; npx playwright install chromium
npx playwright test            # headless Chromium，失敗截圖+Trace
npx playwright show-report     # HTML 報告
```
E2E-01 記帳、E2E-02 冰箱、E2E-03 料理製作、E2E-04 購物記帳連動；全用 role/testid 穩定定位、無固定 sleep、每案時間戳獨立資料。

## 12. 測試報告查看方式

- 頁面：http://localhost:5173/tests（後端需啟動；無檔案顯示「尚未執行測試」）
- 檔案：`reports/unit.json`、`reports/system.json`、`reports/e2e.json`
- 一鍵：`powershell -ExecutionPolicy Bypass -File scripts/run-all.ps1`

## 13. 專案目錄說明

```
HW6.1023/
├─ client/ (React：components/Layout, pages/×7, services/api, types)
├─ server/ (src/main.rs, db.rs, models.rs, logic.rs+UT, routes/×4, migrations/001_init.sql, Cargo.toml)
├─ tests/system/ (conftest.py, test_api.py ST-01~08, requirements.txt, pytest.ini)
├─ tests/e2e/ (playwright.config.ts headless, tests/*.spec.ts E2E-01~04)
├─ scripts/ (export-unit.py, export-system.py, run-all.ps1)
├─ reports/ (.gitkeep, unit/system/e2e.json 執行後產生)
├─ .env.example, .gitignore, README.md
```

## 14. 驗收對照

- 記帳/預算/冰箱/到期/食譜/推薦/完成扣庫存/週菜單/購物產生/採購連動：皆有 API＋前端＋DB Transaction
- 三種測試：`cargo test`、`pytest`、`playwright test` 皆可獨立執行、可重複、隔離資料、失敗可查
- 無寫死假資料、無 Mock 假裝後端、金額整數 NT$、庫存不為負、防重複購買/扣庫存
