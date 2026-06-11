# 🎯 PRÓXIMOS PASOS - DESPUÉS DE LAS CORRECCIONES

**Importante:** Todas las correcciones técnicas ya están hechas. Solo necesitas ejecutar el script.

---

## 📌 REQUERIMIENTOS

Antes de empezar:
- ✅ PowerShell 5.1+ (incluido en Windows)
- ✅ Permisos de Administrador
- ✅ Espacio en disco (para servidor ASA + mods)

---

## 🚀 EJECUCIÓN RÁPIDA (3 PASOS)

### **PASO 1️⃣: Copiar Configuración** ← IMPORTANTE

Abre **PowerShell como Administrador** en la carpeta del proyecto:

```powershell
# Navega a la carpeta del proyecto
cd C:\Users\Max\ArkASA-Servidor-Dedicado

# Copia la configuración actualizada
Copy-Item config-ejemplos\servidor.ps1 config\servidor.ps1
```

✅ Esto asegura que tienes la configuración más reciente con los mods y spawn duplicado.

---

### **PASO 2️⃣: Ejecutar el Despliegue**

En la misma ventana de PowerShell:

```powershell
.\DESPLEGAR.ps1
```

Verás un menú como este:
```
╔══════════════════════════════════════════════════════╗
║      MENÚ DE DESPLIEGUE - ARK: SURVIVAL ASCENDED     ║
╚══════════════════════════════════════════════════════╝

1. Instalación Completa (SteamCMD + Servidor + Configuración)
2. Iniciar servidor The Island
3. Hacer backup de Saved
4. Aplicar Configuración
5. Descargar/actualizar mods
6. Configurar firewall
7. Diagnóstico
8. Salir

¿Selecciona opción: _
```

### **OPCIÓN 1: Instalación Completa** (Recomendado)
```
Selecciona: 1
```
Esto:
- Descarga SteamCMD
- Instala el servidor ASA
- Descarga los 11 mods
- Aplica toda la configuración
- Configura firewall

⏱️ **Tiempo:** 30-60 minutos (depende de tu conexión)

### **O OPCIÓN 4: Solo Aplicar Configuración** (Si ya tienes servidor)
```
Selecciona: 4
```
Si ya tienes el servidor instalado y solo necesitas aplicar la configuración actualizada.

---

### **PASO 3️⃣: Verificar que Funciona**

Una vez que termina, **abre el log del servidor**:

```
C:\ASA\server\ShooterGame\Saved\Logs\ShooterGame.log
```

En el archivo, busca `LoadGameMods` (Ctrl+F):

```
✅ [OK] UShooterEngine::LoadGameMods with 11 mods
   [OK] Loading mod 955131 (TSSEssentials)
   [OK] Loading mod 1102729 (Immersive Vehicles)
   [OK] Loading mod 1306435 (Advanced Structures)
   ...y otros 8 más...
```

**Si ves esto → ✅ Los mods están cargando correctamente**

---

## 🔍 VALIDACIÓN COMPLETA (Avanzado)

Si quieres verificar TODO está correcto:

### **1. Ver la configuración escrita**

```powershell
# Abre el archivo de configuración
notepad "C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\GameUserSettings.ini"
```

Busca estas líneas:

```ini
[ServerSettings]
...
; Mods
ActiveMods=955131,1102729,1306435,958001,1182795,932756,930494,1262693,928650,953154,947033
...
; Spawn de dinos
DinoCountMultiplier=2.0
```

✅ **Verificar:**
- ✅ `ActiveMods=...` **SIN espacios** después de comas
- ✅ `DinoCountMultiplier=2.0` presente
- ✅ Está en sección `[ServerSettings]`

### **2. Iniciar el servidor manualmente**

```powershell
C:\ASA\server\ShooterGame\Binaries\Win64\ArkAscendedServer.exe TheIsland_WP
```

O usa opción 2 del menú:
```
Selecciona: 2
```

Espera a que aparezca:
```
[OK] UShooterEngine::LoadGameMods with 11 mods
```

### **3. Entrar al servidor y verificar**

Una vez en el servidor (conecta con Steam):

En consola RCON o admin commands:
```
cheat GetGameMode Config.CustomGameModeVariableValues
```

