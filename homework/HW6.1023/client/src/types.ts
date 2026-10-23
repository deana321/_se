export interface Transaction {
  id: number; kind: 'income' | 'expense'; name: string; amount: number;
  category: string; date: string; note: string;
}
export interface Ingredient {
  id: number; name: string; category: string; quantity: number; unit: string;
  purchase_date: string; expiry_date: string; note: string; status?: 'normal' | 'soon' | 'expired';
}
export interface RecipeIngredient { name: string; quantity: number; unit: string; }
export interface Recipe {
  id: number; name: string; category: string; time_minutes: number; difficulty: string;
  servings: number; ingredients: RecipeIngredient[]; steps: string[]; note: string;
}
export interface MealPlan {
  id: number; date: string; meal_type: 'breakfast' | 'lunch' | 'dinner';
  recipe_id: number | null; is_eating_out: number; note: string;
}
export interface ShoppingItem {
  id: number; name: string; quantity: number; unit: string;
  estimated_price: number; is_purchased: number; note: string;
}
export const EXPENSE_CATS = ['餐飲','食材','交通','生活用品','娛樂','房租','水電','其他'];
export const UNITS = ['顆','包','盒','瓶','片','g','kg','ml','L'];
export const fmtNT = (n: number) => `NT$ ${Number(n || 0).toLocaleString('zh-TW')}`;
export const todayTW = () => new Date().toLocaleDateString('zh-TW', { year: 'numeric', month: 'long', day: 'numeric', weekday: 'long' });
