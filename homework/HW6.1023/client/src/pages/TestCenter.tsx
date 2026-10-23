import { useEffect, useState } from 'react';
import { Api } from '../services/api';

function Card({ title, data }: { title: string; data: any }) {
  if (!data) return (
    <div className="card"><h2 className="font-bold">{title}</h2><p className="text-gray-400 text-sm mt-2">尚未執行測試</p></div>
  );
  return (
    <div className="card">
      <h2 className="font-bold">{title}</h2>
      <div className="grid grid-cols-2 gap-2 mt-2 text-sm">
        <span>總數：{data.total}</span><span>通過：{data.passed}</span>
        <span>失敗：{data.failed}</span><span>通過率：{data.total ? Math.round((data.passed / data.total) * 100) : 0}%</span>
      </div>
      <div className="text-xs text-gray-500 mt-1">最近執行：{data.run_at || '—'}</div>
      <div className="mt-2 max-h-48 overflow-auto text-xs divide-y divide-gray-50">
        {(data.cases || []).map((c: any, i: number) => (
          <div key={i} className="py-1 flex justify-between gap-2">
            <span>{c.id} {c.name}</span>
            <span className={c.status === 'passed' ? 'text-green-600' : 'text-red-500'}>{c.status}{c.message ? `：${c.message}` : ''}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export default function TestCenter() {
  const [r, setR] = useState<any>(null);
  useEffect(() => { Api.testReports().then(setR).catch(() => setR(null)); }, []);
  if (!r) return <div className="card">載入中…（若後端未啟動則無法顯示）</div>;
  return (
    <div className="space-y-4">
      <h1 className="text-xl font-bold">軟體測試中心</h1>
      <p className="text-sm text-gray-500">測試結果來自 scripts 執行後產生的 reports/*.json，由 /api/test-reports 讀取。沒有執行紀錄會顯示「尚未執行測試」，不會預寫假資料。</p>
      <div className="grid md:grid-cols-3 gap-4">
        <Card title="Unit Testing" data={r.unit} />
        <Card title="System Testing" data={r.system} />
        <Card title="E2E Testing" data={r.e2e} />
      </div>
      <div className="card text-sm text-gray-600">
        執行方式：cargo test（單元）→ scripts/export-unit.py；pytest tests/system（系統）；npx playwright test（E2E）。詳見 README。
      </div>
    </div>
  );
}