Deberías ver:
```
DinoCountMultiplier=2.0
TamingSpeedMultiplier=15
...y otros parámetros
```

### **4. Verificar spawn duplicado**

- Entra al servidor
- Ve a una zona con dinosaurios (ej: The Beach, The Riverlands)
- Compara la cantidad de dinos:
  - Deberían haber **más del doble** de dinos comparado con el spawn normal

---

## ⚠️ SI ALGO SALE MAL

### **Problema: "LoadGameMods with 0 mods"**

Este es el error que solucionamos. Si aún lo ves:

1. **Cierra el servidor completamente**
2. **Abre manualmente el archivo de configuración:**
   ```powershell
   notepad "C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\GameUserSettings.ini"
   ```

3. **Busca la línea `ActiveMods`:**
   ```ini
   ❌ MALO:    ActiveMods=955131, 1102729  # Hay espacios
   ✅ CORRECTO: ActiveMods=955131,1102729  # Sin espacios
   ```

4. **Si hay espacios:**
   - Usa Find & Replace (Ctrl+H)
   - Buscar: `ActiveMods=955131, 1102729` (con espacios)
   - Reemplazar: `ActiveMods=955131,1102729` (sin espacios)
   - Guardar

5. **Reinicia el servidor**

---

### **Problema: Spawn Normal (No Duplicado)**

Si no ves el doble de dinos:

1. **Verifica que `DinoCountMultiplier=2.0`** esté en el archivo
2. **Reinicia completamente el servidor** (no solo reload)
3. **Espera 5 minutos** en el servidor para que spawn complete

---

### **Problema: Script no ejecuta**

```powershell
# Si sale error sobre ejecución de scripts:
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# Luego intenta de nuevo:
.\DESPLEGAR.ps1
```

---

## 📊 RESUMEN DE ACCIONES

| Acción | Comando | Resultado |
|--------|---------|-----------|
| **Copiar config** | `Copy-Item config-ejemplos\servidor.ps1 config\servidor.ps1` | ✅ Config actualizada |
| **Ejecutar script** | `.\DESPLEGAR.ps1` → Opción 1 o 4 | ✅ Instalación/Aplicación |
| **Ver mods cargados** | Buscar en ShooterGame.log "LoadGameMods" | ✅ 11 mods |
| **Ver spawn config** | Buscar en GameUserSettings.ini "DinoCountMultiplier" | ✅ 2.0 |
| **Entrar servidor** | Conectar por Steam | ✅ Todo funciona |

---

## ✅ CHECKLIST FINAL

Después de ejecutar:

- [ ] Ejecuté `Copy-Item config-ejemplos\servidor.ps1 config\servidor.ps1`
- [ ] Ejecuté `DESPLEGAR.ps1` y seleccioné opción 1 o 4
- [ ] El script terminó sin errores críticos
- [ ] Verifiqué en ShooterGame.log "LoadGameMods with 11 mods"
- [ ] Abrí GameUserSettings.ini y vi `ActiveMods=...` sin espacios
- [ ] Abrí GameUserSettings.ini y vi `DinoCountMultiplier=2.0`
- [ ] Inicié el servidor sin errores
- [ ] Me conecté al servidor desde Steam
- [ ] Vi más dinos de lo normal (spawn duplicado)
- [ ] Ejecuté `cheat GetGameMode Config.CustomGameModeVariableValues` y vi los parámetros

---

## 📞 SI NECESITAS AYUDA

Si después de todo esto sigue sin funcionar:

1. **Adjunta el contenido de:**
   - `C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\GameUserSettings.ini`
   - `C:\ASA\server\ShooterGame\Saved\Logs\ShooterGame.log` (últimas 50 líneas)

2. **Describe:**
   - Exactamente qué error ves
   - En qué paso fallé
   - Qué intentaste

---

## 🎮 ¡LISTO!

Una vez verifiques que todo funciona:

- ✅ Los 11 mods están cargados
- ✅ El spawn de dinos es 2x
- ✅ Todas las configuraciones aplicadas

**¡Ya está listo tu servidor personalizado!**

---

**Última actualización:** 2026-06-09  
**Documentos relacionados:** `CORRECCIONES_REALIZADAS.md`, `GUIA_VALIDACION_MODS.md`, `ANALISIS_ERRORES_OFICIAL.md`
