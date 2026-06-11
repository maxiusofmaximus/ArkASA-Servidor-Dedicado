# ✅ VALIDACIÓN DE CORRECCIONES - GUÍA PASO A PASO

## 🎯 Objetivo

Validar que los **mods se cargan correctamente** (sin error "LoadGameMods with 0 mods") y que el **spawn de dinos está duplicado (2x)**.

---

## 📋 PASO 1: Copiar Configuración (RECOMENDADO)

```powershell
# Abre una terminal de PowerShell en la carpeta del proyecto
Copy-Item config-ejemplos\servidor.ps1 config\servidor.ps1
```

**Esto asegura que tienes la configuración más actualizada.**

---

## 📋 PASO 2: Ejecutar el Despliegue

```powershell
# Ejecuta esto en PowerShell como Administrador
.\DESPLEGAR.ps1

# Selecciona opción: 1 (Instalación Completa)
# O selecciona opción: 4 (Aplicar Configuración)
```

**Esto escribirá los archivos de configuración con los valores correctos.**

---

## 📋 PASO 3: Verificar el Archivo GameUserSettings.ini

### ✅ Buscar línea de ActiveMods

```powershell
# Abre el archivo:
C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\GameUserSettings.ini

# Presiona Ctrl+F para buscar: "ActiveMods"

# Debe verse EXACTAMENTE así (sin espacios después de comas):
[ServerSettings]
...
ActiveMods=955131,1102729,1306435,958001,1182795,932756,930494,1262693,928650,953154,947033
...
```

### ⚠️ ERRORES COMUNES a Evitar

```ini
❌ MALO:      ActiveMods=955131, 1102729, 1306435  # Espacios = ERROR
❌ MALO:      ActiveMods = 955131,1102729          # Espacios alrededor del =
❌ MALO:      ActiveMods=955131;1102729            # Punto y coma
✅ CORRECTO:  ActiveMods=955131,1102729,1306435    # Sin espacios
```

### ✅ Verificar DinoCountMultiplier

En el mismo archivo, busca `DinoCountMultiplier`:

```ini
[ServerSettings]
...
DinoCountMultiplier=2.0
...
```

**Esto significa:** Doble cantidad de dinos en el mundo

---

## 📋 PASO 4: Iniciar el Servidor y Revisar Logs

```powershell
# Inicia el servidor con:
.\DESPLEGAR.ps1
# Opción 2 (Iniciar servidor The Island)

# O manualmente:
C:\ASA\server\ShooterGame\Binaries\Win64\ArkAscendedServer.exe TheIsland_WP
```

### 📜 Revisar el Log del Servidor

```powershell
# Abre el archivo de log:
C:\ASA\server\ShooterGame\Saved\Logs\ShooterGame.log

# Busca estas líneas (Ctrl+F):
"LoadGameMods"

# Deberías ver:
✅ [OK] UShooterEngine::LoadGameMods with 11 mods
   [OK] Loading mod 955131
   [OK] Loading mod 1102729
   [OK] Loading mod 1306435
   ...
```

### ❌ Si ves esto = ERROR

```
[ERROR] UShooterEngine::LoadGameMods with 0 mods
```

**Esto significa:**
- Los mods NO se están cargando
- Revisa que `ActiveMods=...` NO tiene espacios
- Revisa que está en la sección `[ServerSettings]`
- Revisa que el formato es exacto: `ActiveMods=ID1,ID2,ID3`

---

## 📊 PASO 5: Validar Spawn de Dinos

### En el Servidor

```
# En la consola del servidor (RCON), ejecuta:
cheat GetGameMode Config.CustomGameModeVariableValues

# Deberías ver algo como:
DinoCountMultiplier=2.0
```

### En el Mundo del Servidor

- Entra al servidor
- Explora diferentes zonas
- Verifica que hay **el doble de dinos** comparado con spawn normal
- Los dinos deben aparece más frecuentemente

---

## 🔍 PASO 6: Verificar Todos los Parámetros (Avanzado)

### Parámetros Críticos de Mods

```ini
[ServerSettings]
; DEBE estar presente:
ActiveMods=955131,1102729,1306435,958001,1182795,932756,930494,1262693,928650,953154,947033
```

