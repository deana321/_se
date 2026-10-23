import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { PieChart, Pie, Cell, ResponsiveContainer, BarChart, Bar, XAxis, YAxis, Tooltip } from 'recharts';
import { Api } from '../services/api';
import { fmtNT, todayTW } from '../types';

const COLORS = ['#A8BFA8', '#F4B8C1', '#F6D186', '#A8C8E8', '#C8B8E0', '#F0B8A0', '#B8D8D0', '#D8C8B8'];

export default function Dashboard() {
  const [d, setD] = useState<any>(null);
  const [stats, setStats] = useState<any>(null);
  const [recs, setRecs] = useState<any[]>([]);
  const [err, setErr] = useState('');
  useEffect(() => {
    Promise.all([Api.dashboard(), Api.stats(), Api.recommendations()])
      .then(([a, b, c]) => { setD(a); setStats(b); setRecs(c.slice(0, 3)); })
      .catch(() => setErr('無法連線到後端，請確認 server 已啟動（cargo run，port 3000）'));
  }, []);
  if (err) return <div className="card" data-testid="dash-error">{err}</div>;
  if (!d || !stats) return <div className="card">載入中…</div>;
  const pie = Object.entries(stats.by_category || {}).map(([name, value]) => ({ name, value }));
  const bars = Object.entries(stats.by_day || {}).map(([day, v]) => ({ day: day.slice(5), 金額: v }));
  return (
    <div className="space-y-6">
      <div><h1 className="text-2xl font-bold">今天也要好好生活 ♡</h1><p className="text-gray-500 text-sm">{todayTW()}</p></div>
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="card"><div className="text-sm text-gray-500">本月總支出</div><div className="text-xl font-bold" data-testid="month-expense">{fmtNT(d.month_expense)}</div></div>
        <div className="card"><div className="text-sm text-gray-500">本月剩餘預算</div><div className="text-xl font-bold" data-testid="month-remaining">{fmtNT(d.remaining)}</div></div>
        <div className="card"><div className="text-sm text-gray-500">冰箱食材總數</div><div className="text-xl font-bold" data-testid="ing-count">{d.ingredient_count}</div></div>
        <div className="card"><div className="text-sm text-gray-500">即將到期食材</div><div className="text-xl font-bold" data-testid="expiring-count">{d.expiring_count}</div></div>
      </div>
      <div className="grid lg:grid-cols-2 gap-4">
        <div className="card">
          <h2 className="font-bold mb-2">本月支出分析</h2>
          {pie.length === 0 ? <p className="text-gray-400 text-sm">尚無支出紀錄</p> : (
            <ResponsiveContainer width="100%" height={220}>
              <PieChart><Pie data={pie} dataKey="value" nameKey="name" outerRadius={80} label>
                {pie.map((_, i) => <Cell key={i} fill={COLORS[i % COLORS.length]} />)}
              </Pie><Tooltip /></PieChart>
            </ResponsiveContainer>
          )}
          {bars.length > 0 && (
            <ResponsiveContainer width="100%" height={180}>
              <BarChart data={bars}><XAxis dataKey="day" /><YAxis /><Tooltip /><Bar dataKey="金額" fill="#A8BFA8" /></BarChart>
            </ResponsiveContainer>
          )}
        </div>
        <div className="space-y-4">
          <div className="card">
            <h2 className="font-bold mb-2">冰箱到期提醒</h2>
            {(d.expiring_list || []).length === 0 ? <p className="text-gray-400 text-sm">沒有即將到期的食材</p> :
              (d.expiring_list || []).slice(0, 5).map((x: any) => (
                <div key={x.id} className="flex justify-between text-sm py-1 border-b border-gray-50">
                  <span>{x.name} {x.quantity}{x.unit}</span><span className="text-red-400">{x.expiry_date} 到期</span>
                </div>
              ))}
          </div>
          <div className="card">
            <h2 className="font-bold mb-2">今日菜單</h2>
            {(d.today_plans || []).length === 0 ? <p className="text-gray-400 text-sm">今天還沒有安排菜單</p> :
              (d.today_plans || []).map((p: any) => (
                <div key={p.id} className="text-sm py-1">{p.meal_type}：{p.is_eating_out ? '外食' : `食譜 #${p.recipe_id}`}</div>
              ))}
          </div>
          <div className="card">
            <h2 className="font-bold mb-2">今日料理推薦</h2>
            {recs.length === 0 ? <p className="text-gray-400 text-sm">尚無食譜</p> :
              recs.map((r: any) => (
                <div key={r.recipe.id} className="text-sm py-1 flex justify-between">
                  <span>{r.recipe.name}</span>
                  <span className={r.can_make ? 'text-green-600' : 'text-gray-400'}>{r.can_make ? '可以直接製作' : `缺 ${r.missing_count} 樣`}</span>
                </div>
              ))}
            <Link to="/recipes" className="text-sm text-green-700 underline">前往一人料理 →</Link>
          </div>
        </div>
      </div>
    </div>
  );
}
