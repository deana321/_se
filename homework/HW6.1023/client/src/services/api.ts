import axios from 'axios';

const baseURL = import.meta.env.VITE_API_URL || 'http://localhost:3000';
export const api = axios.create({ baseURL, timeout: 10000 });

export const Api = {
  health: () => api.get('/api/health').then(r => r.data),
  listTx: (p: any = {}) => api.get('/api/transactions', { params: p }).then(r => r.data),
  createTx: (b: any) => api.post('/api/transactions', b).then(r => r.data),
  updateTx: (id: number, b: any) => api.put(`/api/transactions/${id}`, b).then(r => r.data),
  deleteTx: (id: number) => api.delete(`/api/transactions/${id}`).then(r => r.data),
  budget: () => api.get('/api/budget').then(r => r.data),
  putBudget: (v: number) => api.put('/api/budget', { monthly_budget: v }).then(r => r.data),
  stats: (month?: string) => api.get('/api/statistics', { params: month ? { start: month + '-01' } : {} }).then(r => r.data),
  dashboard: () => api.get('/api/dashboard').then(r => r.data),
  ings: (p: any = {}) => api.get('/api/ingredients', { params: p }).then(r => r.data),
  createIng: (b: any) => api.post('/api/ingredients', b).then(r => r.data),
  updateIng: (id: number, b: any) => api.put(`/api/ingredients/${id}`, b).then(r => r.data),
  deleteIng: (id: number) => api.delete(`/api/ingredients/${id}`).then(r => r.data),
  adjustIng: (id: number, delta: number) => api.post(`/api/ingredients/${id}/adjust`, { delta }).then(r => r.data),
  recipes: (q = '') => api.get('/api/recipes', { params: q ? { q } : {} }).then(r => r.data),
  recipe: (id: number) => api.get(`/api/recipes/${id}`).then(r => r.data),
  createRecipe: (b: any) => api.post('/api/recipes', b).then(r => r.data),
  updateRecipe: (id: number, b: any) => api.put(`/api/recipes/${id}`, b).then(r => r.data),
  deleteRecipe: (id: number) => api.delete(`/api/recipes/${id}`).then(r => r.data),
  recommendations: () => api.get('/api/recipes/recommendations').then(r => r.data),
  completeCooking: (recipe_id: number) => api.post('/api/cooking/complete', { recipe_id }).then(r => r.data),
  plans: (s?: string, e?: string) => api.get('/api/meal-plans', { params: { ...(s ? { start: s } : {}), ...(e ? { end: e } : {}) } }).then(r => r.data),
  createPlan: (b: any) => api.post('/api/meal-plans', b).then(r => r.data),
  updatePlan: (id: number, b: any) => api.put(`/api/meal-plans/${id}`, b).then(r => r.data),
  deletePlan: (id: number) => api.delete(`/api/meal-plans/${id}`).then(r => r.data),
  shopping: () => api.get('/api/shopping-list').then(r => r.data),
  createShopping: (b: any) => api.post('/api/shopping-list', b).then(r => r.data),
  updateShopping: (id: number, b: any) => api.put(`/api/shopping-list/${id}`, b).then(r => r.data),
  deleteShopping: (id: number) => api.delete(`/api/shopping-list/${id}`).then(r => r.data),
  generateShopping: (start: string, end: string) => api.post('/api/shopping-list/generate', { start, end }).then(r => r.data),
  purchase: (b: any) => api.post('/api/purchases', b).then(r => r.data),
  testReports: () => api.get('/api/test-reports').then(r => r.data),
};
