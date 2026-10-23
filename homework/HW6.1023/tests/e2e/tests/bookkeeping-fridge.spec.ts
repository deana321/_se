import { test, expect } from '@playwright/test';

const ts = () => Date.now().toString().slice(-6);

test('E2E-01：生活記帳', async ({ page }) => {
  const name = `晚餐-${ts()}`;
  await page.goto('/');
  await page.getByRole('link', { name: '生活記帳' }).click();
  await expect(page.getByRole('heading', { name: '生活記帳' })).toBeVisible();
  await page.getByTestId('tx-name').fill(name);
  await page.getByTestId('tx-amount').fill('150');
  await page.getByTestId('tx-category').selectOption('餐飲');
  await page.getByTestId('tx-save').click();
  await expect(page.getByTestId('tx-msg')).toContainText('成功');
  await expect(page.getByTestId('tx-row').filter({ hasText: name })).toBeVisible();
  await expect(page.getByTestId('stats-expense')).not.toBeEmpty();
});

test('E2E-02：冰箱管理', async ({ page }) => {
  const name = `雞蛋-${ts()}`;
  const exp = new Date(Date.now() + 7 * 864e5).toISOString().slice(0, 10);
  await page.goto('/');
  await page.getByRole('link', { name: '我的冰箱' }).click();
  await page.getByTestId('ing-name').fill(name);
  await page.getByTestId('ing-qty').fill('6');
  await page.getByTestId('ing-unit').selectOption('顆');
  await page.getByTestId('ing-expiry').fill(exp);
  await page.getByTestId('ing-save').click();
  await expect(page.getByTestId('ing-msg')).toContainText('成功');
  const row = page.getByTestId('ing-row').filter({ hasText: name });
  await expect(row).toBeVisible();
  await expect(row).toContainText('6顆');
  await expect(row).toContainText(exp);
});
