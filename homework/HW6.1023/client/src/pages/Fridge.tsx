import { useEffect, useState } from 'react';
import { Api } from '../services/api';
import { UNITS } from '../types';

export default function Fridge() {
  const [list, setList] = useState<any[]>([]);
  const [q, setQ] = useState('');
  const [form, setForm] = useState({ name: '', category: '其他', quantity: '', unit: '顆', purchase_date: new Date().toISOString().slice(0, 10), expiry_date: '', note: '' });
  const [editing, setEditing] = useState<number | null>(null);
  const [msg, setMsg] = useState('');
  const load = () => Api.ings().then(setList);
  useEffect(() => { load(); }, []);
  const submit = async () => {
    setMsg('');
    try {
      const body = { ...form, quantity: Number(form.quantity) };
      if (editing) await Api.updateIng(editing, body); else await Api.createIng(body);
      setMsg('儲存成功'); setEditing(null);
      setForm({ name: '', category: '其他', quantity: '', unit: '顆', purchase_date: new Date().toISOString().slice(0, 10), expiry_date: '', note: '' });
      load();
    } catch (e: any) { setMsg(e?.response?.data?.error || '儲存失敗'); }
  };
  const filtered = list.filter(x => !q || x.name.includes(q));
  const badge = (s: string) => s === 'expired' ? 'text-red-500' : s === 'soon' ? 'text-orange-500' : 'text-green-600';
  return (
    <div className="space-y-4">
      <h1 className="text-xl font-bold">我的冰箱</h1>
      <div className="card space-y-2">
        <h2 className="font-bold">{editing ? '編輯食材' : '新增食材'}</h2>
        <div className="grid md:grid-cols-4 gap-2">
          <div><label className="label" htmlFor="ing-name">食材名稱</label><input id="ing-name" data-testid="ing-name" aria-label="食材名稱" className="input" value={form.name} onChange={e => setForm({ ...form, name: e.target.value })} /></div>
          <div><label className="label">數量</label><input data-testid="ing-qty" aria-label="數量" type="number" className="input" value={form.quantity} onChange={e => setForm({ ...form, quantity: e.target.value })} /></div>
          <div><label className="label">單位</label><select data-testid="ing-unit" aria-label="單位" className="input" value={form.unit} onChange={e => setForm({ ...form, unit: e.target.value })}>{UNITS.map(u => <option key={u}>{u}</option>)}</select></div>
          <div><label className="label">有效期限</label><input data-testid="ing-expiry" aria-label="有效期限" type="date" className="input" value={form.expiry_date} onChange={e => setForm({ ...form, expiry_date: e.target.value })} /></div>
          <div><label className="label">購買日期</label><input aria-label="購買日期" type="date" className="input" value={form.purchase_date} onChange={e => setForm({ ...form, purchase_date: e.target.value })} /></div>
          <div><label className="label">分類</label><input aria-label="分類" className="input" value={form.category} onChange={e => setForm({ ...form, category: e.target.value })} /></div>
          <div className="md:col-span-2"><label className="label">備註</label><input aria-label="備註" className="input" value={form.note} onChange={e => setForm({ ...form, note: e.target.value })} /></div>
        </div>
        <div className="flex gap-2 items-center">
          <button className="btn-primary" data-testid="ing-save" onClick={submit}>{editing ? '更新' : '新增食材'}</button>
          {editing && <button className="btn-ghost" onClick={() => setEditing(null)}>取消</button>}
          {msg && <span data-testid="ing-msg" className="text-sm text-green-700">{msg}</span>}
        </div>
      </div>
      <div className="card">
        <input className="input mb-3" placeholder="搜尋食材" aria-label="搜尋食材" value={q} onChange={e => setQ(e.target.value)} />
        {filtered.length === 0 ? <p className="text-gray-400 text-sm">冰箱空空如也，快新增第一筆食材吧</p> : (
          <div className="divide-y divide-gray-50">
            {filtered.map(x => (
              <div key={x.id} className="flex flex-wrap justify-between items-center py-2 text-sm gap-2" data-testid="ing-row">
                <span><b>{x.name}</b> {x.quantity}{x.unit}｜{x.expiry_date} 到期 <span className={badge(x.status)}>{x.status === 'expired' ? '已過期' : x.status === 'soon' ? '即將到期' : '正常'}</span></span>
                <span className="flex gap-2">
                  <button className="underline" onClick={() => Api.adjustIng(x.id, 1).then(load)}>+1</button>
                  <button className="underline" onClick={() => Api.adjustIng(x.id, -1).then(load).catch(() => alert('庫存不足'))}>-1</button>
                  <button className="underline" onClick={() => { setEditing(x.id); setForm({ name: x.name, category: x.category, quantity: String(x.quantity), unit: x.unit, purchase_date: x.purchase_date, expiry_date: x.expiry_date, note: x.note }); }}>編輯</button>
                  <button className="underline text-red-500" onClick={() => Api.deleteIng(x.id).then(load)}>刪除</button>
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
