import { expect, test } from '@playwright/test';

test('requires first-launch disclaimer acceptance and remembers it', async ({ page }) => {
  await page.goto('/');
  await page.evaluate(() => window.__AUTODAILY_MOCK__?.seed({ store: {} }));
  await page.reload();

  const dialog = page.getByRole('dialog', { name: '使用 AutoDaily 前请先确认' });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByText('运行自动化脚本可能违反第三方服务规则')).toBeVisible();
  const taskManagementLink = page.getByRole('link', { name: '任务管理' });
  await expect(taskManagementLink).toHaveCount(0);

  await dialog.getByRole('button', { name: '我已阅读并同意' }).click();
  await expect(dialog).toBeHidden();
  await page.reload();
  await expect(dialog).toBeHidden();
  await expect(taskManagementLink).toBeVisible();
});

test('requests application exit when disclaimer is declined', async ({ page }) => {
  await page.goto('/');
  await page.evaluate(() => window.__AUTODAILY_MOCK__?.seed({ store: {} }));
  await page.reload();

  const dialog = page.getByRole('dialog', { name: '使用 AutoDaily 前请先确认' });
  await dialog.getByRole('button', { name: '不同意并退出' }).click();

  await expect
    .poll(() => page.evaluate(() => window.__AUTODAILY_MOCK__?.getState().exitRequested))
    .toBe(true);
});