### Parámetros de Spawn

```ini
[ServerSettings]
; DEBE estar presente:
DinoCountMultiplier=2.0
```

### Otros parámetros que deben estar (Game.ini)

```ini
[/Script/ShooterGame.ShooterGameMode]
BabyMatureSpeedMultiplier=40
EggHatchSpeedMultiplier=20
TamingSpeedMultiplier=15
HarvestAmountMultiplier=8
XpMultiplier=3
```

---

## ✅ LISTA DE VERIFICACIÓN FINAL

| Paso | Elemento | Estado | Notas |
|------|----------|--------|-------|
| 1 | Copiar config/servidor.ps1 | ⬜ | `Copy-Item config-ejemplos\servidor.ps1 config\servidor.ps1` |
| 2 | Ejecutar DESPLEGAR.ps1 | ⬜ | Opción 1 o 4 |
| 3 | ActiveMods sin espacios | ⬜ | Formato: `ID1,ID2,ID3` |
| 4 | DinoCountMultiplier=2.0 | ⬜ | En GameUserSettings.ini |
| 5 | Log del servidor sin errores | ⬜ | "with 11 mods" (no "with 0 mods") |
| 6 | Servidor inicia correctamente | ⬜ | Sin errores de configuración |
| 7 | Mods activos en el servidor | ⬜ | Verificable con comandos/console |
| 8 | Spawn duplicado visible | ⬜ | Más dinos en el mundo |

---

## 🚨 TROUBLESHOOTING

### Problema: "LoadGameMods with 0 mods"

**Causas posibles:**

1. **Espacios en ActiveMods** (MÁS COMÚN)
   - ❌ `ActiveMods=955131, 1102729` (espacio después de coma)
   - ✅ `ActiveMods=955131,1102729` (sin espacios)

2. **Sección incorrecta**
   - ❌ `[GameSettings]` (sección incorrecta)
   - ✅ `[ServerSettings]` (sección correcta)

3. **Mods no descargados**
   - El servidor puede no haber descargado los archivos de mods
   - Intenta ejecutar el script con opción 8 "Descargar/actualizar mods"

4. **Formato incorrecto**
   - ❌ `ActiveMods = 955131,1102729` (espacios alrededor del =)
   - ❌ `ActiveMods=955131;1102729` (punto y coma)
   - ✅ `ActiveMods=955131,1102729` (exacto)

### Solución Rápida

```powershell
# 1. Abre el archivo manualmente:
notepad "C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\GameUserSettings.ini"

# 2. Busca y reemplaza (Ctrl+H):
# Buscar:  "ActiveMods=955131, 1102729" (con espacios)
# Sustituir por: "ActiveMods=955131,1102729" (sin espacios)

# 3. Guarda y reinicia el servidor
```

---

### Problema: Spawn Normal (No Duplicado)

**Causas posibles:**

1. **DinoCountMultiplier no está en 2.0**
   - Verifica que está en `[ServerSettings]` de `GameUserSettings.ini`
   - Valor debe ser: `DinoCountMultiplier=2.0`

2. **Servidor no recargó configuración**
   - Reinicia el servidor completamente (no solo reload)
   - Elimina `ShooterGame/Saved/` y redeploy (copia de seguridad primero)

3. **Otros multiplicadores están bajando spawn**
   - Algunos parámetros pueden reducir spawn (ej: ResourceNoReplenishRadiusPlayers)
   - Revisa el archivo de configuración para valores conflictivos

---

## 📞 CONTACTO / SOPORTE

Si después de seguir estos pasos aún tienes problemas:

1. **Copia el contenido de los archivos:**
   - `GameUserSettings.ini`
   - `Game.ini`
   - `ShooterGame.log` (últimas 100 líneas)

2. **Documenta:**
   - Exactamente qué error ves
   - Qué pasos seguiste
   - Qué intentaste para solucionarlo

---

**✅ Versión:** 1.0  
**📅 Actualizado:** 2026-06-09  
**📌 Estado:** Todas las correcciones implementadas según fuentes oficiales
