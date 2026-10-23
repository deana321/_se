import { useEffect, useState } from 'react';
import { Api } from '../services/api';

const MEALS = [{ k: 'breakfast', n: '早餐' }, { k: 'lunch', n: '午餐' }, { k: 'dinner', n: '晚餐' }];
const MEAL_KEYS = ['breakfast', 'lunch', 'dinner'];

function weekDays(base: Date) {
  const d = new Date(base);
  const day = (d.getDay() + 6) % 7; // Monday=0
  d.setDate(d.getDate() - day);
  return Array.from({ length: 7 }, (_, i) => {
    const t = new Date(d); t.setDate(d.getDate() + i);
    return t.toISOString().slice(0, 10);
  });
}

export default function Plans() {
  const [week] = useState(weekDays(new Date()));
  const [plans, setPlans] = useState<any[]>([]);
  const [recipes, setRecipes] = useState<any[]>([]);
  const [sel, setSel] = useState<Record<string, string>>({});
  const [msg, setMsg] = useState('');
  const load = () => Promise.all([Api.plans(week[0], week[6]), Api.recipes()]).then(([p, r]) => { setPlans(p); setRecipes(r); });
  useEffect(() => { load(); }, []);
  const save = async (date: string, meal: string) => {
    const v = sel[`${date}-${meal}`];
    if (!v) return;
    try {
      if (v === 'out') await Api.createPlan({ date, meal_type: meal, is_eating_out: true });
      else await Api.createPlan({ date, meal_type: meal, recipe_id: Number(v) });
      setMsg('菜單已儲存'); load();
    } catch (e: any) { setMsg(e?.response?.data?.error || '儲存失敗'); }
  };
  const byDateMeal = (date: string, meal: string) => plans.find(p => p.date === date && p.meal_type === meal);
  return (
    <div className="space-y-4">
      <h1 className="text-xl font-bold">每週菜單</h1>
      {msg && <p className="card text-sm text-green-700">{msg}</p>}
      <div className="grid md:grid-cols-7 gap-2">
        {week.map(d => (
          <div key={d} className="card !p-3">
            <div className="font-bold text-sm mb-2">{d.slice(5)}</div>
            {MEAL_KEYS.map((m, i) => {
              const p = byDateMeal(d, m);
              const rec = p && p.recipe_id ? recipes.find(r => r.id === p.recipe_id) : null;
              return (
                <div key={m} className="mb-2 text-xs border-t border-gray-50 pt-2">
                  <div className="text-gray-500">{MEALS[i].n}</div>
                  <div className="my-1">{p ? (p.is_eating_out ? '外食' : rec?.name || `食譜#${p.recipe_id}`) : '未安排'}</div>
                  <select className="input !text-xs" value={sel[`${d}-${m}`] || ''} onChange={e => setSel({ ...sel, [`${d}-${m}`]: e.target.value })}>
                    <option value="">選擇…</option>
                    <option value="out">外食</option>
                    {recipes.map(r => <option key={r.id} value={r.id}>{r.name}</option>)}
                  </select>
                  <div className="flex gap-1 mt-1">
                    <button className="underline" onClick={() => save(d, m)}>儲存</button>
                    {p && <button className="underline text-red-500" onClick={() => Api.deletePlan(p.id).then(load)}>刪除</button>}
                  </div>
                </div>
              );
            })}
          </div>
        ))}
      </div>
      <p className="text-xs text-gray-500">需求自動合併：相同食材會加總；g/kg 與 ml/L 自動換算，其餘單位不混合。菜單只計算需求，完成料理時才扣庫存。</p>
    </div>
  );
}
