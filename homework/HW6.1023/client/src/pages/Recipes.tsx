import { useEffect, useState } from 'react';
import { Api } from '../services/api';
import { Recipe, UNITS } from '../types';

export default function Recipes() {
  const [recs, setRecs] = useState<any[]>([]);
  const [q, setQ] = useState('');
  const [msg, setMsg] = useState('');
  const [show, setShow] = useState(false);
  const [editing, setEditing] = useState<number | null>(null);
  const [form, setForm] = useState({ name: '', category: '中式', time_minutes: 15, difficulty: '簡單', servings: 1, ingredientsText: '雞蛋,2,顆', stepsText: '步驟一\n步驟二', note: '' });
  const load = () => Api.recommendations().then(setRecs);
  useEffect(() => { load(); }, []);
  const parseIng = () => form.ingredientsText.split('\n').filter(Boolean).map(l => {
    const [name, quantity, unit] = l.split(','); return { name: name?.trim(), quantity: Number(quantity), unit: unit?.trim() || '顆' };
  });
  const submit = async () => {
    setMsg('');
    try {
      const body = {
        name: form.name, category: form.category, time_minutes: Number(form.time_minutes),
        difficulty: form.difficulty, servings: Number(form.servings),
        ingredients: parseIng(), steps: form.stepsText.split('\n').filter(Boolean), note: form.note
      };
      if (editing) await Api.updateRecipe(editing, body); else await Api.createRecipe(body);
      setMsg('食譜已儲存'); setShow(false); setEditing(null); load();
    } catch (e: any) { setMsg(e?.response?.data?.error || '儲存失敗'); }
  };
  const complete = async (id: number) => {
    setMsg('');
    try { await Api.completeCooking(id); setMsg('料理完成，已扣除庫存'); load(); }
    catch (e: any) { setMsg(e?.response?.data?.error || '庫存不足，無法完成'); }
  };
  const filtered = recs.filter(r => !q || (r.recipe as Recipe).name.includes(q));
  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h1 className="text-xl font-bold">一人料理</h1>
        <button className="btn-primary" onClick={() => setShow(true)}>新增食譜</button>
      </div>
      {msg && <p data-testid="recipe-msg" className="text-sm text-green-700 card">{msg}</p>}
      <input className="input" placeholder="搜尋食譜" aria-label="搜尋食譜" value={q} onChange={e => setQ(e.target.value)} />
      {filtered.length === 0 ? <div className="card text-gray-400 text-sm">尚無食譜</div> :
        <div className="grid md:grid-cols-2 gap-4">
          {filtered.map(r => (
            <div key={r.recipe.id} className="card" data-testid="recipe-row">
              <div className="flex justify-between"><b>{r.recipe.name}</b><span className="text-xs text-gray-500">{r.recipe.time_minutes}分・{r.recipe.difficulty}</span></div>
              <div className="text-xs text-gray-500 mt-1">食材：{r.recipe.ingredients.map((i: any) => `${i.name}${i.quantity}${i.unit}`).join('、')}</div>
              <div className={`text-sm mt-2 ${r.can_make ? 'text-green-600' : 'text-orange-500'}`}>
                {r.can_make ? (r.uses_soon_expiring ? '可以直接製作（可消耗即期食材）' : '可以直接製作') : `缺少：${r.shortages.map((s: any) => `${s.name}缺${s.missing}${s.unit}`).join('、')}`}
              </div>
              <div className="flex gap-2 mt-3">
                <button className="btn-primary" data-testid={`cook-${r.recipe.id}`} onClick={() => complete(r.recipe.id)}>完成料理</button>
                <button className="btn-ghost" onClick={() => { setEditing(r.recipe.id); setForm({ name: r.recipe.name, category: r.recipe.category, time_minutes: r.recipe.time_minutes, difficulty: r.recipe.difficulty, servings: r.recipe.servings, ingredientsText: r.recipe.ingredients.map((i: any) => `${i.name},${i.quantity},${i.unit}`).join('\n'), stepsText: r.recipe.steps.join('\n'), note: r.recipe.note }); setShow(true); }}>編輯</button>
                <button className="btn-ghost" onClick={() => Api.deleteRecipe(r.recipe.id).then(load)}>刪除</button>
              </div>
            </div>
          ))}
        </div>}
      {show && (
        <div className="fixed inset-0 bg-black/30 flex items-center justify-center p-4">
          <div className="card w-full max-w-lg space-y-2">
            <h2 className="font-bold">{editing ? '編輯食譜' : '新增食譜'}</h2>
            <input className="input" placeholder="料理名稱" aria-label="料理名稱" value={form.name} onChange={e => setForm({ ...form, name: e.target.value })} />
            <div className="grid grid-cols-3 gap-2">
              <select className="input" value={form.category} onChange={e => setForm({ ...form, category: e.target.value })}><option>中式</option><option>日式</option><option>西式</option><option>其他</option></select>
              <select className="input" value={form.difficulty} onChange={e => setForm({ ...form, difficulty: e.target.value })}><option>簡單</option><option>普通</option><option>困難</option></select>
              <input className="input" type="number" value={form.time_minutes} onChange={e => setForm({ ...form, time_minutes: Number(e.target.value) })} />
            </div>
            <label className="label">所需食材（每行：名稱,數量,單位）</label>
            <textarea className="input" rows={3} value={form.ingredientsText} onChange={e => setForm({ ...form, ingredientsText: e.target.value })} />
            <label className="label">步驟（每行一步驟）</label>
            <textarea className="input" rows={3} value={form.stepsText} onChange={e => setForm({ ...form, stepsText: e.target.value })} />
            <div className="text-xs text-gray-500">單位可用：{UNITS.join('、')}</div>
            <div className="flex gap-2"><button className="btn-primary" onClick={submit}>儲存</button><button className="btn-ghost" onClick={() => { setShow(false); setEditing(null); }}>取消</button></div>
          </div>
        </div>
      )}
    </div>
  );
}
