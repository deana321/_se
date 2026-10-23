import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Transactions from './pages/Transactions';
import Fridge from './pages/Fridge';
import Recipes from './pages/Recipes';
import Plans from './pages/Plans';
import Shopping from './pages/Shopping';
import TestCenter from './pages/TestCenter';

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route path="/" element={<Dashboard />} />
          <Route path="/transactions" element={<Transactions />} />
          <Route path="/fridge" element={<Fridge />} />
          <Route path="/recipes" element={<Recipes />} />
          <Route path="/shopping" element={<Shopping />} />
          <Route path="/plans" element={<Plans />} />
          <Route path="/tests" element={<TestCenter />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
