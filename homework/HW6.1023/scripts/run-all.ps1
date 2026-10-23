# SoloLife 一鍵執行（Windows PowerShell）
# 用法：在專案根目錄執行 powershell -ExecutionPolicy Bypass -File scripts/run-all.ps1
$ErrorActionPreference = "Continue"
$ROOT = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

Write-Host "=== 1/3 Unit test ==="
Push-Location "$ROOT/server"
cargo test
if ($LASTEXITCODE -ne 0) { Write-Host "單元測試失敗" }
python "$ROOT/scripts/export-unit.py"
Pop-Location

Write-Host "=== 2/3 System test (需 server 啟動) ==="
$env:BASE_URL = "http://localhost:3000"
Push-Location "$ROOT/tests/system"
python -m pip install -r requirements.txt
pytest -v
Pop-Location
python "$ROOT/scripts/export-system.py"

Write-Host "=== 3/3 E2E (需 server + client 啟動) ==="
Push-Location "$ROOT/tests/e2e"
npm install
npx playwright install chromium
npx playwright test
Pop-Location
Write-Host "完成。報告：reports/unit.json、reports/system.json、reports/e2e.json；Playwright HTML 在 tests/e2e/playwright-report"
