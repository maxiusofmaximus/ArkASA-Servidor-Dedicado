# 📖 Guía Completa del Usuario - ARK ASA Configuration Manager

> **Para usuarios que quieren dominar TODAS las opciones y funcionalidades**

---

## 📋 Índice

1. [Interfaz general](#interfaz-general)
2. [Pestaña ARKS](#pestaña-arks)
3. [Pestaña MOD SETTINGS](#pestaña-mod-settings)
4. [Pestaña GAME RULES](#pestaña-game-rules)
5. [Pestaña ADVANCED](#pestaña-advanced)
6. [Pestaña ENGRAMS](#pestaña-engrams)
7. [Options Modal](#options-modal)
8. [Server Logs Panel](#server-logs-panel)
9. [Comandos de administrador](#comandos-de-administrador)
10. [Keyboard Shortcuts](#keyboard-shortcuts)

---

## Interfaz general

### Layout principal

```
┌─────────────────────────────────────────────────┐
│  [ARK LOGO]  ARK ASA Configuration Manager  [🔧]│
├─────────────────────────────────────────────────┤
│ [ARKS] [MOD SETTINGS] [GAME RULES] [ADVANCED]   │
├────────────────────┬──────────────────────────┤
│                    │                          │
│  Sub-tabs (si)     │                          │
│  (izq)             │  Main Content (der)      │
│                    │                          │
│                    │                          │
├────────────────────┴──────────────────────────┤
│ [SAVE] [RESET] [DEPLOY] [START/STOP] [⚙️ OPTIONS]│
└─────────────────────────────────────────────────┘
```

### Botones principales (ActionBar)

| Botón | Qué hace | Hotkey |
|-------|----------|--------|
| **SAVE SETTINGS** | Guarda todos los cambios | Ctrl+S |
| **RESET** | Deshace cambios (vuelve a último guardado) | Ctrl+Z |
| **CHOOSE DIFFICULTY** | Abre modal de dificultad | - |
| **START SERVER** | Inicia el servidor ARK | - |
| **STOP SERVER** | Detiene el servidor (gracefully) | - |
| **OPTIONS** (⚙️) | Abre configuración avanzada | - |
| **LOGS** (📋) | Abre panel de logs en tiempo real | - |

### Colores de validación

- 🟢 **Verde:** Válido, puedes guardar
- 🔴 **Rojo:** Error, no puedes guardar
- 🟡 **Amarillo:** Advertencia, puedes guardar pero ten cuidado
- ⚪ **Gris:** Deshabilitado o sin validación

---

## Pestaña ARKS

**Aquí configuras la identidad del servidor, mapa, puertos y red.**

### Sección: Map Selection

| Campo | Explicación | Rango | Defecto |
|-------|---|---|---|
| **Map** | Mapa donde van a jugar | Dropdown con 11 mapas | The Island |
| **Cluster Mode** | ¿Múltiples mapas simultáneamente? | On/Off | Off |

**Mapas disponibles:**
1. **The Island** — Clásico, equilibrado, recomendado para principiantes
2. **The Center** — Más grande, con cueva central
3. **Ragnarok** — Muy grande, con muchos biomas
4. **Scorched Earth** — Desértico, difícil
5. **Aberration** — Subterráneo, oscuro, peligroso
6. **Extinction** — Post-apocalíptico, dinosaurios modificados

**Si activas Cluster Mode:**
- Puedes tener 2-6 mapas ejecutándose simultáneamente
- Los puertos se auto-incrementan (7777, 7779, 7781, etc.)
- Los jugadores pueden viajar entre mapas con el Obelisco

### Sección: Identification

| Campo | Explicación | Limites | Requerido |
|-------|---|---|---|
| **Session Name** | Nombre que ven en Steam | 1-64 caracteres | ✅ Sí |
| **Server Password** | Contraseña para entrar (jugadores) | 1-50 caracteres, opcional | ❌ No |
| **Server Admin Password** | Contraseña de admin (tú la usas) | 4-27 caracteres | ✅ Sí |
| **Server MOTD** | Mensaje del día (aparece en login) | 1-1000 caracteres | ❌ No |

**Consejos:**
- **Session Name:** Sé descriptivo pero conciso
  - ✅ Bien: "Survival Casual · PvE · No Mods"
  - ❌ Mal: "AAAAAAA" o caracteres raros (ñ, é, 中文)
- **Server Password:** Déjalo vacío si quieres jugadores públicos
- **Admin Password:** Usa algo fuerte, **NO** "admin" o "123456"
  - Ejemplo fuerte: `MiServidor2024!Pk`
- **MOTD:** Aparece cuando alguien entra (máximo 1 minuto)

### Sección: Network

| Campo | Explicación | Rango | Defecto |
|-------|---|---|---|
| **Game Port** | Puerto principal (UDP) | 1024-65535 | 7777 |
| **Query Port** | Puerto de búsqueda Steam (UDP) | 1024-65535 | 27015 |
| **RCON Port** | Puerto de comandos remotos (TCP) | 1024-65535 | 27020 |
| **Server IP** | IP que reportas a los clientes | IP Address | Automático |
| **Use Public IP** | ¿Reportar IP pública en lugar de local? | On/Off | Off |

**Explicación de puertos:**

```
Game Port (7777 UDP):
  ├─ Puerto principal del juego
  ├─ Lo que ve Steam para conectar
  └─ Debe estar abierto en router/firewall

Query Port (27015 UDP):
  ├─ Puerto para A2S queries
  ├─ Steam lo usa para buscar en servidor browser
  ├─ Debe estar abierto
  └─ Puede ser el mismo que Game Port si lo deseas

RCON Port (27020 TCP):
  ├─ Remote Console (administración)
  ├─ Para dar comandos sin entrar al juego
  ├─ Puede cerrarse si no lo usas
  └─ Recomendado: Mantenlo cerrado en producción
```

**Importante:** Los 3 puertos deben ser diferentes.

**Server IP explicado:**
- **Automático:** La app detecta tu IP local
- **Si está para cambiar:** Úsalo solo si tienes Tailscale o VPN
  - Ejemplo: `100.100.100.50` (IP Tailscale)

### Sección: Server Performance

| Campo | Explicación | Impacto |
|-------|---|---|
| **Max Players** | Límite de jugadores simultáneos | Mayor = Más lag posible |
| **Cluster ID** | Identificador del cluster (si multi-mapa) | Debe ser igual en todos los mapas |
| **Cluster Dir** | Carpeta compartida entre mapas | Permite transferir dinos con Obelisco |

**Max Players:**
- 1-127 (motor de ARK no permite más)
- Recomendación por RAM:
  - **16 GB:** 8-15 jugadores
  - **32 GB:** 20-40 jugadores
  - **64 GB:** 50+ jugadores

---

## Pestaña MOD SETTINGS

**Aquí agregas y configuras mods de CurseForge.**

### Sub-tab: Active Mods

Muestra los mods que el servidor cargan cuando arranca.

| Opción | Qué hace |
|--------|----------|
| **Mod ID List** | Lista de IDs (números) |
| **Add Mod** | Agregar nuevo mod |
| **Remove Mod** | Eliminar mod de la lista |
| **Refresh from CurseForge** | Actualizar metadatos de mods |

**Cómo agregar un mod:**

1. Haz clic en "Add Mod"
2. Se abre un campo de entrada
3. Escribe el ID del mod (solo números)
4. Presiona Enter o click fuera
5. La app verifica que exista
6. Si existe, aparece el nombre y descripción
7. Guarda con "SAVE SETTINGS"

**¿De dónde obtengo el ID?**

1. Ve a: https://www.curseforge.com/ark-survival-ascended/mods
2. Busca tu mod
3. Mira la URL: `https://www.curseforge.com/ark-survival-ascended/mods/IDAQUI`
4. Ese número es el ID
5. Cópialo

**Información que ve:**

| Campo | Significado |
|-------|---|
| **Download Count** | Cuánta gente lo descargó |
| **Category** | Tipo de mod (Weapons, Buildings, QoL, etc.) |
| **PC-Only?** | ¿Solo el servidor lo necesita? |
| **Last Updated** | Cuándo fue actualizado por última vez |

**Restricciones:**
- ID debe ser numérico (sin letras, caracteres especiales)
- No puedes tener IDs duplicados
- Un mod no puede estar 2 veces en la lista
- Máximo ~200 mods (depende del servidor)

### Sub-tab: Available Mods

Navegador de mods de CurseForge.

| Opción | Qué hace |
|--------|----------|
| **Search** | Buscar por nombre |
| **Category Filter** | Filtrar por tipo |
| **Sort** | Ordenar por descargas, fecha, etc. |
| **Pagination** | Ir a página siguiente |

**Categorías populares:**

| Categoría | Ejemplos |
|-----------|----------|
| **Weapons** | Armas nuevas, balanceo |
| **Structures** | Construcciones, decoraciones |
| **Quality of Life** | Mejoras de interfaz, comodidad |
| **Creatures** | Dinosaurios nuevos |
| **Gameplay** | Mecánicas nuevas |

**API Key:**

Si alcanzas límite de búsquedas (no es común):

1. Ve a: https://www.curseforge.com/account/settings/api
2. Crea una API Key (gratis)
3. Cópiala
4. En la app → OPTIONS → pega la key
5. Ya puedes hacer más búsquedas

---

## Pestaña GAME RULES

**Configura cómo juega el servidor (dinosaurios, jugadores, reglas).**

### Sub-tab 1: Player

**Configuración de jugadores.**

| Setting | Rango | Defecto | Qué hace |
|---------|-------|---------|----------|
| **XP Multiplier** | 0.1 - 100 | 1.0 | Cuánto XP ganan |
| **Taming Speed Multiplier** | 0.1 - 100 | 1.0 | Cuánto tardan en taming |
| **Harvest Amount Multiplier** | 0.1 - 100 | 1.0 | Cuánto recolectan |
| **Player Health Multiplier** | 0.1 - 100 | 1.0 | Vida de jugadores |
| **Player Stamina Multiplier** | 0.1 - 100 | 1.0 | Resistencia de jugadores |
| **Player Weight Multiplier** | 0.1 - 100 | 1.0 | Peso que pueden cargar |
| **Crafting Speed Multiplier** | 0.1 - 100 | 1.0 | Velocidad de crafting |
| **Structure Damage Multiplier** | 0.1 - 100 | 1.0 | Daño a estructuras |

**Valores recomendados por dificultad:**

| Tipo de servidor | XP Mult | Taming | Harvest |
|---|---|---|---|
| Hardcore | 0.5x | 0.3x | 0.5x |
| Normal | 1.0x | 1.0x | 1.0x |
| Casual | 2.0x | 3.0x | 2.0x |
| Creative | 5.0x+ | 10.0x+ | 10.0x+ |

### Sub-tab 2: Creature

**Configuración de dinosaurios salvajes.**

| Setting | Rango | Defecto | Impacto |
|---------|-------|---------|---------|
| **Dino Count Multiplier** | 0.1 - 10 | 1.0 | Cantidad de dinos en el mapa |
| **Wild Dino Level** | 1 - 500 | 30 | Nivel máximo de dinos salvajes |
| **Food Drain Multiplier** | 0.1 - 10 | 1.0 | Cuánto come cada dino |
| **Tameness Decay Multiplier** | 0.1 - 10 | 1.0 | Qué tan rápido se rebela un dino |
| **Creature Spawn Interval** | 0.1 - 10 | 1.0 | Frecuencia de spawn |

**Notas:**
- Más DinoCountMultiplier = Más lag (cuidado)
- Wild Dino Level alto = Mejor loot pero más peligro
- Food Drain bajo = Dinos mueren menos de hambre

### Sub-tab 3: Structure

**Configuración de construcciones.**

| Setting | Rango | Defecto | Qué hace |
|---------|-------|---------|----------|
| **Structure Resistance** | 1 - 1000 | 1.0 | Durabilidad de estructuras |
| **Placement Distance** | 0.5 - 10 | 1.0 | Distancia mínima entre estructuras |
| **Decay Multiplier** | 0.1 - 10 | 1.0 | Velocidad de decay (destrucción) |
| **Max Structures** | 1 - 10000 | 5000 | Máximo de estructuras totales |
| **Tribe Limit** | 1 - 100 | 50 | Máximo de jugadores por tribu |

**Importante - Decay:**
- Decay = Auto-destrucción si nadie está cerca
- Multiplier bajo = Estructuras duran más
- En PvE: Puedes desactivar (0 decay)
- En PvP: Actívalo para evitar spam

### Sub-tab 4: World Rules

**Reglas generales del mundo.**

| Setting | Opciones | Defecto |
|---------|----------|---------|
| **Server PvE** | On/Off | On |
| **Day Cycle Length** | 1 - 300 min | 30 |
| **Night Length Multiplier** | 0.1 - 10 | 1.0 |
| **Temperature Modifier** | 0.1 - 10 | 1.0 |
| **Show MapPlayerLocation** | On/Off | Off |

**PvE vs PvP:**
- **PvE:** No hay daño entre jugadores, solo PvP con dinos
- **PvP:** Todo vale, jugadores pueden matarse

### Sub-tab 5: Difficulty Offset

| Setting | Rango | Qué hace |
|---------|-------|----------|
| **Difficulty Offset** | 0.0 - 1.0 | Multiplica todo (es un "meta-mult") |

Si lo pones a 1.0 = el doble de todo (nivel máx de dinos = 60 en vez de 30)

---

## Pestaña ADVANCED

**Configuración avanzada para jugadores experimentados.**

### Sub-tab 1: PvE

**Específico para servidores PvE.**

| Setting | Defecto | Qué hace |
|---------|---------|----------|
| **Cave Exclusion Zones** | On | Desactiva PvP en cuevas (solo loot) |
| **Friendly Fire** | Off | ¿Daño entre aliados en tribu? |
| **Tribe Alliances** | On | ¿Pueden aliarse tribus? |
| **Dino Ownership Inheritance** | On | ¿Heredan dinos si jugador se va? |

### Sub-tab 2: PvP

**Específico para servidores PvP.**

| Setting | Defecto | Qué hace |
|---------|---------|----------|
| **Friendly Fire** | On | Daño entre aliados |
| **Tribe Limit** | 50 | Máximo de jugadores por tribu |
| **Offline Raid Enabled** | On | ¿Pueden atacar cuando owner está offline? |
| **Protected Build** | Off | ¿Construcciones indestructibles? |
| **Raid Interval** | 24h | Cuándo se abre período de raid |

### Sub-tab 3: World Advanced

**Física del mundo.**

| Setting | Rango | Defecto |
|---------|-------|---------|
| **Day Time Speed Scale** | 0.1 - 100 | 1.0 |
| **Night Time Speed Scale** | 0.1 - 100 | 1.0 |
| **Weather Interval** | 0.1 - 100 | 1.0 |
| **Oxygen Consumption** | 0.1 - 100 | 1.0 |
| **Gravity** | 0.1 - 10 | 1.0 |

**Ejemplo:** Day Time Speed = 2.0 = días el doble de rápido = noches menos = menos lag de luz

### Sub-tab 4: Wild Dino Stats

**Estadísticas de dinosaurios salvajes.**

Hay 10 multiplicadores (uno por stat):

| Stat | Qué es |
|------|--------|
| **Health** | Vida |
| **Stamina** | Resistencia |
| **Oxygen** | Aire (para bucear) |
| **Food** | Comida que puede llevar |
| **Weight** | Peso máximo |
| **Melee DMG** | Daño físico |
| **Speed** | Velocidad de movimiento |
| **Temperature Insulation** | Resistencia al calor/frío |
| **Crafting Speed** | Velocidad de crafting (con el dino) |

Cada stat tiene **2 multiplicadores:**
- **Add Base Level Multiplier** - Cómo sube de nivel salvaje
- **Tamed Add Multiplier** - Cómo sube cuando lo crías

---

## Pestaña ENGRAMS

**Configuración de tecnologías desbloqueadas.**

| Opción | Qué hace |
|--------|----------|
| **Engram List** | Engrams que van desbloqueados por defecto |
| **Auto Engrams** | ¿Los jugadores desbloquean auto al level? |
| **Engram Point Multiplier** | Cuántos puntos de tech ganan |
| **Level Cap** | Nivel máximo de jugador |

**Terminology:**
- **Engram** = Tecnología/receta desbloqueada
- **Engram Points** = Puntos que gastas para desbloquear

---

## OPTIONS Modal

**Configuración global de la app y servidor.**

### Pestaña: General

| Opción | Qué hace |
|--------|----------|
| **Language** | Idioma de la app (inglés/español) |
| **Theme** | Tema claro/oscuro |
| **Auto-Save** | Guardar config automáticamente cada 1min |
| **Minimize to Tray** | Minimizar = Ocultar en bandeja |
| **Start with Windows** | ¿Abrir al encender PC? |

### Pestaña: Backup

**Configuración de backups automáticos.**

| Campo | Qué es |
|-------|--------|
| **Provider** | Dónde guardar (S3/Google Drive/OneDrive/iCloud) |
| **Backup Scope** | Qué guardar (solo mapa / full) |
| **Frequency** | Cada cuánto (1h/6h/24h) |
| **Auto-upload** | ¿Subir a nube automáticamente? |

**Providers:**

| Proveedor | Configuración | Gratis |
|---|---|---|
| **Local Folder** | Ruta en tu PC | ✅ Sí |
| **Google Drive** | Conecta cuenta Google | ✅ Sí |
| **OneDrive** | Conecta Microsoft 365 | ✅ Si tienes |
| **AWS S3** | Bucket + credenciales | ❌ No (pago) |
| **iCloud** | Ruta iCloud local | ✅ Sí (pero macOS) |

---

## Server Logs Panel

**Visualización en tiempo real de lo que pasa en el servidor.**

### Controles

| Botón | Qué hace |
|-------|----------|
| **Filter** | Filtrar por palabra clave |
| **Level** | Mostrar solo errores/warnings/info |
| **Follow** | Auto-scroll al último mensaje |
| **Clear** | Borrar todos los logs mostrados |
| **Export** | Guardar logs a archivo .txt |

### Lectura de logs

Formato típico:
```
[2026-06-14 15:32:10] [SERVER] Initializing world...
[2026-06-14 15:32:15] [PLAYER] PlayerName joined the server
[2026-06-14 15:32:20] [ERROR] Failed to load mod ID: 12345
```

**Colores:**
- 🟢 **Verde:** Info, todo normal
- 🟡 **Amarillo:** Warning, cuidado
- 🔴 **Rojo:** Error, algo falló
- ⚪ **Blanco:** Sistema, mensajes generales

**Palabras clave a buscar:**

| Palabra | Significado |
|---------|---|
| "Crash" | Server se cayó |
| "OutOfMemory" | Sin RAM |
| "Mod" "failed" | Mod no cargó |
| "Player" "connected" | Alguien entró |
| "RCON" | Comando remoto ejecutado |
| "SaveWorld" | Se guardó el mundo |

---

## Comandos de administrador

**Comandos que ejecutas en el juego para administrar.**

### Activar modo admin

**En el juego, presiona Tab (o tu tecla de consola):**

```
enablecheats TUCONTRASEÑA
```

Reemplaza `TUCONTRASEÑA` con tu **Server Admin Password** (la del campo de Identification).

### Comandos útiles

| Comando | Qué hace |
|---------|----------|
| `cheat SaveWorld` | Guardar el mundo (importante antes de shutdown) |
| `cheat DestroyWildDinos` | Matar todos los dinos salvajes y hacer respawn |
| `cheat ListPlayers` | Ver lista de jugadores conectados |
| `cheat KillPlayer PLAYERID` | Kick/ban a un jugador |
| `cheat Broadcast MENSAJE` | Enviar mensaje a todo el servidor |
| `cheat GiveItemNum ITEMID NUM 1 1` | Darte items (comida, etc.) |
| `cheat GiveAbility` | Darte un poder especial |
| `cheat Teleport` | Teletransportarse |
| `cheat SetAdminIcon` | Mostrar icono de admin |

**Ejemplos:**

```
cheat SaveWorld
→ Guarda el mundo (recomendado antes de parar servidor)

cheat ListPlayers
→ Muestra: PlayerName, PlayerID, Tribe, Level

cheat Broadcast HOLA A TODOS!
→ Mensaje aparece para todos

cheat DestroyWildDinos
→ Mata todos los dinos, espera ~5 minutos mientras respawnean nuevos
```

### Más información

Para lista completa de comandos:
- https://ark.wiki.gg/wiki/Console_Commands

---

## Keyboard Shortcuts

| Atajo | Qué hace |
|-------|----------|
| **Ctrl + S** | Guardar configuración |
| **Ctrl + Z** | Deshacer cambios |
| **Ctrl + F** | Buscar en logs |
| **Tab** | En-juego, abrir consola |
| **F11** | Fullscreen (algunos servidores) |
| **Ctrl + Alt + Delete** | Task Manager (matar procesos) |

---

## Flujo completo: Crear un servidor desde cero

### Día 1: Instalación

1. ✅ Descarga instalador `.exe`
2. ✅ Ejecuta instalador
3. ✅ Abre app
4. ✅ Descarga servidor ARK (SteamCMD, ~60 GB, 1-2 horas)

### Día 1-2: Configuración básica

1. En **ARKS:**
   - Session Name: "Mi Servidor"
   - Admin Password: "SuperSecret123"
   - Guarda
2. En **GAME RULES:**
   - Ajusta dificultad
   - Multipliers a gusto
3. En **MOD SETTINGS:**
   - Agrega mods (opcional)
4. En **OPTIONS:**
   - Activa Auto-Save
   - Configura backup (opcional)

### Día 2+: Arrancar y probar

1. Click **"START SERVER"**
2. Espera 3-5 minutos a que inicie
3. Abre ARK en tu PC
4. Conecta por nombre o IP local
5. Prueba todo funciona
6. Si hay problemas, revisa [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

### Día 3+: Invitar amigos

1. **Amigos locales:**
   - Dale tu IP local
   - Conectan por IP:Puerto

2. **Amigos remotos:**
   - Configura router (Port Forwarding)
   - Abre Firewall Windows
   - Dale tu IP pública
   - Conectan por IP:Puerto

   O:
   - Instala Tailscale ambos
   - Dale tu IP Tailscale
   - Conectan por IP Tailscale:Puerto

---

## Pro Tips

1. **Guarda ANTES de experimentar**
   - Ctrl+S antes de hacer cambios grandes

2. **Usa Auto-Save**
   - Actívalo en OPTIONS
   - Nunca pierdes cambios

3. **Multiplayer Settings**
   - Recomendado: MaxPlayers = RAM en GB / 2
   - Ejemplo: 32 GB RAM = 16 jugadores

4. **Backups regularmente**
   - Usa Cloud Backup (Google Drive es gratis)
   - Haz backup manual cada semana

5. **Monitorea Logs**
   - Revisa LOGS periódicamente
   - Busca "ERROR" en logs

6. **Cluster para avanzados**
   - Múltiples mapas simultáneamente
   - Jugadores transfieren con Obelisco
   - Requiere más RAM

---

**¿Preguntas?** Ve a [FAQ.md](FAQ.md) o abre un issue en GitHub.

