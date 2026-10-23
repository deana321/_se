"""System tests: 對真正執行中的 Rust Web API 發 HTTP Request。"""
import os
import time
import uuid

import httpx
import pytest

BASE_URL = os.environ.get("BASE_URL", "http://localhost:3000")

@pytest.fixture(scope="session")
def client():
    # 等待 server 就緒
    for _ in range(30):
        try:
            r = httpx.get(f"{BASE_URL}/api/health", timeout=2)
            if r.status_code == 200:
                break
        except Exception:
            pass
        time.sleep(1)
    with httpx.Client(base_url=BASE_URL, timeout=10) as c:
        yield c

@pytest.fixture(autouse=True)
def _reset(client):
    # 若 server 允許 reset（ALLOW_TEST_RESET=1），每個測試前清空；否則靠唯一名稱隔離
    try:
        client.post("/api/test/reset")
    except Exception:
        pass
    yield

def uniq(prefix: str) -> str:
    return f"{prefix}-{uuid.uuid4().hex[:8]}"

def today() -> str:
    import datetime
    return datetime.date.today().isoformat()

def future(days: int) -> str:
    import datetime
    return (datetime.date.today() + datetime.timedelta(days=days)).isoformat()
