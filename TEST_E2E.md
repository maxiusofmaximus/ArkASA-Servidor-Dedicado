# Playwright E2E Tests

Configuración de tests E2E con Playwright para ARK ASA Config.

## Instalación

Las dependencias ya están en `package.json`. Instálalas:

```bash
npm install
# o
pnpm install
```

## Archivos de configuración

- `playwright.config.ts` - Configuración principal
- `tests/e2e/` - Directorio con los tests

## Ejecutar tests

### Modo headless (sin interfaz)
```bash
npm run test:e2e
```

### Con interfaz gráfica
```bash
npm run test:e2e:headed
```

### Modo interactivo (debug)
```bash
npm run test:e2e:debug
```

### Con UI de Playwright
```bash
npm run test:e2e:ui
```

### Tests específicos
```bash
npx playwright test tests/e2e/example.spec.ts
npx playwright test --grep "should load the app"
```

## Browserstack (opcional - si quieres tests en navegadores reales)

```bash
npx playwright test --config=playwright.config.ts
```

## Estructura de tests

```
tests/
└── e2e/
    ├── example.spec.ts      (tests de ejemplo)
    ├── status.spec.ts       (tests de status)
    ├── settings.spec.ts     (tests de configuración)
    └── ...
```

## Tips

- Los tests esperan que la app esté corriendo en `http://localhost:1420`
- El archivo `playwright.config.ts` inicia automáticamente `npm run tauri:dev`
- Los screenshots de fallos se guardan en `test-results/`
- Los reports HTML se guardan en `playwright-report/`

## Documentación

- [Playwright Docs](https://playwright.dev)
- [Testing Best Practices](https://playwright.dev/docs/best-practices)
