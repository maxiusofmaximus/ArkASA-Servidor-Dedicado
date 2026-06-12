# ARK ASA Configuration Manager

**Professional Server Configuration UI for ARK: Survival Ascended**

[![Release](https://img.shields.io/github/v/release/maxiusofmaximus/ArkASA-Servidor-Dedicado)](https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A modern, desktop application built with **Rust + Tauri + React** for managing ARK Survival Ascended dedicated servers with an intuitive, game-like interface inspired by the official ARK UI.

## ✨ Features

- **Modern Desktop UI** - Tauri-based application (5-15MB, ultra-lightweight)
- **Cyan/Purple Theme** - Matches ARK Survival Ascended aesthetic
- **Type-Safe Configuration** - Rust backend with full validation
- **Hot Reload** - Changes apply without server restart (where possible)
- **Config Export/Import** - TOML format, human-readable
- **Automatic INI Generation** - Generates Game.ini & GameUserSettings.ini
- **Extensible Architecture** - SOLID principles throughout

## 🚀 Quick Start

### Download Pre-Built Version

Download the latest installer from [GitHub Releases](https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases):
- **ARK ASA Config Manager v1.0** (Windows 64-bit installer)

Simply run the `.exe` installer and follow the on-screen prompts.

### Build from Source

#### Prerequisites

1. **Rust** (1.70+) - [Install](https://rustup.rs/)
2. **Node.js** (18+) - [Install](https://nodejs.org/)

#### Installation

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Build for release
npm run tauri:build
```

### First Time Setup

1. Run the application
2. Go to "General" tab
3. Configure:
   - Session Name
   - Admin Password (REQUIRED - change from default)
   - Network ports
4. Click "Save Configuration"
5. Server is ready to start

## 📁 Project Structure

```
ark-asa-config/
├── src/                          # Rust backend
│   ├── config/                   # Configuration module
│   │   ├── schema.rs            # Data structures
│   │   ├── validator.rs         # Validation logic (OCP)
│   │   ├── loader.rs            # Load from TOML/INI
│   │   └── persister.rs         # Save to disk + INI generation
│   ├── ark/                     # Server management (future)
│   ├── storage/                 # Database (future)
│   ├── error.rs                 # Error types
│   ├── lib.rs                   # Tauri commands
│   └── main.rs                  # Entry point
│
├── frontend/src/                # React + TypeScript frontend
│   ├── components/              # Reusable components
│   ├── pages/                   # Tab pages
│   ├── stores/                  # Zustand global state
│   ├── services/                # API calls to Tauri
│   ├── types/                   # TypeScript types
│   ├── styles/                  # CSS + Tailwind
│   ├── App.tsx                  # Main app
│   └── main.tsx                 # React entry
│
├── docs/                        # Documentation
├── tests/                       # Integration tests
├── migrations/                  # Database migrations
├── Cargo.toml                   # Rust dependencies
├── package.json                 # Node dependencies
└── build.rs                     # Tauri build script
```

## 🏗️ Architecture

### Backend (Rust)

**SOLID Principles:**
- **S**ingle Responsibility - Each module has one job
- **O**pen/Closed - Validators are composable via traits
- **L**iskov Substitution - All validators implement ConfigValidator
- **I**nterface Segregation - Minimal, focused trait interfaces
- **D**ependency Inversion - DI through constructor injection

**Modules:**
- `config` - Configuration loading, validation, persistence
- `ark` - Server lifecycle management (future: installer, process mgmt)
- `storage` - SQLite for audit logs and version history (future)
- `error` - Typed error handling

### Frontend (React + TypeScript)

**Tech Stack:**
- **React 19** - Modern, concurrent rendering
- **TypeScript** - Full type safety
- **Tailwind CSS** - Utility-first styling
- **Zustand** - Lightweight state management
- **Tauri** - Desktop integration

**Design System:**
- Colors: Cyan (#00d4ff), Purple (#9d4edd), Dark (#0a0e27)
- Responsive grid layout
- Keyboard-accessible components

## 🔧 Configuration

### TOML Format (Primary)

```toml
[identification]
session_name = "My ARK Server"
admin_password = "SecurePassword123"

[network]
port = 7777
query_port = 27015

[gameplay]
server_pve = true
max_players = 70
dino_count_multiplier = 2.0

[multipliers]
xp_multiplier = 3.0
taming_speed_multiplier = 15.0
```

### Generated INI Files

The app auto-generates `Game.ini` and `GameUserSettings.ini` based on TOML config. These are placed at:
- `C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\Game.ini`
- `C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\GameUserSettings.ini`

## ✅ Validation

All configuration changes are validated before saving:

1. **Port Validation** - Unique ports in valid range (1024-65535)
2. **Password Validation** - Not default, minimum length 4
3. **Mod Validation** - No empty IDs, no "0", numeric only
4. **Multiplier Validation** - Positive numbers, sensible ranges
5. **Path Validation** - Must point to accessible directories

Add custom validators without modifying existing code (OCP principle).

## 📝 Development

### Running Tests

```bash
# Rust tests
cargo test

# Frontend tests
npm run test
```

### Code Quality

```bash
# Lint Rust
cargo clippy

# Format code
cargo fmt
npm run lint
```

### Building for Release

```bash
npm run tauri:build
# Generates MSI installer in src-tauri/target/release/bundle/msi/
```

## 🔄 Legacy Migration

Old PowerShell scripts and documentation have been archived in `/archive/`.

If you need the old files:
- Scripts: `/archive/legacy-scripts/`
- Docs: `/archive/legacy-docs/`
- Config examples: `/archive/legacy-config/`

## 📖 Documentation

- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design, principles, patterns
- **[API.md](docs/API.md)** - Tauri command reference
- **[CONTRIBUTING.md](docs/CONTRIBUTING.md)** - Development guidelines

## ✅ Current Status

**v1.0 Release** - Core configuration management complete:
- ✅ Desktop application with game-inspired UI
- ✅ Configuration loading/saving (TOML format)
- ✅ Automatic INI generation
- ✅ Comprehensive validation
- ✅ Type-safe Rust backend
- ✅ Full documentation

## 🛠️ Roadmap (Future Versions)

- [ ] Server start/stop/restart integration
- [ ] Real-time server status monitoring
- [ ] Config version history and rollback
- [ ] Mod management UI with validation
- [ ] Backup/restore functionality
- [ ] Multi-server support
- [ ] Web UI option (in addition to desktop)
- [ ] Performance monitoring and diagnostics

## 📄 License

MIT

---

**Built with ❤️ using Rust, Tauri, and React**

```text
C:\ASA\steamcmd
C:\ASA\server
```

Para que amigos entren desde internet todavia necesitas hacer en el router el port forwarding hacia tu IP local.

## Sobre Tauri

Una app con Rust + Tauri se puede hacer, pero en este equipo la consola actual no tiene `cargo` ni `rustc` disponibles. Para resolver el problema de hoy deje un despliegue de un solo archivo (`DESPLEGAR.bat` + `DESPLEGAR.ps1`) que ya detecta estado real, instala, configura, hace backup, prepara firewall y arranca el servidor.

Cuando Rust este instalado, este mismo flujo se puede envolver en Tauri sin cambiar la logica base.

## Subir a GitHub

El repo local ya puede subirse con GitHub CLI. Si `gh auth status` dice que el token esta vencido, ejecuta primero:

```text
gh auth login -h github.com
```

Luego ejecuta:

```text
SUBIR_GITHUB.bat
```

Por defecto crea un repo privado llamado `ArkASA-Servidor-Dedicado`.

Ruta sugerida para trabajar:

```text
C:\Users\Max\ArkASA-Servidor-Dedicado
```

## Resumen rapido

El metodo mas directo y efectivo es:

1. Revisar la configuracion en `config-ejemplos\servidor.ps1`.
2. Ejecutar `DESPLEGAR.bat`.
3. Instalar el servidor dedicado de ASA con SteamCMD usando App ID `2430930`.
4. Aplicar configuracion recomendada.
5. Arrancar `ArkAscendedServer.exe`.
6. Probar que tu puedes entrar desde tu red local.
7. Abrir firewall de Windows.
8. Redirigir puertos en el router hacia tu PC.
9. Dar a tus amigos tu IP publica o buscar el servidor por nombre dentro del juego.

No necesitas crear una aplicacion en Epic Developer Portal para montar un servidor normal. Tampoco necesitas usar manualmente el enlace `crl.r2m02.amazontrust.com`; eso es parte de comprobaciones de certificados que Windows/servicios hacen internamente. Para jugar hoy con amigos, SteamCMD es el camino practico.

## Requisitos

- Windows 10 22H2 o Windows 11 recomendado.
- ARK: Survival Ascended instalado en el PC de quien va a jugar.
- Bastante RAM. ASA consume mucho. Para pocos amigos, intenta tener al menos 16 GB de RAM total; 32 GB es mas comodo si juegas y alojas en el mismo PC.
- Conexion con buena subida. Para casa, la wiki recomienda una conexion de 100 Mb o mejor para alojar una cantidad decente de jugadores.
- Acceso al router para abrir puertos, salvo que uses una VPN/tunel tipo ZeroTier, Radmin VPN, Tailscale o playit.gg.

## Estructura recomendada

Usaremos estas carpetas:

```text
C:\ASA\steamcmd
C:\ASA\server
C:\ASA\backups
```

Si prefieres no crear `C:\ASA`, puedes usar otra ruta, pero evita espacios y acentos en la ruta para reducir problemas.

## Uso rapido del kit

Abre esta carpeta en el Explorador de Windows y ejecuta:

```text
DESPLEGAR.bat
```

El menu permite:

- ver el estado de lo instalado;
- desplegar todo lo posible en una sola opcion;
- instalar o actualizar el servidor;
- aplicar la configuracion inicial;
- iniciar The Island;
- hacer backup de `Saved`;
- crear reglas de firewall;
- ver diagnostico basico de rutas, puertos e IP local.

La configuracion central esta en:

```text
config-ejemplos\servidor.ps1
```

Si quieres personalizar sin modificar el ejemplo, crea una carpeta `config`, copia el archivo como:

```text
config\servidor.ps1
```

Los scripts usaran `config\servidor.ps1` si existe. Si no existe, usaran el ejemplo.

## Paso 1: Descargar SteamCMD

1. Abre la pagina oficial de Valve:
   `https://developer.valvesoftware.com/wiki/SteamCMD`
2. Descarga SteamCMD para Windows.
3. Crea esta carpeta:

```text
C:\ASA\steamcmd
```

4. Extrae `steamcmd.exe` dentro de:

```text
C:\ASA\steamcmd\steamcmd.exe
```

## Paso 2: Instalar o actualizar el servidor

Abre PowerShell o CMD y ejecuta:

```bat
C:\ASA\steamcmd\steamcmd.exe +force_install_dir C:\ASA\server +login anonymous +app_update 2430930 validate +quit
```

Esto descarga ARK: Survival Ascended Dedicated Server en:

```text
C:\ASA\server
```

El App ID correcto de ASA Dedicated Server es `2430930`.

Tambien deje una plantilla lista en:

```text
scripts\01_instalar_o_actualizar_servidor.bat
```

Este script ahora descarga SteamCMD si no existe, crea las carpetas necesarias y ejecuta SteamCMD con el App ID correcto.

## Paso 3: Primer arranque basico

El ejecutable deberia quedar en:

```text
C:\ASA\server\ShooterGame\Binaries\Win64\ArkAscendedServer.exe
```

Para The Island, el mapa de ASA se llama:

```text
TheIsland_WP
```

Comando basico:

```bat
C:\ASA\server\ShooterGame\Binaries\Win64\ArkAscendedServer.exe TheIsland_WP?listen?SessionName=ServidorMax?ServerPassword=ClaveParaAmigos?ServerAdminPassword=CambiaEstaClave -server -log -port=7777 -QueryPort=27015 -RCONPort=27020 -ServerPlatform=ALL
```

Notas importantes:

- En ASA el puerto de juego debe ir como `-port=7777`.
- No uses `?port=7777` para ASA, porque puede ignorarse y volver al puerto por defecto.
- `SessionName` es el nombre que veras en el buscador.
- `ServerPassword` es la clave para entrar al servidor.
- `ServerAdminPassword` es la clave para usar comandos de administrador.
- `-ServerPlatform=ALL` es lo mas simple si quieres permitir jugadores de plataformas compatibles cuando el servidor/listado lo soporte.

Plantilla lista:

```text
scripts\02_iniciar_servidor_the_island.bat
```

Ese `.bat` lee la configuracion desde `config\servidor.ps1` o `config-ejemplos\servidor.ps1`, asi que no hace falta editar el comando largo a mano.

## Paso 4: Esperar a que genere configuraciones

El primer arranque puede tardar bastante. Cuando el servidor arranque por primera vez, se generaran archivos como:

```text
C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\GameUserSettings.ini
C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\Game.ini
```

Cuando veas que el servidor ya esta corriendo, puedes cerrarlo con `Ctrl+C` en la ventana, o cerrando la consola con cuidado.

Luego puedes editar esos `.ini`.

## Paso 5: Configuracion inicial recomendada

Deje ejemplos en:

```text
config-ejemplos\GameUserSettings.ini
config-ejemplos\Game.ini
```

No reemplaces a ciegas si el servidor ya genero archivos. Copia solo las lineas que entiendas o guarda copia antes.

Tambien puedes ejecutar:

```text
scripts\04_aplicar_configuracion.bat
```

Si existe `C:\ASA\server\ShooterGame\Saved`, el script hace backup antes de tocar los `.ini`.

Configuracion sugerida para jugar con amigos:

```ini
[ServerSettings]
ServerPassword=ClaveParaAmigos
ServerAdminPassword=CambiaEstaClave
ServerPVE=true
AllowThirdPersonPlayer=true
ShowMapPlayerLocation=true
ServerCrosshair=true
RCONEnabled=false
RCONPort=27020
MaxPlayers=8
```

Para rates comodos sin romper demasiado el juego:

```ini
XPMultiplier=2.0
TamingSpeedMultiplier=3.0
HarvestAmountMultiplier=2.0
```

## Paso 6: Firewall de Windows

Para que amigos entren desde fuera de tu red, Windows debe permitir el trafico.

Abre PowerShell como administrador y ejecuta:

```powershell
New-NetFirewallRule -DisplayName "ASA UDP 7777" -Direction Inbound -Protocol UDP -LocalPort 7777 -Action Allow
New-NetFirewallRule -DisplayName "ASA UDP 7778" -Direction Inbound -Protocol UDP -LocalPort 7778 -Action Allow
New-NetFirewallRule -DisplayName "ASA UDP 27015" -Direction Inbound -Protocol UDP -LocalPort 27015 -Action Allow
New-NetFirewallRule -DisplayName "ASA TCP 27020 RCON" -Direction Inbound -Protocol TCP -LocalPort 27020 -Action Allow
```

Si no vas a usar RCON, puedes no abrir `27020` hacia internet. Para un servidor casero, mejor dejar RCON desactivado al principio.

Tambien puedes ejecutar `scripts\05_configurar_firewall.bat` como administrador. Ese script crea las reglas UDP necesarias y solo abre TCP RCON si `EnableRcon = $true` en la configuracion.

## Paso 7: IP local fija o reserva DHCP

Tu router debe enviar los puertos al PC correcto. Para eso tu PC necesita mantener la misma IP local.

1. Pulsa `Win + R`.
2. Escribe `cmd`.
3. Ejecuta:

```bat
ipconfig
```

Busca algo como:

```text
IPv4 Address . . . . . . . . . . . : 192.168.1.50
Default Gateway . . . . . . . . . . : 192.168.1.1
```

La IPv4 es tu PC. El gateway es tu router.

En el router, busca una opcion como:

- DHCP Reservation
- Address Reservation
- Reserva de DHCP
- LAN > DHCP

Reserva la IP de tu PC para que no cambie.

## Paso 8: Port forwarding en el router

En el router, redirige hacia la IP local de tu PC:

| Protocolo | Puerto externo | Puerto interno | Destino |
|---|---:|---:|---|
| UDP | 7777 | 7777 | IP local de tu PC |
| UDP | 7778 | 7778 | IP local de tu PC |
| UDP | 27015 | 27015 | IP local de tu PC |
| TCP | 27020 | 27020 | IP local de tu PC, solo si usas RCON |

La wiki oficial lista `7777/UDP`, `7778/UDP`, `27015/UDP` y `27020/TCP` opcional. Algunos recursos recientes de ASA dicen que para entrar basta con menos, pero abrir el conjunto oficial evita perder tiempo en diagnostico.

## Paso 9: Comprobar si tienes CG-NAT

Si todo parece bien pero tus amigos no pueden entrar, puede que tu proveedor use CG-NAT.

Comprobacion simple:

1. Entra al panel del router.
2. Busca la IP WAN o Internet.
3. Compara con tu IP publica vista desde una web como:

```text
https://ifconfig.me
```

Si la IP WAN del router es distinta de la IP publica, o si la WAN empieza por rangos como `100.64.x.x`, probablemente tienes CG-NAT.

Soluciones:

- Pedir al proveedor una IP publica real.
- Usar una VPN/tunel para juegos.
- Usar un VPS o hosting externo.
- Usar una herramienta de tunel tipo playit.gg si funciona bien con ASA en tu caso.

## Paso 10: Entrar al servidor

Primero prueba desde tu propia PC/red:

1. Inicia el servidor con `02_iniciar_servidor_the_island.bat`.
2. Abre ARK: Survival Ascended.
3. Busca servidores no oficiales/unofficial.
4. Busca el nombre configurado en `SessionName`.
5. Si no aparece, espera unos minutos y revisa filtros.

Para amigos:

- Dales el nombre del servidor.
- Dales la clave `ServerPassword`.
- Si lo necesitan, dales tu IP publica.

## Paso 11: Comandos de admin

Dentro del juego, abre la consola.

En PC normalmente se usa `Tab`; en ASA puede requerir abrir/expandir la consola segun configuracion.

Activa permisos:

```text
enablecheats CambiaEstaClave
```

Ejemplos utiles:

```text
cheat SaveWorld
cheat DestroyWildDinos
cheat ListPlayers
```

`DestroyWildDinos` borra dinos salvajes y fuerza respawn. Es util despues de cambiar spawns o eventos, pero puede causar lag temporal.

## Paso 12: Backups

Antes de tocar mods, mapas o configuraciones, haz copia de:

```text
C:\ASA\server\ShooterGame\Saved
```

Plantilla:

```text
scripts\03_backup_saved.bat
```

## Paso 13: Actualizar el servidor

Cuando el juego se actualice, detiene el servidor y ejecuta:

```bat
C:\ASA\steamcmd\steamcmd.exe +force_install_dir C:\ASA\server +login anonymous +app_update 2430930 validate +quit
```

Luego vuelve a iniciar.

## Mapas oficiales de ASA

Algunos nombres de mapa de ARK: Survival Ascended:

| Mapa | Nombre tecnico |
|---|---|
| The Island | `TheIsland_WP` |
| The Center | `TheCenter_WP` |
| Scorched Earth | `ScorchedEarth_WP` |
| Ragnarok | `Ragnarok_WP` |
| Aberration | `Aberration_WP` |

Para cambiar mapa, cambia el primer argumento del `.bat`.

## Mods

No empieces con mods. Primero logra que el servidor arranque, puedas entrar tu y pueda entrar un amigo.

Despues:

1. Haz backup.
2. Agrega un solo mod.
3. Arranca.
4. Prueba entrada.
5. Repite.

Si metes 10 mods de golpe y falla, no sabras cual rompio el arranque.

## Diagnostico rapido

### El servidor no arranca

- Revisa que existe `ArkAscendedServer.exe`.
- Ejecuta update con `validate`.
- Instala Visual C++ Redistributable y DirectX End-User Runtimes si faltan.
- Mira la consola del servidor.

### Yo puedo entrar, mis amigos no

- Firewall de Windows.
- Port forwarding al PC correcto.
- IP local del PC cambio.
- CG-NAT.
- Router doble: modem del proveedor + router propio.
- El servidor aun no termino de arrancar.

### No aparece en la lista

- Revisa filtros del buscador.
- Revisa `SessionName`.
- Espera unos minutos.
- Comprueba puertos.
- Prueba que un amigo entre por IP si el juego lo permite en ese flujo.

### Hay lag

- Cierra juegos/programas pesados en el PC servidor.
- Baja cantidad de jugadores.
- Evita mods pesados.
- Reinicia el servidor cada cierto tiempo.
- Haz backup antes de tocar rates o spawns.

## Fuentes consultadas

- ARK Official Community Wiki, Dedicated server setup: `https://ark.wiki.gg/wiki/Dedicated_server_setup`
- ARK Official Community Wiki, Server configuration: `https://ark.wiki.gg/wiki/Server_configuration`
- Valve Developer Community, SteamCMD: `https://developer.valvesoftware.com/wiki/SteamCMD`
- SteamDB, ASA Dedicated Server App 2430930: `https://steamdb.info/app/2430930/config/`
