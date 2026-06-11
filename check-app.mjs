import { chromium } from '@playwright/test';

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  
  // Capturar todos los logs de la consola
  const consoleLogs = [];
  page.on('console', msg => {
    consoleLogs.push({
      type: msg.type(),
      text: msg.text()
    });
  });
  
  // Capturar errores no capturados
  page.on('pageerror', error => {
    console.log('PAGE ERROR:', error.message);
  });

  console.log('Navegando a http://localhost:5173/...');
  await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
  
  console.log('\n=== CONSOLE LOGS ===');
  consoleLogs.forEach(log => {
    const color = log.type === 'error' ? '\x1b[31m' : log.type === 'warn' ? '\x1b[33m' : '\x1b[0m';
    console.log(`${color}[${log.type.toUpperCase()}] ${log.text}\x1b[0m`);
  });
  
  console.log('\n=== ESTADO DE LA PÁGINA ===');
  const html = await page.content();
  if (html.includes('Failed to load config')) {
    console.log('✗ ERROR EN LA PÁGINA: "Failed to load config"');
  }
  if (html.includes('Cannot read properties')) {
    console.log('✗ ERROR: "Cannot read properties"');
  }
  
  // Revisar si hay elementos de UI
  const formExists = await page.locator('form').count() > 0;
  console.log(`Form: ${formExists ? '✓' : '✗'}`);
  
  const logsButton = await page.locator('button:has-text("Show Logs")').count() > 0;
  console.log(`Show Logs button: ${logsButton ? '✓' : '✗'}`);
  
  await browser.close();
})();
