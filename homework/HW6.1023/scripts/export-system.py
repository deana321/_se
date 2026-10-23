"""將 pytest -v 輸出轉為 reports/system.json。"""
import json
import re
import subprocess
import datetime
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]

def main():
    p = subprocess.run(["pytest", "-v"], cwd=ROOT / "tests" / "system",
                       capture_output=True, text=True)
    out = (p.stdout or "") + "\n" + (p.stderr or "")
    cases = []
    for line in out.splitlines():
        m = re.match(r"(tests?/)?\S+::(test_\w+)\s+(PASSED|FAILED|SKIPPED|ERROR)", line.strip())
        if m:
            status = {"PASSED": "passed", "FAILED": "failed", "ERROR": "failed"}.get(m.group(3), "skipped")
            cases.append({"id": m.group(2).upper(), "name": m.group(2), "status": status, "message": ""})
    total = len(cases)
    passed = sum(1 for c in cases if c["status"] == "passed")
    data = {
        "total": total, "passed": passed, "failed": total - passed,
        "run_at": datetime.datetime.now().isoformat(timespec="seconds"),
        "cases": cases, "raw_exit": p.returncode,
    }
    (ROOT / "reports").mkdir(exist_ok=True)
    (ROOT / "reports" / "system.json").write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(data, ensure_ascii=False, indent=2))
    raise SystemExit(0 if p.returncode == 0 else 1)

if __name__ == "__main__":
    main()
