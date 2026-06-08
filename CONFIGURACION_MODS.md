# Configuración de Mods y Servidor ARK

## ¿Qué cambié?

Se han realizado las siguientes actualizaciones para habilitar mods automáticamente:

### 1. **DESPLEGAR.ini** (NUEVO)
Archivo de configuración centralizado que contiene TODAS las variables del servidor:
- Configuración básica del servidor
- Multiplicadores de gameplay
- IDs de mods a instalar
- Configuración de stats por nivel
- Y muchas más...

### 2. **config-ejemplos/servidor.ps1** (ACTUALIZADO)
Ahora incluye:
- Parámetro `ActiveMods` con los IDs de tus mods
- TODAS las variables de configuración del DESPLEGAR.ini
- Multiplicadores de stats completos

### 3. **DESPLEGAR.ps1** (ACTUALIZADO)
Nuevas funcionalidades:
- Función `Install-Mods` que descarga mods automáticamente
- Parámetro `ActiveMods` agregado a GameUserSettings.ini
- Nueva opción "8. Descargar/actualizar mods" en el menú
- Los mods se descargan automáticamente al hacer un despliegue completo

## Cómo Configurar los Mods

### Opción 1: Usando config\servidor.ps1 (Recomendado)

1. **Copia el archivo de ejemplo:**
   ```
   Copy-Item config-ejemplos\servidor.ps1 config\servidor.ps1 -Force
   ```

2. **Edita `config\servidor.ps1`** y localiza la línea:
   ```powershell
   ActiveMods = '955131,1102729,1306435,958001,1182795,932756,930494,1262693,928650,953154,947033'
   ```

3. **Reemplaza los IDs con tus mods** (separados por comas):
   ```powershell
   ActiveMods = 'ID_MOD_1,ID_MOD_2,ID_MOD_3'
   ```

### Opción 2: Usando DESPLEGAR.ini

1. **Edita `DESPLEGAR.ini`** y localiza la sección `[MODS]`:
   ```ini
   [MODS]
   ActiveMods=955131,1102729,1306435,958001,1182795,932756,930494,1262693,928650,953154,947033
   ```

2. **Reemplaza los IDs con tus mods** (separados por comas)

## Cómo Descargar e Instalar Mods

### Método 1: Despliegue Completo (Lo más fácil)
```
Ejecuta DESPLEGAR.bat → Opción 1 → Desplegar todo lo posible ahora
```
Esto instalará:
- ✓ SteamCMD (si no existe)
- ✓ Servidor ARK
- ✓ **Todos los mods listados en ActiveMods**
- ✓ Configuración
- ✓ Firewall

### Método 2: Solo Descargar Mods
```
Ejecuta DESPLEGAR.bat → Opción 8 → Descargar/actualizar mods
```

### Método 3: Desde PowerShell
```powershell
.\DESPLEGAR.ps1
# Luego selecciona opción 8
```

## Qué Son los IDs de Mods

Los IDs de mods son números únicos de **CurseForge** que identifican cada mod:

- **955131** - Ejemplo de un mod
- **1102729** - Ejemplo de otro mod
- etc.

**¿Dónde encontrar IDs de mods?**
1. Abre https://www.curseforge.com/ark-survival-ascended/mods
2. Busca un mod que te interese
3. Abre el mod → la URL mostrará el ID
   - Ejemplo: `...mod/123456/...` → ID es `123456`

## Estructura de Archivos de Configuración

Los mods se descargan en:
```
C:\ASA\server\ShooterGame\Mods\
├── 955131\
├── 1102729\
├── 1306435\
└── ...
```

El archivo de configuración del servidor usa:
```
C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\
├── GameUserSettings.ini    (contiene ActiveMods)
└── Game.ini                (contiene Game Settings)
```

## Verificar que los Mods Están Habilitados

1. **En GameUserSettings.ini:**
   ```ini
   ActiveMods=955131,1102729,1306435,958001,1182795,932756,930494,1262693,928650,953154,947033
   ```

2. **En el log del servidor (después de iniciar):**
   ```
   UShooterEngine::LoadGameMods with 11 mods
   ```
   (Debería mostrar el número de mods, no 0)

## Solucionar Problemas

### Los mods no se descargan
- Asegúrate de tener espacio disco disponible
- Verifica los IDs de mods en DESPLEGAR.ini o config\servidor.ps1
- Los IDs deben estar separados por comas sin espacios

### Los mods se descargan pero no se cargan
- Verifica que `ActiveMods` esté en `GameUserSettings.ini`
- Los mods pueden ser incompatibles con tu versión de ARK
- Algunos mods pueden requerir otros mods como dependencias

### El servidor no inicia
- Revisa el log del servidor en `C:\ASA\server\ShooterGame\Saved\Logs\`
- Los mods pueden estar corruptos - intenta descargarlos de nuevo
- Verifica que los IDs de mods sean válidos

## Comandos Útiles

### Ver qué mods están instalados:
```powershell
Get-ChildItem "C:\ASA\server\ShooterGame\Mods" -Directory
```

### Actualizar todos los mods:
Ejecuta DESPLEGAR.bat → Opción 8

### Limpiar mods viejos:
```powershell
Remove-Item "C:\ASA\server\ShooterGame\Mods" -Recurse -Force
```
Luego descarga nuevamente con la opción 8.

## Resumen Rápido

1. Edita `config\servidor.ps1` con tus IDs de mods
2. Ejecuta `DESPLEGAR.bat`
3. Selecciona opción `1` para despliegue completo
4. Los mods se descargarán automáticamente
5. ¡Listo! El servidor estará listo con mods habilitados
