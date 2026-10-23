import { useEffect, useState } from 'react';
import { Api } from '../services/api';
import { EXPENSE_CATS, Transaction, fmtNT } from '../types';

export default function Transactions() {
  const [list, setList] = useState<Transaction[]>([]);
  const [budget, setBudget] = useState(20000);
  const [stats, setStats] = useState<any>(null);
  const [form, setForm] = useState({ kind: 'expense', name: '', amount: '', category: '餐飲', date: new Date().toISOString().slice(0, 10), note: '' });
  const [editing, setEditing] = useState<number | null>(null);
  const [q, setQ] = useState(''); const [cat, setCat] = useState('');
  const [msg, setMsg] = useState('');
  const load = () => Promise.all([Api.listTx(), Api.budget(), Api.stats()]).then(([a, b, c]) => { setList(a); setBudget(b.monthly_budget); setStats(c); });
  useEffect(() => { load(); }, []);
  const submit = async () => {
    setMsg('');
    try {
      const body = { ...form, amount: Number(form.amount), note: form.note };
      if (editing) await Api.updateTx(editing, body); else await Api.createTx(body);
      setMsg('儲存成功'); setEditing(null);
      setForm({ kind: 'expense', name: '', amount: '', category: '餐飲', date: new Date().toISOString().slice(0, 10), note: '' });
      load();
    } catch (e: any) { setMsg(e?.response?.data?.error || '儲存失敗'); }
  };
  const filtered = list.filter(t =>
    (!q || t.name.includes(q)) && (!cat || t.category === cat));
  return (
    <div className="space-y-4">
      <h1 className="text-xl font-bold">生活記帳</h1>
      {stats && (
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <div className="card"><div className="text-sm text-gray-500">本月收入</div><div className="font-bold">{fmtNT(stats.total_income)}</div></div>
          <div className="card"><div className="text-sm text-gray-500">本月支出</div><div className="font-bold" data-testid="stats-expense">{fmtNT(stats.total_expense)}</div></div>
          <div className="card"><div className="text-sm text-gray-500">剩餘預算</div><div className="font-bold">{fmtNT(stats.remaining)}</div></div>
          <div className="card">
            <div className="text-sm text-gray-500">每月預算</div>
            <div className="flex gap-2"><input className="input" type="number" value={budget} onChange={e => setBudget(Number(e.target.value))} />
              <button className="btn-ghost" onClick={() => Api.putBudget(budget).then(() => { setMsg('預算已更新'); load(); })}>儲存</button></div>
            {stats.usage_rate >= 1 ? <p className="text-red-500 text-sm mt-1">已超支！</p> : stats.usage_rate >= 0.8 ? <p className="text-orange-500 text-sm mt-1">預算使用超過 80%</p> : null}
          </div>
        </div>
      )}
      <div className="card space-y-2">
        <h2 className="font-bold">{editing ? '編輯紀錄' : '新增收支'}</h2>
        <div className="grid md:grid-cols-3 gap-2">
          <div><label className="label">類型</label><select aria-label="類型" className="input" value={form.kind} onChange={e => setForm({ ...form, kind: e.target.value })}><option value="expense">支出</option><option value="income">收入</option></select></div>
          <div><label className="label" htmlFor="tx-name">項目名稱</label><input id="tx-name" data-testid="tx-name" placeholder="項目名稱" aria-label="項目名稱" className="input" value={form.name} onChange={e => setForm({ ...form, name: e.target.value })} /></div>
          <div><label className="label" htmlFor="tx-amount">金額</label><input id="tx-amount" data-testid="tx-amount" placeholder="金額" aria-label="金額" type="number" className="input" value={form.amount} onChange={e => setForm({ ...form, amount: e.target.value })} /></div>
          <div><label className="label">分類</label><select aria-label="分類" data-testid="tx-category" className="input" value={form.category} onChange={e => setForm({ ...form, category: e.target.value })}>{EXPENSE_CATS.map(c => <option key={c}>{c}</option>)}</select></div>
          <div><label className="label">日期</label><input aria-label="日期" type="date" className="input" value={form.date} onChange={e => setForm({ ...form, date: e.target.value })} /></div>
          <div><label className="label">備註</label><input aria-label="備註" className="input" value={form.note} onChange={e => setForm({ ...form, note: e.target.value })} /></div>
        </div>
        <div className="flex gap-2 items-center">
          <button className="btn-primary" data-testid="tx-save" onClick={submit}>{editing ? '更新' : '新增支出'}</button>
          {editing && <button className="btn-ghost" onClick={() => setEditing(null)}>取消</button>}
          {msg && <span data-testid="tx-msg" className="text-sm text-green-700">{msg}</span>}
        </div>
      </div>
      <div className="card">
        <div className="flex gap-2 mb-3">
          <input className="input" placeholder="搜尋紀錄" aria-label="搜尋紀錄" value={q} onChange={e => setQ(e.target.value)} />
          <select className="input" aria-label="類別篩選" value={cat} onChange={e => setCat(e.target.value)}><option value="">全部類別</option>{EXPENSE_CATS.map(c => <option key={c}>{c}</option>)}</select>
        </div>
        {filtered.length === 0 ? <p className="text-gray-400 text-sm">尚無紀錄</p> : (
          <div className="divide-y divide-gray-50">
            {filtered.map(t => (
              <div key={t.id} className="flex justify-between py-2 text-sm" data-testid="tx-row">
                <span>{t.date}｜{t.name}（{t.category}）<b className={t.kind === 'expense' ? 'text-red-500' : 'text-green-600'}> {t.kind === 'expense' ? '-' : '+'}{fmtNT(t.amount)}</b></span>
                <span className="flex gap-2">
                  <button className="underline" onClick={() => { setEditing(t.id); setForm({ kind: t.kind, name: t.name, amount: String(t.amount), category: t.category, date: t.date, note: t.note }); }}>編輯</button>
                  <button className="underline text-red-500" onClick={() => Api.deleteTx(t.id).then(load)}>刪除</button>
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
