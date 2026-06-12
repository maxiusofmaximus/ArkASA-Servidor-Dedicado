import { test } from "@playwright/test";

test("Capture with logo", async ({ page }) => {
  await page.goto("http://localhost:5173/", { waitUntil: "networkidle" });
  await page.waitForTimeout(2000);
  await page.screenshot({ path: "real-screenshots/01-arks-with-logo.png", fullPage: true });
  console.log("✓ Screenshot 1");

  await page.click("button:nth-of-type(3)");
  await page.waitForTimeout(500);
  await page.screenshot({ path: "real-screenshots/02-game-rules-logo.png", fullPage: true });
  console.log("✓ Screenshot 2");

  await page.click("button:nth-of-type(4)");
  await page.waitForTimeout(500);
  await page.screenshot({ path: "real-screenshots/03-advanced-logo.png", fullPage: true });
  console.log("✓ Screenshot 3");
});
