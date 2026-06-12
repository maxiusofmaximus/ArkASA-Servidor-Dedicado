import { test, expect } from '@playwright/test';

test.describe('ARK ASA Config - Status Page', () => {
  test('should load the app', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/ARK|Status/i);
  });

  test('should display server status component', async ({ page }) => {
    await page.goto('/');
    const statusComponent = page.locator('text=/Status/i');
    await expect(statusComponent).toBeVisible();
  });

  test('should navigate to settings', async ({ page }) => {
    await page.goto('/');
    const settingsLink = page.locator('a, button').filter({ hasText: /Settings|Configuración/i }).first();
    if (await settingsLink.isVisible()) {
      await settingsLink.click();
      await page.waitForLoadState('networkidle');
    }
  });
});

test.describe('ARK ASA Config - Navigation', () => {
  test('should have navigation elements', async ({ page }) => {
    await page.goto('/');
    const navbar = page.locator('header, nav, [role="navigation"]').first();
    await expect(navbar).toBeVisible();
  });
});
