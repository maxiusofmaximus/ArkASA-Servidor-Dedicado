import { chromium } from '@playwright/test';

(async () => {
  console.log('Abriendo navegador...');
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();
  
  const consoleLogs = [];
  const pageErrors = [];
  
  page.on('console', msg => {
    consoleLogs.push({ type: msg.type(), text: msg.text() });
  });
  
  page.on('pageerror', error => {
    pageErrors.push(error.message);
  });

  console.log('\nNavegando a http://localhost:5173/...');
  try {
    await page.goto('http://localhost:5173/', { waitUntil: 'networkidle', timeout: 10000 });
  } catch (e) {
    console.log('Error navigating:', e.message);
  }
  
  await page.waitForTimeout(2000);
  
  console.log('\n=== ERRORES DE LA PÁGINA ===');
  if (pageErrors.length > 0) {
    pageErrors.forEach(err => console.log('✗', err));
  } else {
    console.log('✓ Sin errores no capturados');
  }
  
  console.log('\n=== CONSOLE (últimos 15) ===');
  consoleLogs.slice(-15).forEach(log => {
    const color = log.type === 'error' ? '\x1b[31m' : log.type === 'warn' ? '\x1b[33m' : '\x1b[0m';
    console.log(`${color}[${log.type.toUpperCase()}]${'\x1b[0m'} ${log.text}`);
  });
  
  console.log('\n=== ESTADO DEL DOM ===');
  const title = await page.title();
  console.log('Title:', title);
  
  const errorText = await page.locator('body').evaluate(el => el.textContent);
  if (errorText.includes('Failed to load config')) {
    console.log('✗ ENCONTRADO: "Failed to load config"');
  }
  if (errorText.includes('Cannot read properties')) {
    console.log('✗ ENCONTRADO: "Cannot read properties"');
  }
  if (errorText.includes('Activar Windows')) {
    console.log('✗ Parece ser la ventana Tauri');
  }
  
  const formCount = await page.locator('form').count();
  console.log('Forms encontradas:', formCount);
  
  const buttonCount = await page.locator('button').count();
  console.log('Buttons encontradas:', buttonCount);
  
  const logsButtonCount = await page.locator('button:has-text("Show Logs")').count();
  console.log('Show Logs button:', logsButtonCount > 0 ? '✓' : '✗');
  
  await browser.close();
  console.log('\n✓ Verificación completada');
})();
