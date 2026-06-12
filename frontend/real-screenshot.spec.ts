import { test } from "@playwright/test";

test("Capture real UI screenshots", async ({ page }) => {
  await page.goto("http://localhost:5173/", { waitUntil: "networkidle" });
  await page.waitForTimeout(2000);

  // Screenshot 1: ARKS tab
  await page.screenshot({ path: "real-screenshots/01-arks-ui.png", fullPage: true });
  console.log("✓ Screenshot 1: ARKS tab");

  // Screenshot 2: GAME RULES
  await page.click("button:has-text(\"GAME RULES\")");
  await page.waitForTimeout(500);
  await page.screenshot({ path: "real-screenshots/02-game-rules-ui.png", fullPage: true });
  console.log("✓ Screenshot 2: GAME RULES");

  // Screenshot 3: ADVANCED
  await page.click("button:has-text(\"ADVANCED\")");
  await page.waitForTimeout(500);
  await page.screenshot({ path: "real-screenshots/03-advanced-ui.png", fullPage: true });
  console.log("✓ Screenshot 3: ADVANCED");

  // Screenshot 4: MOD SETTINGS
  await page.click("button:has-text(\"MOD SETTINGS\")");
  await page.waitForTimeout(500);
  await page.screenshot({ path: "real-screenshots/04-mods-ui.png", fullPage: true });
  console.log("✓ Screenshot 4: MOD SETTINGS");

  // Screenshot 5: ENGRAMS
  await page.click("button:has-text(\"ENGRAMS\")");
  await page.waitForTimeout(500);
  await page.screenshot({ path: "real-screenshots/05-engrams-ui.png", fullPage: true });
  console.log("✓ Screenshot 5: ENGRAMS");
});
