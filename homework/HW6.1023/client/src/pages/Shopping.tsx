import { useEffect, useState } from 'react';
import { Api } from '../services/api';
import { UNITS, fmtNT } from '../types';

export default function Shopping() {
  const [items, setItems] = useState<any[]>([]);
  const [total, setTotal] = useState(0);
  const [form, setForm] = useState({ name: '', quantity: '', unit: '顆', estimated_price: '0', note: '' });
  const [msg, setMsg] = useState('');
  const [buy, setBuy] = useState({ qty: '', price: '', expiry: '' });
  const [buyId, setBuyId] = useState<number | null>(null);
  const load = () => Api.shopping().then(d => { setItems(d.items); setTotal(d.estimated_total); });
  useEffect(() => { load(); }, []);
  const add = async () => {
    setMsg('');
    try {
      await Api.createShopping({ name: form.name, quantity: Number(form.quantity), unit: form.unit, estimated_price: Number(form.estimated_price), note: form.note });
      setMsg('已新增購物項目'); setForm({ name: '', quantity: '', unit: '顆', estimated_price: '0', note: '' }); load();
    } catch (e: any) { setMsg(e?.response?.data?.error || '新增失敗'); }
  };
  const gen = async () => {
    const end = new Date(); end.setDate(end.getDate() + 7);
    try {
      const r = await Api.generateShopping(new Date().toISOString().slice(0, 10), end.toISOString().slice(0, 10));
      setMsg(`已依菜單產生 ${r.generated.length} 項`); load();
    } catch (e: any) { setMsg('產生失敗'); }
  };
  const purchase = async (it: any) => {
    try {
      await Api.purchase({ shopping_item_id: it.id, name: it.name, quantity: Number(buy.qty || it.quantity), unit: it.unit, actual_price: Number(buy.price || 0), expiry_date: buy.expiry || undefined });
      setMsg('購買完成，已同步記帳與冰箱'); setBuyId(null); load();
    } catch (e: any) { setMsg(e?.response?.data?.error || '購買失敗'); }
  };
  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h1 className="text-xl font-bold">購物清單</h1>
        <button className="btn-ghost" onClick={gen}>依本週菜單自動產生</button>
      </div>
      {msg && <p data-testid="shop-msg" className="card text-sm text-green-700">{msg}</p>}
      <div className="card space-y-2">
        <h2 className="font-bold">手動新增</h2>
        <div className="grid md:grid-cols-5 gap-2">
          <input className="input" placeholder="品名" aria-label="品名" data-testid="shop-name" value={form.name} onChange={e => setForm({ ...form, name: e.target.value })} />
          <input className="input" placeholder="數量" aria-label="數量" data-testid="shop-qty" type="number" value={form.quantity} onChange={e => setForm({ ...form, quantity: e.target.value })} />
          <select className="input" aria-label="單位" value={form.unit} onChange={e => setForm({ ...form, unit: e.target.value })}>{UNITS.map(u => <option key={u}>{u}</option>)}</select>
          <input className="input" placeholder="預估單價" aria-label="預估單價" data-testid="shop-price" type="number" value={form.estimated_price} onChange={e => setForm({ ...form, estimated_price: e.target.value })} />
          <button className="btn-primary" data-testid="shop-add" onClick={add}>新增</button>
        </div>
      </div>
      <div className="card">
        <div className="flex justify-between mb-2"><h2 className="font-bold">清單</h2><span className="text-sm">預估總金額：{fmtNT(total)}</span></div>
        {items.length === 0 ? <p className="text-gray-400 text-sm">購物清單是空的</p> : (
          <div className="divide-y divide-gray-50">
            {items.map(it => (
              <div key={it.id} className="py-2 text-sm flex flex-wrap justify-between gap-2" data-testid="shop-row">
                <span><b>{it.name}</b> {it.quantity}{it.unit}・預估 {fmtNT((it.quantity * it.estimated_price))} {it.is_purchased ? '（已購買）' : ''}</span>
                <span className="flex gap-2 items-center">
                  {!it.is_purchased && buyId !== it.id && <button className="underline" data-testid={`buy-${it.id}`} onClick={() => { setBuyId(it.id); setBuy({ qty: String(it.quantity), price: String(it.estimated_price), expiry: '' }); }}>完成購買</button>}
                  <button className="underline text-red-500" onClick={() => Api.deleteShopping(it.id).then(load)}>刪除</button>
                </span>
                {buyId === it.id && (
                  <div className="w-full grid md:grid-cols-4 gap-2 mt-2 bg-mist rounded-xl p-2">
                    <input className="input" placeholder="實際數量" aria-label="實際數量" type="number" value={buy.qty} onChange={e => setBuy({ ...buy, qty: e.target.value })} />
                    <input className="input" placeholder="實際金額" aria-label="實際金額" type="number" value={buy.price} onChange={e => setBuy({ ...buy, price: e.target.value })} />
                    <input className="input" type="date" aria-label="有效期限" value={buy.expiry} onChange={e => setBuy({ ...buy, expiry: e.target.value })} />
                    <div className="flex gap-2"><button className="btn-primary" data-testid="shop-confirm" onClick={() => purchase(it)}>確認購買</button><button className="btn-ghost" onClick={() => setBuyId(null)}>取消</button></div>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
