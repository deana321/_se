import { NavLink, Outlet } from 'react-router-dom';
import { Home, Wallet, Refrigerator, ChefHat, CalendarDays, ShoppingCart, FlaskConical } from 'lucide-react';

const links = [
  { to: '/', label: '首頁', icon: Home },
  { to: '/transactions', label: '生活記帳', icon: Wallet },
  { to: '/fridge', label: '我的冰箱', icon: Refrigerator },
  { to: '/recipes', label: '一人料理', icon: ChefHat },
  { to: '/plans', label: '每週菜單', icon: CalendarDays },
  { to: '/shopping', label: '購物清單', icon: ShoppingCart },
  { to: '/tests', label: '測試中心', icon: FlaskConical },
];

export default function Layout() {
  return (
    <div className="min-h-screen flex">
      <aside className="hidden md:flex w-60 flex-col bg-white border-r border-gray-100 p-4 gap-1">
        <div className="text-xl font-bold px-2 py-4">SoloLife ♡</div>
        {links.map(l => (
          <NavLink key={l.to} to={l.to} end={l.to === '/'}
            className={({ isActive }) => `flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm ${isActive ? 'bg-blush font-bold' : 'hover:bg-mist'}`}>
            <l.icon size={18} /><span>{l.label}</span>
          </NavLink>
        ))}
      </aside>
      <div className="flex-1 min-w-0">
        <main className="max-w-6xl mx-auto p-4 md:p-8 pb-24 md:pb-8"><Outlet /></main>
        <nav className="md:hidden fixed bottom-0 inset-x-0 bg-white border-t border-gray-100 flex justify-around py-2">
          {links.map(l => (
            <NavLink key={l.to} to={l.to} end={l.to === '/'} className="flex flex-col items-center text-[11px] gap-1 px-2 py-1">
              <l.icon size={20} /><span>{l.label}</span>
            </NavLink>
          ))}
        </nav>
      </div>
    </div>
  );
}
