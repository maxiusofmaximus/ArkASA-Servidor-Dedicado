import { test } from "@playwright/test";

test("Capture UI with logo", async ({ page }) => {
  await page.goto("http://localhost:5173/", { waitUntil: "networkidle" });
  await page.waitForTimeout(3000);
  await page.screenshot({ path: "real-screenshots/01-with-logo.png", fullPage: true });
});
