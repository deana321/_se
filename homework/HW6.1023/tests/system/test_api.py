"""ST-01 ~ ST-08：Web API 系統測試（httpx 對真正 server）。"""
from conftest import uniq, today, future


def test_st01_create_expense(client):
    """ST-01: POST 新增支出，201 且資料存在。"""
    name = uniq("晚餐")
    r = client.post("/api/transactions", json={
        "kind": "expense", "name": name, "amount": 150,
        "category": "餐飲", "date": today(), "note": "ST-01",
    })
    assert r.status_code == 201, r.text
    body = r.json()
    assert body["name"] == name and body["amount"] == 150
    # 副作用：列表可查到
    lst = client.get("/api/transactions").json()
    assert any(x["name"] == name for x in lst)


def test_st02_update_expense(client):
    """ST-02: PUT 修改支出，確認更新。"""
    name = uniq("午餐")
    c = client.post("/api/transactions", json={
        "kind": "expense", "name": name, "amount": 100,
        "category": "餐飲", "date": today(),
    })
    tid = c.json()["id"]
    u = client.put(f"/api/transactions/{tid}", json={
        "kind": "expense", "name": name, "amount": 250,
        "category": "餐飲", "date": today(),
    })
    assert u.status_code == 200, u.text
    assert u.json()["amount"] == 250


def test_st03_delete_expense(client):
    """ST-03: DELETE 刪除支出，確認移除。"""
    name = uniq("待刪")
    c = client.post("/api/transactions", json={
        "kind": "expense", "name": name, "amount": 50,
        "category": "其他", "date": today(),
    })
    tid = c.json()["id"]
    d = client.delete(f"/api/transactions/{tid}")
    assert d.status_code == 200, d.text
    lst = client.get("/api/transactions").json()
    assert all(x["id"] != tid for x in lst)


def test_st04_create_ingredient(client):
    """ST-04: POST 新增冰箱食材，201 且庫存正確。"""
    name = uniq("雞蛋")
    r = client.post("/api/ingredients", json={
        "name": name, "category": "蛋", "quantity": 6, "unit": "顆",
        "purchase_date": today(), "expiry_date": future(7),
    })
    assert r.status_code == 201, r.text
    assert r.json()["quantity"] == 6


def test_st05_recommendations(client):
    """ST-05: GET 食譜推薦，比對庫存狀態。"""
    # 先確保有足夠的蛋與番茄 → 番茄炒蛋應可製作
    for n, q, u in [("雞蛋", 6, "顆"), ("番茄", 3, "顆")]:
        client.post("/api/ingredients", json={
            "name": n, "quantity": q, "unit": u,
            "purchase_date": today(), "expiry_date": future(7),
        })
    r = client.get("/api/recipes/recommendations")
    assert r.status_code == 200, r.text
    recs = r.json()
    assert isinstance(recs, list) and len(recs) > 0
    tomato = [x for x in recs if x["recipe"]["name"] == "番茄炒蛋"]
    assert tomato and tomato[0]["can_make"] is True


def test_st06_complete_cooking(client):
    """ST-06: POST 完成料理，食材正確扣除。"""
    egg = uniq("雞蛋ST06")
    client.post("/api/ingredients", json={
        "name": egg, "quantity": 6, "unit": "顆",
        "purchase_date": today(), "expiry_date": future(7),
    })
    rc = client.post("/api/recipes", json={
        "name": uniq("炒蛋ST06"), "category": "中式",
        "time_minutes": 10, "difficulty": "簡單", "servings": 1,
        "ingredients": [{"name": egg, "quantity": 2, "unit": "顆"}],
        "steps": ["打蛋", "下鍋炒"],
    })
    rid = rc.json()["id"]
    done = client.post("/api/cooking/complete", json={"recipe_id": rid})
    assert done.status_code == 200, done.text
    ings = client.get("/api/ingredients", params={"q": egg}).json()
    total = sum(x["quantity"] for x in ings)
    assert total == 6 - 2


def test_st07_purchase_sync(client):
    """ST-07: POST 完成購買，記帳+冰箱同步。"""
    before_tx = len(client.get("/api/transactions").json())
    item = client.post("/api/shopping-list", json={
        "name": uniq("牛奶ST07"), "quantity": 1, "unit": "瓶",
        "estimated_price": 60,
    }).json()
    p = client.post("/api/purchases", json={
        "shopping_item_id": item["id"], "name": item["name"],
        "quantity": 1, "unit": "瓶", "actual_price": 65,
        "expiry_date": future(10),
    })
    assert p.status_code == 200, p.text
    after_tx = client.get("/api/transactions").json()
    assert len(after_tx) == before_tx + 1
    assert any("採購" in x["name"] and x["category"] == "食材" for x in after_tx)
    ings = client.get("/api/ingredients", params={"q": item["name"]}).json()
    assert sum(x["quantity"] for x in ings) >= 1


def test_st08_invalid_purchase_rollback(client):
    """ST-08: 不合法採購 → 400/422 且資料庫無部分更新。"""
    before_tx = len(client.get("/api/transactions").json())
    bad = client.post("/api/purchases", json={
        "name": uniq("壞資料"), "quantity": -1, "unit": "顆",
        "actual_price": 10,
    })
    assert bad.status_code in (400, 422), bad.text
    after_tx = len(client.get("/api/transactions").json())
    assert after_tx == before_tx
    # 重複購買同一項目應被拒絕
    item = client.post("/api/shopping-list", json={
        "name": uniq("重複購"), "quantity": 1, "unit": "顆",
        "estimated_price": 10,
    }).json()
    ok1 = client.post("/api/purchases", json={
        "shopping_item_id": item["id"], "name": item["name"],
        "quantity": 1, "unit": "顆", "actual_price": 10,
        "expiry_date": future(5),
    })
    assert ok1.status_code == 200, ok1.text
    ok2 = client.post("/api/purchases", json={
        "shopping_item_id": item["id"], "name": item["name"],
        "quantity": 1, "unit": "顆", "actual_price": 10,
        "expiry_date": future(5),
    })
    assert ok2.status_code in (400, 422, 409), ok2.text
