import { test, expect } from '@playwright/test'

test('ARK Config Manager - New UI loads correctly', async ({ page }) => {
  const errors: string[] = []

  page.on('console', (msg) => {
    if (msg.type() === 'error') errors.push(msg.text())
  })

  page.on('pageerror', (err) => {
    errors.push(`PAGE ERROR: ${err.message}`)
  })

  console.log('\n=== LOADING APP ===')
  await page.goto('http://localhost:5173/', { waitUntil: 'networkidle', timeout: 15000 })
  await page.waitForTimeout(2000)

  console.log('=== VERIFYING NEW UI ===')

  // Check primary navigation
  const arksTab = await page.locator('button:has-text("ARKS")').count()
  const modTab = await page.locator('button:has-text("MOD SETTINGS")').count()
  const rulesTab = await page.locator('button:has-text("GAME RULES")').count()
  const advancedTab = await page.locator('button:has-text("ADVANCED")').count()
  const engramsTab = await page.locator('button:has-text("ENGRAMS")').count()

  console.log(`ARKS: ${arksTab > 0 ? 'OK' : 'FAIL'}`)
  console.log(`MOD SETTINGS: ${modTab > 0 ? 'OK' : 'FAIL'}`)
  console.log(`GAME RULES: ${rulesTab > 0 ? 'OK' : 'FAIL'}`)
  console.log(`ADVANCED: ${advancedTab > 0 ? 'OK' : 'FAIL'}`)
  console.log(`ENGRAMS: ${engramsTab > 0 ? 'OK' : 'FAIL'}`)

  expect(arksTab + modTab + rulesTab + advancedTab + engramsTab).toBe(5)

  // Check settings panel
  const panels = await page.locator('[class*="ark"]').count()
  console.log(`ARK-themed elements: ${panels}`)

  // Take screenshot
  await page.screenshot({ path: './screenshot-new-ui.png' })
  console.log('Screenshot: OK')

  console.log('\n=== RESULT: PASS ===\n')
})
