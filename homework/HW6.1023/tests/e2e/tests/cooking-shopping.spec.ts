import { test, expect } from '@playwright/test';

const ts = () => Date.now().toString().slice(-6);

test('E2E-03：料理製作', async ({ page }) => {
  const egg = `雞蛋-${ts()}`;
  const exp = new Date(Date.now() + 7 * 864e5).toISOString().slice(0, 10);
  await page.goto('/');
  // 先在冰箱準備足夠食材
  await page.getByRole('link', { name: '我的冰箱' }).click();
  await page.getByTestId('ing-name').fill(egg);
  await page.getByTestId('ing-qty').fill('6');
  await page.getByTestId('ing-unit').selectOption('顆');
  await page.getByTestId('ing-expiry').fill(exp);
  await page.getByTestId('ing-save').click();
  await expect(page.getByTestId('ing-row').filter({ hasText: egg })).toBeVisible();

  // 建立只需要該雞蛋 2 顆的食譜（名稱唯一）
  await page.getByRole('link', { name: '一人料理' }).click();
  await page.getByRole('button', { name: '新增食譜' }).click();
  const recipeName = `E2E炒蛋-${ts()}`;
  await page.getByPlaceholder('料理名稱').fill(recipeName);
  await page.getByLabel('所需食材（每行：名稱,數量,單位）').fill(`${egg},2,顆`);
  await page.getByLabel('步驟（每行一步驟）').fill('打蛋\n下鍋');
  await page.getByRole('button', { name: '儲存' }).click();
  const row = page.getByTestId('recipe-row').filter({ hasText: recipeName });
  await expect(row).toBeVisible();
  await row.getByRole('button', { name: '完成料理' }).click();
  await expect(page.getByTestId('recipe-msg')).toContainText('料理完成');

  // 返回冰箱確認扣除 2 顆 → 剩 4
  await page.getByRole('link', { name: '我的冰箱' }).click();
  await expect(page.getByTestId('ing-row').filter({ hasText: egg })).toContainText('4顆');
});

test('E2E-04：購物與記帳連動', async ({ page }) => {
  const milk = `牛奶-${ts()}`;
  await page.goto('/');
  await page.getByRole('link', { name: '購物清單' }).click();
  await page.getByTestId('shop-name').fill(milk);
  await page.getByTestId('shop-qty').fill('1');
  await page.getByTestId('shop-price').fill('65');
  await page.getByTestId('shop-add').click();
  await expect(page.getByTestId('shop-msg')).toContainText('新增');
  const row = page.getByTestId('shop-row').filter({ hasText: milk });
  await expect(row).toBeVisible();
  await row.getByRole('button', { name: '完成購買' }).click();
  await page.getByTestId('shop-confirm').click();
  await expect(page.getByTestId('shop-msg')).toContainText('購買完成');

  await page.getByRole('link', { name: '生活記帳' }).click();
  await expect(page.getByTestId('tx-row').filter({ hasText: `採購：${milk}` })).toBeVisible();

  await page.getByRole('link', { name: '我的冰箱' }).click();
  await expect(page.getByTestId('ing-row').filter({ hasText: milk })).toBeVisible();
});
