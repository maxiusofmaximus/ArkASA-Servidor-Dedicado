# 🔧 CORRECCIÓN - ERROR "mod id 0" Y PARÁMETROS FALTANTES

## ❌ Problemas Identificados

### 1. **ERROR: "LoadGameMods with 0 mods"**
- **Causa:** Los mods no se cargan correctamente
- **Razón:** Probablemente hay un valor "0" en la lista de ActiveMods o el formato es incorrecto

### 2. **WARNING: "Could not find mod id 0"**  
- **Causa:** El servidor encuentra un mod ID "0" que no existe
- **Razón:** Puede ser una línea vacía o un valor incorrecto en ActiveMods

### 3. **Parámetros No Referenciados**
- ❌ `DifficultyOffset=2`
- ❌ `DisableCryopodFridgeRequirement=True`
- ❌ `DisableCryopodEnemyCheck=True`
- ❌ `EnableCryoSicknessPVE=False`

**Estado:** ✅ FIJO - Ya agregados a la configuración

---

## ✅ Solución

### PASO 1: Validar el archivo de configuración

Abre PowerShell **como Administrador** y ejecuta:

```powershell
cd C:\Users\Max\ArkASA-Servidor-Dedicado\scripts
.\99_validar_config.ps1
```

Este script te mostrará:
- ✅ Si ActiveMods está bien formado
- ✅ Si hay espacios problemáticos
- ✅ Si hay un "mod id 0"
- ✅ El estado de todos los parámetros

### PASO 2: Si hay problemas, reparar automáticamente

```powershell
.\99_validar_config.ps1 -Repair
```

Esto va a:
- ✅ Hacer backup del archivo actual
- ✅ Eliminar espacios incorrectos en ActiveMods
- ✅ Eliminar el "mod id 0" si existe
- ✅ Guardar la configuración corregida

### PASO 3: Regenerar completamente (Si aún hay problemas)

Si el error persiste después de reparar:

```powershell
cd C:\Users\Max\ArkASA-Servidor-Dedicado
.\DESPLEGAR.ps1
# Selecciona opción 4: Aplicar Configuración
```

Esto regenerará **completo** el archivo con los valores correctos.

---

## 🔍 Qué Buscar en el Log

Después de reiniciar el servidor, ve a:
```
C:\ASA\server\ShooterGame\Saved\Logs\ShooterGame.log
```

### ✅ Deberías ver ESTO:

```
[TIMESTAMP][ XX]UShooterEngine::LoadGameMods with 11 mods
[TIMESTAMP][ XX][OK] Loading mod 955131
[TIMESTAMP][ XX][OK] Loading mod 1102729
...etc...
```

### ❌ Si aún ves ESTO:

```
[TIMESTAMP][ XX]UShooterEngine::LoadGameMods with 0 mods
[TIMESTAMP][ XX]WARNING: Could not find mod id 0 in enabled mods list
```

→ Ejecuta DESPLEGAR.ps1 opción 4 nuevamente

---

## 📋 Qué Se Corrigió

### config-ejemplos/servidor.ps1
✅ Actualizado DifficultyOffset = 2  
✅ Agregado DisableCryopodFridgeRequirement = $true  
✅ Agregado DisableCryopodEnemyCheck = $true  
✅ Agregado EnableCryoSicknessPVE = $false  

### DESPLEGAR.ps1
✅ Función Apply-ServerConfig ahora aplica estos parámetros

---

## 🚀 Ejecución Recomendada

### OPCIÓN A: Rápida (Validar + Reparar)

```powershell
# 1. Validar
.\scripts\99_validar_config.ps1

# 2. Si hay errores, reparar
.\scripts\99_validar_config.ps1 -Repair

# 3. Reiniciar servidor
.\DESPLEGAR.ps1
# Opción 2: Iniciar servidor The Island
```

### OPCIÓN B: Completa (Regenerar todo)

```powershell
# 1. Aplicar configuración nueva
.\DESPLEGAR.ps1
# Opción 4: Aplicar Configuración

# 2. Esperar a que termine

# 3. Reiniciar servidor
.\DESPLEGAR.ps1
# Opción 2: Iniciar servidor The Island
```

---

## 📝 Checklist de Verificación

Después de ejecutar:

- [ ] Ejecuté `scripts\99_validar_config.ps1`
- [ ] Si tuvo errores, ejecuté `scripts\99_validar_config.ps1 -Repair`
- [ ] Reinicié el servidor
- [ ] Busqué en el log "LoadGameMods with 11 mods" ✅
- [ ] NO veo "LoadGameMods with 0 mods" ❌
- [ ] NO veo "WARNING: Could not find mod id 0" ❌
- [ ] Los 11 mods se cargan correctamente ✅

---

## 💡 Si Sigue Sin Funcionar

1. **Verifica manualmente el archivo:**
   ```powershell
   notepad "C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\GameUserSettings.ini"
   ```
   
2. **Busca la línea ActiveMods:**
   - Debe verse: `ActiveMods=955131,1102729,1306435,...` (SIN espacios)
   - NO debe verse: `ActiveMods=0` o `ActiveMods=0,955131...`

3. **Si ves algo mal, reemplázalo:**
   - Busca: `ActiveMods=.*`
   - Reemplaza por: `ActiveMods=955131,1102729,1306435,958001,1182795,932756,930494,1262693,928650,953154,947033`
   - Guarda

4. **Reinicia el servidor y verifica el log**

---

## 📞 Resumen

| Problema | Solución | Comando |
|----------|----------|---------|
| Error "mod id 0" | Validar y reparar | `.\scripts\99_validar_config.ps1 -Repair` |
| Parámetros faltantes | Ya agregados | Ejecuta `DESPLEGAR.ps1` opción 4 |
| Mods aún no cargan | Regenerar config | `DESPLEGAR.ps1` → opción 4 |

---

**Estado:** ✅ Correcciones aplicadas  
**Próximo paso:** Ejecuta `.\scripts\99_validar_config.ps1` para diagnosticar el problema exacto
