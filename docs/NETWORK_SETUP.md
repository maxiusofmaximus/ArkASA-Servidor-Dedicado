---
layout: default
title: Configuración de Red y Router — ARK ASA
description: Guía completa para port forwarding, IP fija, DuckDNS, firewall y seguridad RCON
---

# 🌐 Configuración de Red y Router — ARK ASA

> Esta guía cubre todo lo necesario para que amigos accedan a tu servidor desde internet.
> Válida para cualquier router. Al final hay apéndices para routers específicos.

---

## Contenido

1. [Qué puertos necesita ARK ASA](#puertos)
2. [IP local fija para el servidor](#ip-fija)
3. [Port forwarding en el router](#port-forwarding)
4. [Firewall de Windows](#firewall)
5. [DDNS con DuckDNS](#duckdns)
6. [Seguridad RCON](#rcon)
7. [CG-NAT — diagnóstico y alternativas](#cgnat)
8. [Checklist de verificación](#checklist)
9. [Apéndice A: Movistar Colombia (MITRASTAR GPT-2741GNAC)](#movistar)
10. [Apéndice B: Scripts de utilidad](#scripts)
11. [Diagnóstico y reparación de lista in-game (botón en Opciones)](#diag)
12. [No pongas `IP:puerto` en Connection Manager](#ip-no-port)

---

## 1. Qué puertos necesita ARK ASA {#puertos}

| Puerto | Protocolo | Uso | ¿Abrirlo al internet? |
|--------|-----------|-----|----------------------|
| 7777 | UDP | Game port — conexión de jugadores | **SÍ** |
| 7778 | UDP | Peer port — siempre es game_port + 1 | **SÍ** |
| 27015 | UDP | Query port — buscador de servidores | **SÍ** |
| 27020 | TCP | RCON — administración remota | **NUNCA** |

El servidor escucha en **todos los puertos en la IP local** del PC servidor. El router reenvía tráfico externo solo a los que configuremos.

---

## 2. IP local fija para el servidor {#ip-fija}

El port forwarding envía tráfico a una IP local específica. Si esa IP cambia (porque el router reasigna DHCP al reiniciarse), el forwarding deja de funcionar.

### Opción A: Reserva DHCP en el router (preferida si el router la soporta)

1. Entra al panel del router
2. Busca: **LAN → DHCP → Address Reservation** (o "Reserva de IP", "Static DHCP")
3. Agrega: MAC del servidor → IP deseada (p.ej. `192.168.1.14`)
4. Aplica y reinicia el router

Para encontrar la MAC del servidor:
```powershell
Get-NetAdapter -Physical | Select-Object Name, MacAddress, Status
```

### Opción B: IP estática en Windows (cuando el router no soporta reserva DHCP)

Ejecutar en **PowerShell como administrador**:

```powershell
# Reemplaza "Wi-Fi" por el nombre de tu adaptador y los valores por los de tu red
netsh interface ip set address name="Wi-Fi" source=static address=192.168.1.14 mask=255.255.255.0 gateway=192.168.1.1
netsh interface ip set dns name="Wi-Fi" source=static address=8.8.8.8
netsh interface ip add dns name="Wi-Fi" address=8.8.4.4 index=2
```

Para verificar que quedó estático:
```powershell
Get-NetIPAddress -InterfaceAlias "Wi-Fi" -AddressFamily IPv4 | Select-Object IPAddress, PrefixOrigin
# PrefixOrigin: Manual = estático ✓
# PrefixOrigin: Dhcp   = todavía dinámico
```

Para volver a DHCP si es necesario:
```powershell
netsh interface ip set address name="Wi-Fi" source=dhcp
netsh interface ip set dns name="Wi-Fi" source=dhcp
```

---

## 3. Port forwarding en el router {#port-forwarding}

### Reglas a crear

| Nombre | Protocolo | Puerto externo | Puerto interno | IP destino |
|--------|-----------|----------------|----------------|------------|
| ARK-Game | UDP | 7777 | 7777 | IP del servidor (p.ej. 192.168.1.14) |
| ARK-Peer | UDP | 7778 | 7778 | IP del servidor |
| ARK-Query | UDP | 27015 | 27015 | IP del servidor |

> **No abrir 27020** — RCON nunca debe ser accesible desde internet.

### Por tipo de router

**ASUS / ASUS con modo AP:**
- Avanzado → WAN → Port Forwarding virtual
- Si el ASUS está en modo AP (sin NAT propio), el forwarding debe configurarse en el router principal (el del ISP)

**TP-Link:**
- Avanzado → NAT → Port Forwarding

**Netgear:**
- Avanzado → Configuración avanzada → Port Forwarding / Port Triggering

**Modem ISP (Movistar, Claro, ETB, etc.):**
- Buscar sección "Puertos", "NAT", "Port Forwarding", o "Multipuesto"
- Si no aparece la opción, ver el Apéndice A para modems Movistar específicos

---

## 4. Firewall de Windows {#firewall}

Windows bloquea tráfico entrante por defecto. Hay que abrir los puertos del servidor.

### Reglas correctas (por puerto, no por programa)

Ejecutar en **PowerShell como administrador**:

```powershell
# Puertos de juego — abiertos a internet
New-NetFirewallRule -DisplayName "ARK-ASA-7777-Game-IN"  -Direction Inbound -Protocol UDP -LocalPort 7777  -Action Allow
New-NetFirewallRule -DisplayName "ARK-ASA-7778-Peer-IN"  -Direction Inbound -Protocol UDP -LocalPort 7778  -Action Allow
New-NetFirewallRule -DisplayName "ARK-ASA-27015-Query-IN" -Direction Inbound -Protocol UDP -LocalPort 27015 -Action Allow

# RCON — solo desde Tailscale (100.64.0.0/10) y localhost
New-NetFirewallRule -DisplayName "ARK-RCON-27020-Tailscale" -Direction Inbound -Protocol TCP -LocalPort 27020 -RemoteAddress "100.64.0.0/255.192.0.0" -Action Allow
New-NetFirewallRule -DisplayName "ARK-RCON-27020-Localhost"  -Direction Inbound -Protocol TCP -LocalPort 27020 -RemoteAddress "127.0.0.1" -Action Allow

# Outbound — el servidor necesita conectar a Epic y Steam
New-NetFirewallRule -DisplayName "ARK-ASA-Server-Outbound" -Direction Outbound -Protocol Any -Program "C:\ASA\server\ShooterGame\Binaries\Win64\ArkAscendedServer.exe" -Action Allow
```

### Problema: Windows creó reglas genéricas automáticamente

Cuando ejecutas ARK por primera vez, Windows pregunta si permitir el acceso y crea reglas para **toda la aplicación en todos los puertos**. Estas reglas son demasiado permisivas y exponen RCON a la red local.

Para limpiarlas y dejar solo reglas por puerto, usa `scripts/firewall-cleanup.ps1` de este repositorio (requiere ejecutar como administrador).

### Verificar que el puerto 27020 NO está expuesto

```powershell
# Debe mostrar solo conexiones SALIENTES de ARK a servidores Epic (no entrantes)
netstat -an | Select-String ":27020"
```

---

## 5. DDNS con DuckDNS {#duckdns}

La IP pública que te asigna el ISP puede cambiar. Con DuckDNS obtienes un hostname fijo (`tu-nombre.duckdns.org`) que siempre apunta a tu IP actual.

### Configuración

1. Ve a [duckdns.org](https://www.duckdns.org) y crea una cuenta (con Google o GitHub)
2. Crea un subdominio: p.ej. `ark-miservidor` → `ark-miservidor.duckdns.org`
3. Copia tu token de API (cadena tipo `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`)
4. Usa el script de este repo como base: `scripts/duckdns-updater-template.ps1`

### Instalar como tarea programada (cada 5 minutos)

```powershell
# Reemplaza los valores en el script primero, luego ejecuta esto como Admin:
$action  = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NonInteractive -File C:\ASA\scripts\duckdns-updater.ps1"
$trigger = New-ScheduledTaskTrigger -RepetitionInterval (New-TimeSpan -Minutes 5) -Once -At (Get-Date)
$settings = New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Minutes 2)
Register-ScheduledTask -TaskName "DuckDNS-ARK" -Action $action -Trigger $trigger -Settings $settings -RunLevel Highest -Force
```

### Verificar que funciona

```powershell
Resolve-DnsName "ark-miservidor.duckdns.org" | Select-Object Name, IPAddress
# Debe mostrar tu IP pública actual
```

---

## 6. Seguridad RCON {#rcon}

RCON permite ejecutar **cualquier comando de administrador** en el servidor, incluyendo banear jugadores, cambiar configuración, y hasta detener el servidor. Si se expone a internet, cualquier persona puede intentar explotarlo.

### Reglas de seguridad

- **NUNCA** crear una regla de port forwarding para el puerto 27020
- **NUNCA** poner 27020 en la lista de puertos abiertos en el firewall con RemoteAddress = Any
- Para administrar el servidor remotamente, usar **Tailscale** (VPN cifrada) y conectarse al RCON vía `100.x.x.x:27020`
- Para administración local, usar `127.0.0.1:27020`

### Verificar que el router NO reenvía 27020

Revisa las reglas de port forwarding en tu router. La lista debe tener solo:
- UDP 7777 → servidor
- UDP 7778 → servidor
- UDP 27015 → servidor

Si ves TCP 27020 en la lista, elimínala inmediatamente.

---

## 7. CG-NAT — diagnóstico y alternativas {#cgnat}

Algunos ISP usan NAT compartido (CG-NAT), lo que hace que el port forwarding no funcione porque la IP pública no es tuya exclusivamente.

### Diagnóstico

1. Entra al panel del router
2. Busca la IP WAN (la que el ISP te asigna)
3. Compara con tu IP pública real:
   ```powershell
   (Invoke-WebRequest -Uri "https://api.ipify.org" -UseBasicParsing).Content
   ```

**Sin CG-NAT:** IP WAN del router = IP pública que muestra el sitio web → port forwarding funciona.

**Con CG-NAT:** IP WAN del router ≠ IP pública, o la WAN empieza por `100.64.x.x` → port forwarding NO funciona.

### Alternativas si hay CG-NAT

| Opción | Dificultad | Limitación |
|--------|-----------|------------|
| **Tailscale** | Fácil | Solo amigos que también instalen Tailscale |
| **playit.gg** | Fácil | Gratuito con lag, tunneling UDP |
| **Llamar al ISP** | Fácil | Algunos dan IP pública por pedido o costo adicional |
| **VPS propio** | Avanzado | Requiere servidor cloud |

**Tailscale** es la opción más segura para amigos de confianza: instalan Tailscale, los agregas a tu red, y se conectan por IP interna sin exponer nada a internet.

---

## 8. Checklist de verificación {#checklist}

Después de configurar todo, verifica:

```
[ ] IP del servidor es fija (PrefixOrigin: Manual, o reserva DHCP confirmada)
[ ] Port forwarding: UDP 7777, 7778, 27015 apuntan al servidor
[ ] Port forwarding: TCP 27020 NO existe en la lista del router
[ ] Firewall Windows: reglas por puerto (no genéricas) para 7777, 7778, 27015
[ ] Firewall Windows: 27020 solo permitido desde Tailscale/localhost
[ ] DuckDNS: subdominio resuelve a tu IP pública actual
[ ] DuckDNS: tarea programada ejecutándose cada 5 min (ver logs en C:\ASA\scripts\duckdns.log)
[ ] Servidor ARK corriendo: amigo puede buscar por nombre en Unofficial Servers
[ ] CG-NAT: IP WAN del router == IP pública → port forwarding funciona
```

---

## Apéndice A: Movistar Colombia — MITRASTAR GPT-2741GNAC {#movistar}

El modem de Movistar Colombia tiene una interfaz web en `https://192.168.1.1:8000` (HTTPS con certificado auto-firmado). La UI para port forwarding existe pero tiene comportamiento no estándar.

### Acceso al panel

- URL: `https://192.168.1.1:8000` (aceptar advertencia de certificado)
- Login: navegar a `/cgi-bin/logIn_mhs.cgi`, escribir la contraseña en el campo de texto, hacer clic en el botón de envío

### Port forwarding — comportamiento descubierto

La interfaz gráfica tiene un formulario en `/cgi-bin/applications.cgi`. Los botones visibles invocan JavaScript que hace un POST. El endpoint correcto es:

```
POST https://192.168.1.1:8000/cgi-bin/applications.cgi
Content-Type: application/x-www-form-urlencoded
```

Parámetros requeridos:

| Parámetro | Valor | Notas |
|-----------|-------|-------|
| `sessionKey` | `""` (vacío) | El modo `api_fake` siempre lo ignora |
| `reloadFlag` | `1` | Indica recarga |
| `actionFlag` | `1` | **Para agregar** una regla nueva |
| `ruleindex` | `N` | Índice del slot vacío (0 está reservado para el ISP) |
| `PortRule_Active` | `"Yes"` | Activar la regla |
| `start_port` | Puerto WAN inicio | p.ej. `7777` |
| `end_port` | Puerto WAN fin | p.ej. `7777` |
| `appName` | Nombre descriptivo | p.ej. `ARK-Game` |
| `Addr` | IP local destino | p.ej. `192.168.1.14` |
| `PortRule_Protocol` | `"UDP"`, `"TCP"`, o `"TCP/UDP"` | |
| `oStart` | Puerto LAN inicio | igual al WAN normalmente |
| `oEnd` | Puerto LAN fin | igual al WAN normalmente |
| `submitValue` | `1` | |

> ⚠️ **Importante:** El parámetro correcto es `actionFlag: 1`, **NO** `actionType=add`. Usar `actionType=add` devuelve 200 pero no guarda la regla.

### Regla ISP preexistente — NO tocar

El router siempre tiene una regla en el índice 0:
```
HDM_CR_XXXXXXXX_HGW-XXXXXX-XXX  TCP  1024 → 7547  →  192.168.1.48
```
Esta regla es para TR-069 (gestión remota del ISP). No eliminarla.

### Reserva DHCP

Este router **no tiene reserva DHCP en la interfaz web**. La función `sethost()` en `network_map.cgi` solo actualiza el nombre y tipo de dispositivo, no fija una IP. Solución: usar **IP estática en Windows** (Opción B del paso 2).

### Reglas ARK correctas (verificadas en el panel)

```
Índice 0: [ISP] TCP 1024→7547 → 192.168.1.48   ← NO tocar
Índice 1: ARK-Game  UDP 7777→7777 → 192.168.1.14
Índice 2: ARK-Peer  UDP 7778→7778 → 192.168.1.14
Índice 3: ARK-Query UDP 27015→27015 → 192.168.1.14
```

### Si se necesita reconfigurar desde cero

El proceso completo puede automatizarse con Playwright (ver `C:\Users\Max\.claude\projects\...\memory\movistar-router-api.md` para los detalles del API). En resumen:

1. Navegar a `/cgi-bin/logIn_mhs.cgi`, llenar `#syspasswd_1`, click `#Submit`
2. Por cada regla, hacer POST a `/cgi-bin/applications.cgi` con los parámetros de la tabla anterior
3. Verificar leyendo `parsePortMapping` en la respuesta de `/cgi-bin/applications.cgi`

---

## Apéndice B: Scripts de utilidad {#scripts}

Los scripts de `C:\ASA\scripts\` complementan esta guía:

| Script | Uso |
|--------|-----|
| `scripts/duckdns-updater-template.ps1` | Template para actualizar DuckDNS automáticamente |
| `scripts/firewall-cleanup.ps1` | Elimina reglas genéricas de Windows Firewall, deja solo reglas por puerto |

### Usar duckdns-updater-template.ps1

1. Copia el archivo: `Copy-Item scripts\duckdns-updater-template.ps1 C:\ASA\scripts\duckdns-updater.ps1`
2. Edita `C:\ASA\scripts\duckdns-updater.ps1` y reemplaza `TU-TOKEN-AQUI` con tu token de DuckDNS y `tu-subdominio` con tu subdominio
3. Instala como tarea programada (ver sección DuckDNS)

### Usar firewall-cleanup.ps1

```powershell
# Ejecutar como administrador
powershell -ExecutionPolicy Bypass -File "scripts\firewall-cleanup.ps1"
```

Después del script, las reglas de ARK quedan:
- `ARK-ASA-7777-Game-IN` — UDP 7777 desde cualquier IP
- `ARK-ASA-7778-Peer-IN` — UDP 7778 desde cualquier IP
- `ARK-ASA-27015-Query-IN` — UDP 27015 desde cualquier IP
- `ARK-RCON-27020-Tailscale` — TCP 27020 solo desde 100.64.0.0/10
- `ARK-RCON-27020-Localhost` — TCP 27020 solo desde 127.0.0.1
- `ARK-ASA-Server-Outbound` — Salida de la app sin restricciones

---

## 11. Diagnóstico y reparación de lista in-game {#diag}

**Si tu servidor funciona por IP directa pero NO aparece en la lista in-game
— especialmente después de una actualización mayor de ARK (p.ej. v89.x Genesis
1 / Tides of Fortune) — usa el botón de diagnóstico del Config Manager:**

> **Options → General → "Diagnóstico y reparación de lista in-game"**

Hay dos botones:

- **DIAGNOSTICAR** — sólo verifica las 3 causas conocidas sin tocar nada.
- **REPARAR TODO** — aplica las correcciones automáticas (ver abajo) y
  re-verifica.

### Causas que detecta y repara automáticamente

1. **Bloque `[Internationalization] Culture=en` faltante** en
   `ShooterGame\Saved\Config\WindowsServer\GameUserSettings.ini`.
   ASA lo requiere para registrar la sesión con EOS/Epic; sin él el
   servidor nunca se pública en la lista (fallo silencioso — funciona por
   IP directa pero la lista no lo ve). La app lo añade automáticamente al
   INI en cada START SERVER, pero si el INI existía de antes puede faltar.

2. **Certificado EOS `Amazon RSA 2048 M02`** no instalado en Trusted Root
   de Windows. ASA registra el servidor en la lista in-game vía Epic
   Online Services sobre TLS 1.2 (puertos 80/443); tras updates mayores
   de ARK o rotaciones de cert de Epic, puede faltar. La app lo
   descarga e instala en `CurrentUser\Root` automáticamente; si querés
   instalarlo también en `LocalMachine\Root` (todos los usuarios), tenés
   que ejecutar la app como administrador.

3. **Build de instalación Steam** desactualizado (sólo informativo, NO se
   auto-ejecuta porque pisaría partidas en curso). Tras updates mayores,
   un `steamcmd +app_update 2430930` (sin `validate`) puede dejar
   binarios viejos: la lista in-game filtra servidores cuyo `buildid`
   no coincide con el cliente. La app muestra el comando exacto a
   ejecutar a mano:

   ```
   steamcmd +force_install_dir "C:\ASA\server" +login anonymous +app_update 2430930 validate +quit
   ```

### Pasos posteriores a "REPARAR TODO"

1. En el Config Manager: **START SERVER → STOP SERVER → START SERVER**
   (para que ARK relea el INI y reanuncie con EOS usando el nuevo
   certificado).
2. Si el caso #3 apareció como "stale": correr el `steamcmd validate` que
   muestra el reporte.
3. (Opcional, en el cliente del juego) abrir la consola con `~` y
   ejecutar `Ark.UseServerList 0` para forzar el path Steam Master —
   si el servidor aparece con `UseServerList 0` pero NO aparece normal,
   confirmá que el fix del certificado + Culture=en se aplicó y el
   servidor reinició.

Más detalle en `docs/TROUBLESHOOTING.md` ("El servidor no aparece en la
lista in-game tras una actualización de ARK").

---

## 12. No pongas `IP:puerto` en Connection Manager {#ip-no-port}

ARK pasa el valor del campo **Address** de tu ConnectionEntry primaria
directamente al flag `-ip=` del binario `ArkAscendedServer.exe`. ASA **no
acepta `IP:puerto`** en ese flag — espera sólo la IP.

Si escribís por ejemplo `147.185.221.29:32181` (una dirección de playit.gg
o similar) en un campo tipo "Manual" / "Local IP" / "Public IP" / "DuckDNS",
ARK se rompe silenciosamente: el proceso arranca, los jugadores pueden
entrar por IP directa vía el túnel, pero **el servidor no aparece en la
lista in-game** (EOS no acepta la dirección como IP válida para anunciar).

### Señales de que estás en este caso

- El campo Address muestra un chip amarillo ⚠ al lado de la IP.
- Al editar la IP, el borde del input se pone rojo y aparece el texto:
  *"ARK does not accept "IP:port" in the -ip= flag — remove the
  ":NNNNN" ..."*.
- En el log de ARK (`ShooterGame\Saved\Logs\ShooterGame.log`) la línea
  `Commandline:` muestra `-ip=147.185.221.29:32181`.

### Cómo solucionarlo

- **La app ya sanea la IP automáticamente desde la v[ultima]**:
  `ConnectionEntry::ip_without_port()` quita el `:NNNNN` antes de
  pasarlo al flag `-ip=`, así que aunque dejes el campo mal cargado el
  bind no se rompe. Aun así, te conviene editarlo y limpiar el campo
  para no confundirte en el futuro.
- Si lo que querés es publicar el servidor vía tunnel playit.gg,
  usa el tipo **"Playit.gg"** en Connection Manager — ese tipo SÍ admite
  `host.gl.at.ply.gg:NNNNN` como address (es el único que lo admite).
- Si querés IP + puerto para conexión por IP directa, **no lo pongas
  acá** — el puerto se asigna automáticamente según el puerto de juego
  configurado en `Options → General → Network` (7777+1 por cada slot
  del clúster). Para publicar amables, lo correcto es pasarles a tus
  jugadores la URL del tunnel playit.gg por separado.
