# 🔧 Solución de Problemas - ARK ASA Configuration Manager

> **Guía paso a paso para arreglar los problemas más comunes**

---

## 📋 Índice de problemas

1. [La app no abre](#la-app-no-abre)
2. [El servidor no arranca](#el-servidor-no-arranca)
3. [Amigos no pueden entrar](#amigos-no-pueden-entrar)
4. [Timeout / Conexión rechazada](#timeout--conexión-rechazada)
5. [El servidor va lento / Lag](#el-servidor-va-lento--lag)
6. [Error al guardar configuración](#error-al-guardar-configuración)
7. [Los mods no cargan](#los-mods-no-cargan)
8. [El servidor se cierra de repente (Crash)](#el-servidor-se-cierra-de-repente-crash)
9. [No puedo cambiar de mapa](#no-puedo-cambiar-de-mapa)
10. [Los logs están vacíos](#los-logs-están-vacíos)

---

## La app no abre

### Síntomas
- Haces doble clic en el icono y nada pasa
- O ves un error tipo "Application Error"

### Soluciones

**Paso 1: ¿Está instalada?**
1. Ve a `C:\Program Files\`
2. Busca carpeta `ARK ASA Config Manager` o similar
3. Si no existe, descarga el instalador `.exe` de GitHub Releases
4. Ejecuta el instalador nuevamente

**Paso 2: Reinicia tu PC**
1. A veces Windows guarda archivos en memoria que interfieren
2. Apaga completamente tu PC
3. Espera 10 segundos
4. Enciende de nuevo
5. Intenta abrir la app

**Paso 3: Verifica permiso de administrador**
1. Haz **clic derecho** en el icono
2. "Propiedades"
3. "Advanced"
4. ☑️ Marca "Run as administrator"
5. Haz clic en "Apply" → OK
6. Abre la app nuevamente

**Paso 4: Desinstala y reinstala**
1. Panel de Control → Programas → Desinstalar un programa
2. Busca "ARK ASA Config Manager"
3. Haz clic derecho → Desinstalar
4. Espera a que termine
5. Descarga el instalador `.exe` nuevamente
6. Ejecuta

**Paso 5: Revisa antivirus/firewall**
1. Algunos antivirus bloquean apps nuevas
2. Ve a tu antivirus (Defender, AVG, Norton, etc.)
3. Busca "Aplicaciones permitidas"
4. Agrega la app a la whitelist/permitidas

---

## El servidor no arranca

### Síntomas
- Haces clic en "START SERVER"
- Se abre una consola negra pero:
  - Se cierra inmediatamente, O
  - Dice "ERROR", O
  - Se queda congelada sin hacer nada

### Soluciones

**Paso 1: ¿Bajaste el servidor?**

El servidor ARK es enorme (~100 GB) y debe descargarse antes.

1. Abre CMD como administrador:
   - Presiona **Windows + R** → `cmd` → OK
   - O busca "Command Prompt" en Inicio

2. Copia y pega:
```
C:\ASA\steamcmd\steamcmd.exe +force_install_dir C:\ASA\server +login anonymous +app_update 2430930 validate +quit
```

3. Presiona Enter
4. **ESPERA 30-60 minutos** (depende del internet)
5. No cierres la ventana
6. Cuando veas "App '2430930' fully installed" = ✅ Terminó

7. Intenta iniciar el servidor nuevamente en la app

**Paso 2: ¿Tienes espacio suficiente?**

ARK necesita ~150 GB:
- 100 GB para el servidor
- 50 GB para el servidor ejecutándose

1. Abre "This PC" (Este equipo)
2. Haz clic derecho en C:\ → "Properties"
3. Mira "Free space" (Espacio libre)
4. Si hay menos de 150 GB, borra algo o agrega disco

**Paso 3: ¿Los puertos están disponibles?**

Otro programa podría estar usando 7777 o 27015.

1. Abre CMD como administrador
2. Copia:
```
netstat -ano | findstr :7777
netstat -ano | findstr :27015
```

3. Si hay resultados con PID, esos puertos están ocupados
4. Solución: En la app, cambia los puertos a números distintos:
   - Game Port: `7779` (en vez de 7777)
   - Query Port: `27017` (en vez de 27015)
5. Intenta iniciar nuevamente

**Paso 4: ¿Suficiente RAM?**

ARK necesita mínimo 16 GB, idealmente 32 GB.

1. Abre Task Manager (Ctrl + Shift + Esc)
2. Pestaña "Performance"
3. "Memory"
4. Mira "Total" - ese es tu RAM
5. Si es menos de 16 GB:
   - Cierra otros programas
   - Aumenta RAM (hardware upgrade)

**Paso 5: Visual C++ Redistributable**

ARK necesita ciertas librerías de Windows.

1. Ve a: https://support.microsoft.com/en-us/help/2977003
2. Descarga "Visual C++ Redistributable for Visual Studio 2022" (versión más nueva)
3. Ejecuta el instalador
4. Reinicia tu PC
5. Intenta iniciar el servidor

**Paso 6: Revisa los logs**

1. En la app, haz clic en **"📋 LOGS"** (botón de logs)
2. Busca mensajes rojos o "ERROR"
3. Copia un error y búscalo en Google o GitHub Issues
4. O comparte el error completo en el repositorio

---

## Amigos no pueden entrar

### Síntomas
- El servidor aparece en la lista de ARK
- Amigo hace clic en "Join"
- Ve un error: "Connection rejected" o "Timeout" o se queda cargando

### Soluciones (en orden)

**Paso 1: ¿El servidor está corriendo?**

1. Mira tu pantalla: ¿Ves la consola negra abierta?
2. Si no está, el servidor se cerró
3. En la app, haz clic en "START SERVER"
4. Espera a que la consola diga "Server initialized"
5. Avísale a tu amigo que intente nuevamente

**Paso 2: ¿Tu amigo está en WiFi local?**

Si están en el MISMO WiFi:

1. En tu PC, abre CMD:
   ```
   ipconfig
   ```
2. Busca "IPv4 Address" (algo como `192.168.1.50`)
3. Dale este número a tu amigo
4. Tu amigo abre ARK → Join → "Connect by IP"
5. Escribe: `192.168.1.50:7777`
6. Intenta conectar

Si funciona localmente = ✅ El servidor está bien, el problema es la conexión remota

**Paso 3: ¿Configuraste Port Forwarding?**

Si tu amigo está FUERA de tu red:

1. Abre tu router: `192.168.1.1` en navegador
2. Busca "Port Forwarding"
3. Verifica que exista una regla para puerto 7777 → 192.168.1.50:7777
4. Si no existe, créala (ver [STEAM_A2S.md](STEAM_A2S.md))

**Paso 4: ¿El Firewall de Windows está permitiendo?**

1. Presiona **Windows + R** → `wf.msc`
2. Lado izquierdo: "Inbound Rules"
3. Busca las reglas que creaste para puerto 7777 y 27015
4. Si no existen, créalas (ver [STEAM_A2S.md](STEAM_A2S.md))
5. Si existen pero están deshabilitadas (gris):
   - Haz clic derecho → "Enable Rule"

**Paso 5: ¿Tu IP pública es real?**

Algunos ISP usan **CG-NAT** (no te dan IP pública "real").

1. Ve a: https://ifconfig.me
2. Copia el número (tu IP pública)
3. Ve a tu router y busca "WAN IP" o "Internet IP"
4. ¿Son iguales?
   - **Sí** = IP real, todo bien
   - **No** = Tienes CG-NAT (debes usar Tailscale, ver [STEAM_A2S.md](STEAM_A2S.md))

**Paso 6: Usa Tailscale (solución universal)**

Si nada de lo anterior funciona:

1. Ambos descargan Tailscale: https://tailscale.com/download
2. Ambos inician sesión con Google/Microsoft
3. En tu PC, abre Tailscale → copia tu IP (ej: `100.100.100.50`)
4. Tu amigo abre ARK → Join → "Connect by IP"
5. Escribe: `100.100.100.50:7777`
6. ✅ Funciona

---

## Timeout / Conexión rechazada

### Síntomas
- "Waiting for server response..."
- "Connection timeout"
- "Server rejected connection"
- Se queda cargando y luego da error

### Causas y soluciones

| Causa | Síntoma | Solución |
|-------|---------|----------|
| Servidor no corriendo | Consola negra cerrada | Click "START SERVER" |
| Puerto bloqueado en firewall | Error inmediato | Abrir puerto en Windows Firewall |
| Router no redirige puertos | Timeout después de espera | Configurar Port Forwarding en router |
| CG-NAT del ISP | No conecta desde afuera | Usar Tailscale |
| Servidor cargando aún | Timeout después de 30s | Esperar 3-5 minutos más |
| Contraseña incorrecta | "Access denied" | Verificar Server Admin Password |

**Debug rápido:**
1. ¿Puedes entrar tú mismo desde ARK? → Servidor bien
2. ¿Amigo entra en WiFi local? → Router/Firewall problema
3. ¿Amigo no entra desde afuera? → Port Forwarding/CG-NAT

---

## El servidor va lento / Lag

### Síntomas
- El juego "congela" cada 5-10 segundos
- Los dinosaurios "teletransporte" (aparecen en otro lugar)
- Las acciones tardán en procesarse
- FPS bajísimo en el servidor

### Soluciones

**Paso 1: Cierra otros programas**

El PC no puede dedicarse 100% a ARK si hay otros programas corriendo.

1. Abre Task Manager (Ctrl + Shift + Esc)
2. Pestaña "Processes"
3. Busca programas que usen mucho "CPU" o "Memory"
   - Chrome/Firefox con muchas pestañas
   - Discord/Zoom
   - Antivirus escaneando
4. Haz clic derecho → "End Task" en esos programas
5. Deja solo Esenciales de Windows

**Paso 2: Baja la cantidad de dinosaurios**

En la app, ve a **"GAME RULES"** → **"Creature"**

Reduce estos multiplicadores:
- DinoCountMultiplier → Cambia de 1.0 a 0.5
- WildDinoCharacterFoodDrainMultiplier → Cambia de 1.0 a 0.8

Esto hace que haya menos dinosaurios = menos lag

**Paso 3: Baja gráficos/física del servidor**

En **"ADVANCED"** → **"World Advanced"**:

- DayTimeSpeedScale → Cambia de 1.0 a 1.5 (días más cortos)
- PreventDiseases → Activa (menos procesos)
- UseCorpseLocator → Desactiva (menos procesamiento)

**Paso 4: Limita jugadores simultáneos**

En **"ARKS"** → **"MaxPlayers"**:

Baja de 70 a 20-30. Menos jugadores = menos lag.

**Paso 5: Reduce mods pesados**

Algunos mods son muy pesados (muchas cosas nuevas).

En **"MOD SETTINGS"**:
- Quita mods que no uses
- Mantén solo esenciales

**Paso 6: Aumenta RAM**

Si aún va lento:
- Compra RAM adicional (DDR4/DDR5 compatible con tu PC)
- Instálala (YouTube tutorial si no sabes)
- Reinicia PC

---

## Error al guardar configuración

### Síntomas
- Haces clic en "SAVE SETTINGS"
- Aparece un error rojo
- La configuración no se guarda

### Soluciones

**Paso 1: Valida la configuración**

Algunos valores son inválidos:

1. Lee el mensaje de error
2. Busca qué campo está rojo
3. Arreglalo:
   - **Port:** Debe estar entre 1024-65535
   - **Max Players:** Debe ser 1-127
   - **Multipliers:** Deben ser números positivos
   - **Passwords:** No caracteres especiales raros

**Paso 2: Revisa permisos de carpeta**

La app necesita escribir en `C:\ASA\server\`

1. Haz clic derecho en `C:\ASA\`
2. "Properties"
3. "Security" tab
4. Haz clic en tu usuario
5. "Edit"
6. Marca ☑️ "Full Control"
7. OK → Apply

**Paso 3: Reinicia la app**

A veces un archivo queda bloqueado:

1. Cierra la app completamente
2. Espera 5 segundos
3. Abre nuevamente
4. Intenta guardar

**Paso 4: Revisa espacio en disco**

Si no hay espacio, no puede escribir:

1. Abre "This PC"
2. Haz clic derecho en C:\ → Properties
3. Mira "Free space"
4. Si hay poco, borra archivos innecesarios
5. Intenta nuevamente

---

## Los mods no cargan

### Síntomas
- Agregaste IDs de mods en la app
- El servidor inicia pero los mods no aparecen
- Error en los logs sobre mods

### Soluciones

**Paso 1: Verifica los IDs de mods**

1. En la app → **"MOD SETTINGS"** → **"Active Mods"**
2. Los IDs deben ser números, ej: `731604991`
3. NO deben ser:
   - Vacíos
   - Con letras
   - Duplicados

**Paso 2: Busca mods en CurseForge**

1. Ve a: https://www.curseforge.com/ark-survival-ascended/mods
2. Busca el mod
3. Mira la URL: `/mods/12345` ← ese número es el ID correcto
4. Cópialo a la app

**Paso 3: Algunos mods son "PC-only"**

Eso significa que solo el SERVER necesita el mod, no los jugadores.

Si un mod es "PC-only" pero tus amigos no lo tienen:
- La app te avisa automáticamente
- Déjalos así, funciona

**Paso 4: Reinicia el servidor completamente**

Los mods se cargan cuando inicia:

1. Haz clic en "STOP SERVER"
2. Espera a que la consola se cierre
3. Espera 10 segundos
4. Haz clic en "START SERVER"
5. Los mods se cargarán

**Paso 5: Verifica logs**

1. Abre la pestaña de **LOGS**
2. Busca "mod" (case insensitive)
3. Si hay errores sobre mods, significa que CurseForge los rechazó
4. Intenta con otro mod más popular

---

## El servidor se cierra de repente (Crash)

### Síntomas
- La consola desaparece sin razón
- El mensaje final dice "Crashed" o "Exception"
- Tus amigos salen sin hacer nada

### Soluciones

**Paso 1: Revisa los logs**

Antes de que se cierre, hay pistas en los logs:

1. Cuando arranca el servidor, abre los LOGS
2. Si hay mensajes rojos, anótalos
3. Si dice algo como "Out of memory" = Problema de RAM
4. Si dice "Bad mod ID" = Problema de mods
5. Si no hay error visible = Crash random (ver Paso 3)

**Paso 2: Agrega más RAM o reduce jugadores**

Si el error es "Out of memory":

1. Cierra otros programas
2. En la app, reduce MaxPlayers
3. Baja multiplicadores de dinosaurios
4. O agrega RAM (hardware upgrade)

**Paso 3: Desactiva mods problemáticos**

Si sospechas que es un mod:

1. Quita un mod a la vez
2. Arranca el servidor
3. Espera 30 minutos
4. Si no crashea, ese mod causaba el problema
5. Usa otra versión del mod o reemplázalo

**Paso 4: Actualiza Windows y drivers**

A veces crashes son por software desactualizado:

1. Windows Update → Instala todas las actualizaciones
2. Actualiza drivers de GPU (NVIDIA/AMD)
3. Reinicia
4. Intenta nuevamente

**Paso 5: Auto-restart**

Mientras investigas, usa auto-restart:

1. En la app → OPTIONS
2. Activa "Auto-restart on crash"
3. El servidor se reinicia automáticamente si cae
4. Esto compra tiempo mientras arreglas el problema

---

## No puedo cambiar de mapa

### Síntomas
- Quieres cambiar de "The Island" a otro mapa
- Haces cambios en la app pero el servidor sigue el mismo
- O aparece error

### Soluciones

**Paso 1: Entiende que cambiar mapa = Nuevo mundo**

Cuando cambias de mapa:
- **Se pierden todos los dinosaurios, estructuras, progreso**
- No es como cambiar cuarentena
- Es empezar de cero en un mapa nuevo

¿Seguro que quieres hacer esto?

**Paso 2: Para cambiar de mapa**

1. En la app → Pestaña **"ARKS"**
2. **DETÉN el servidor:** Haz clic en "STOP SERVER"
3. Espera a que la consola se cierre completamente (10-30 segundos)
4. Cambia el mapa: Selecciona de la lista (The Center, Ragnarok, etc.)
5. Ajusta puertos si es necesario (la app te ayuda)
6. Guarda: Click en "SAVE SETTINGS"
7. Inicia: Click en "START SERVER"
8. Espera a que cargue (2-5 minutos)

**Paso 3: Mapa cluster (multi-mapa)**

Si quieres tener múltiples mapas simultáneamente:

1. En **"ARKS"** → Hay una sección "Cluster Maps"
2. Agrega múltiples mapas (The Island, The Center, etc.)
3. Cada mapa en puerto diferente (7777, 7779, 7781, etc.)
4. Ambos servidores corren a la vez
5. En el juego, usa el Obelisco para cambiar de mapa sin perder personaje

---

## Los logs están vacíos

### Síntomas
- Abres la pestaña "LOGS"
- No hay nada
- O solo dice "Connecting..."

### Soluciones

**Paso 1: ¿El servidor está corriendo?**

Los logs solo aparecen si el servidor está activo:

1. ¿Ves la consola negra abierta?
2. Si no, el servidor no está corriendo
3. Click en "START SERVER"
4. Espera 30 segundos
5. Abre LOGS nuevamente

**Paso 2: Espera a que inicie completamente**

Los logs tardán un poco en empezar a mostrar:

1. Inicia el servidor
2. Espera **2-3 minutos**
3. Abre LOGS
4. Debería haber contenido

**Paso 3: Revisa ruta de logs**

Los logs están en:
```
C:\ASA\server\ShooterGame\Saved\Logs\
```

1. Abre esa carpeta
2. ¿Hay archivos `.log` ahí?
3. Si no, el servidor nunca escribió logs = No arrancó bien
4. Ve a "El servidor no arranca"

**Paso 4: Permiso de lectura**

La app necesita leer esos archivos:

1. Haz clic derecho en `C:\ASA\server\ShooterGame\Saved\Logs\`
2. Properties → Security
3. Tu usuario debe tener "Read" permitido
4. Si no, modifica permisos (Ask admin si necesario)

---

## Problema no en la lista

Si tu problema no está aquí:

1. Ve a **GitHub Issues:** https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/issues
2. Haz clic en "New Issue"
3. Describe:
   - Qué intentaste hacer
   - Qué error ves (copia el texto completo)
   - Tu hardware (RAM, CPU, espacio disponible)
   - Qué ya intentaste

4. Agrega logs si puedes (copia-pega desde la pestaña LOGS)

---

## Tabla de referencia rápida

| Problema | Causa probable | Solución rápida |
|----------|---|---|
| No abre | Archivo corrupto | Reinstalar |
| No arranca | No bajó servidor | `steamcmd.exe +app_update 2430930` |
| Timeout remoto | Port forwarding | Configurar router |
| Lag | Muchos dinos | Reducir multiplicadores |
| Crash | Sin RAM | Cerrar programas / Agregar RAM |
| Mods no cargan | ID inválido | Copiar ID de CurseForge correctamente |
| Logs vacíos | Server no corre | Click START SERVER |

---

## El servidor no aparece en la lista in-game tras una actualización de ARK

### Síntomas
- Servidor funciona, jugadores pueden entrar por IP directa (Tailscale/playit.gg)
- El log indica `Server has completed startup and is now advertising for join`
- Pero NO aparece en la lista oficial "Unofficial PC" del juego
- Empezó a ocurrir después de un update de ARK ASA (p.ej. v89.41 / Genesis 1 + Tides of Fortune)

### Causa #1 (más común): Falta `[Internationalization] Culture=en` en `GameUserSettings.ini`

ASA usa la sección `[Internationalization]` del `GameUserSettings.ini` para cierto proceso de registro de sesión con EOS/Epic. Sin ella, **el servidor nunca se pública en la lista, aunque funcione por conexión directa**. Esto es un fallo silencioso — no arroja error.

**Fix:**
1. Abre `ShooterGame\Saved\Config\WindowsServer\GameUserSettings.ini`
2. **Añade al principio del archivo** (antes de `[ServerSettings]`):
   ```ini
   [Internationalization]
   Culture=en
   ```
3. Reinicia el servidor

> Nota: La versión actual de esta app (Config Manager) ya escribe este bloque automáticamente al regenerar el INI (`persister.rs::write_gamesettings_ini`). Si tienes un INI viejo sin el bloque, agrégalo manualmente.

### Causa #2: Certificado EOS / Epic expirado o no instalado

ASA registra el servidor en la lista in-game vía **Epic Online Services** sobre TLS 1.2 (puertos 80/443). En Windows, esto requiere el **certificado CRL `Amazon RSA 2048 M02`** instalado en el Trusted Root store. Tras updates mayores, Epic rotó certificados y el viejo puede inválidar el registro.

**Fix:**
1. Descargar el CRL desde: `http://crl.r2m02.amazontrust.com/r2m02.crl`
2. Abrir `certlm.msc` (Local Machine) → **Trusted Root Certification Authorities → Certificates**
3. Click derecho → **All Tasks → Import** → seleccionar `r2m02.crl` → **Place all certificates in: Trusted Root Certification Authorities**
4. (Opcional) Repetir en `certmgr.msc` (Current User)
5. **Quitar cualquier certificado viejo** llamado `Amazon RSA 2048 M02` si fue instalado antes del 18 nov 2023 (si instalaste uno previo)
6. Reiniciar el PC

### Causa #3: Build del servidor no coincide con el cliente

Tras cada update mayor, si el SteamCMD solo hizo `app_update 2430930` (sin `validate`), pueden quedar binarios viejos. El servidor arranca y reporta "advertising", pero la lista in-game filtra por `buildid` y lo oculta.

**Fix:**
```bat
steamcmd +force_install_dir "C:\ASA\server" +login anonymous +app_update 2430930 validate +quit
```

> El flag `validate` es obligatorio — re-descarga archivos corruptos/incompletos que un `app_update` normal salta.

### Test rápido de localización (cliente)

En la consola del juego (tecla `~`):
```
Ark.UseServerList 0
```

Esto fuerza al cliente a usar el path Steam Master Server en lugar de EOS. **Si tu servidor aparece con `UseServerList 0` pero NO aparece normalmente → el problema es del path EOS** (causa #1 o #2). Si tampoco aparece con `UseServerList 0` → el problema es de Steam A2S (puertos/firewall).

### Logs relevantes a revisar
- `<server_dir>\ShooterGame\Saved\Logs\ShooterGame.log` → mirar `ARK Version`, `Server has completed startup and is now advertising for join`, `EOS`, `OnlineService`
- `app.log` del Config Manager → confirmar que `start_server` invoca `build_launch_args` con `-clusterid=` correcto

---

¿Aún necesitas ayuda? Abre un issue en GitHub con toda la información que puedas proporcionar.

