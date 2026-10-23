"""將 cargo test 輸出轉為 reports/unit.json（測試中心讀取用）。"""
import json
import re
import subprocess
import datetime
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]

def main():
    p = subprocess.run(["cargo", "test"],
                       cwd=ROOT / "server", capture_output=True, text=True,
                       encoding="utf-8", errors="replace")
    out = (p.stdout or "") + "\n" + (p.stderr or "")
    # 解析類似 "test logic::tests::ut01_remaining_budget ... ok"
    cases = []
    for line in out.splitlines():
        m = re.match(r"test\s+(\S+)\s+\.\.\.\s+(ok|FAILED|ignored)", line.strip())
        if m:
            name = m.group(1)
            status = "passed" if m.group(2) == "ok" else "failed"
            short = name.split("::")[-1]
            cases.append({"id": short.upper(), "name": name, "status": status, "message": ""})
    total = len(cases)
    passed = sum(1 for c in cases if c["status"] == "passed")
    data = {
        "total": total, "passed": passed, "failed": total - passed,
        "run_at": datetime.datetime.now().isoformat(timespec="seconds"),
        "cases": cases, "raw_exit": p.returncode,
    }
    (ROOT / "reports").mkdir(exist_ok=True)
    (ROOT / "reports" / "unit.json").write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(data, ensure_ascii=False, indent=2))
    raise SystemExit(0 if p.returncode == 0 else 1)

if __name__ == "__main__":
    main()
