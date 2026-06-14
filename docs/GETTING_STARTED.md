# 🎮 ARK ASA Configuration Manager - Guía de Inicio Rápido

> **Para Principiantes Completos:** Esta guía explica CADA paso, sin asumir conocimiento técnico previo.

---

## 📋 Tabla de Contenidos
1. [¿Qué es esto?](#qué-es-esto)
2. [Requisitos previos](#requisitos-previos)
3. [Descargar e instalar](#descargar-e-instalar)
4. [Primer arranque](#primer-arranque)
5. [Configuración básica](#configuración-básica)
6. [Iniciar el servidor](#iniciar-el-servidor)
7. [Conectarse localmente](#conectarse-localmente)
8. [Invitar amigos](#invitar-amigos)
9. [Pasos siguientes](#pasos-siguientes)

---

## ¿Qué es esto?

**ARK ASA Configuration Manager** es una aplicación (programa) que te permite:
- ✅ Crear y configurar un servidor de **ARK: Survival Ascended** en tu PC
- ✅ Cambiar las "reglas del juego" (cuántos dinosaurios, velocidad de crecimiento, etc.)
- ✅ Agregar "mods" (modificaciones que cambian el juego)
- ✅ Hacer copias de seguridad automáticas en la nube
- ✅ Ver en tiempo real qué está pasando en el servidor
- ✅ Controlar cuándo se enciende y apaga

**Sin esta aplicación:** Tendrías que editar archivos .INI incomprensibles manualmente.

**Con esta aplicación:** Todo es visual, como un juego más.

---

## Requisitos previos

Antes de empezar, necesitas:

### Hardware
- **PC con Windows 10 o 11** (no funciona en Mac/Linux)
- **Al menos 32 GB de RAM** (16 GB mínimo si solo tu juegas, pero va lento)
- **200 GB de espacio en disco** (para descargar el servidor)
- **Buena conexión a internet** (si quieres que amigos entren desde afuera)

### Software Instalado
1. **ARK: Survival Ascended** en Steam
   - No necesita estar instalado en el mismo PC del servidor, pero ayuda para probar
   - Si está en otro PC, eso es bien

2. **SteamCMD** (herramienta de Valve)
   - Se descarga automáticamente la primera vez
   - Es solo 100 MB

3. **Espacio en C:\**
   - Carpeta `C:\ASA\` con subcarpetas:
     - `C:\ASA\steamcmd\` — donde se descarga SteamCMD
     - `C:\ASA\server\` — donde va el servidor ARK
     - `C:\ASA\backups\` — copias de seguridad (opcional)

---

## Descargar e instalar

### Paso 1: Descargar el instalador

1. Ve a: https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases
2. Busca la versión más nueva (debería decir "Latest Release" en verde)
3. Descarga el archivo que termina en `.exe` (ejemplo: `ARK.ASA.Config.Manager_2.0.0_x64-setup.exe`)
   - Este es el "instalador" — es como descargar un juego desde Steam
4. El archivo es ~7.7 MB (tarda 10-30 segundos en descargar)

### Paso 2: Ejecutar el instalador

1. Una vez descargado, **haz doble clic** en el archivo `.exe`
2. Windows pedirá confirmación: **"¿Quieres permitir que este app haga cambios?"**
   - Haz clic en **"Sí"**
3. El instalador se abre y pregunta:
   - ¿Dónde instalar? → Acepta la ubicación por defecto (`C:\Program Files\...`)
   - ¿Crear acceso directo? → Sí, es útil
4. Espera a que termine (1-2 minutos)
5. Haz clic en **"Finalizar"**

### Paso 3: Primera ejecución

1. Aparecerá un icono en tu escritorio o en Inicio
2. **Doble clic** para abrir la app
3. La ventana se ve así:
   - **Encabezado azul-morado** con el logo de ARK
   - **Pestañas a la izquierda** (ARKS, MOD SETTINGS, etc.)
   - **Botones abajo** (SAVE SETTINGS, RESET, etc.)

✅ **¡Instalación completa!**

---

## Primer arranque

### Lo que ves la primera vez

Cuando abres la app por primera vez:

**Pestaña "ARKS"** (se abre automáticamente)
- Dice "The Island" (el mapa principal)
- Campos vacíos para nombre del servidor, contraseña, puertos
- Esto es lo que necesitas llenar

### Entender los campos

| Campo | Qué es | Ejemplo |
|-------|--------|---------|
| **Session Name** | Nombre que ven tus amigos en el navegador de servidores | "Servidor de Max" |
| **Server Password** | Contraseña para entrar al servidor (opcional) | "amigossoloamigos" |
| **Server Admin Password** | Contraseña de administrador (OBLIGATORIA, cámbiala siempre) | "CambiaEsto123!" |
| **Game Port** | Puerto del juego (déjalo en 7777) | 7777 |
| **Query Port** | Puerto para que Steam encuentre el servidor (déjalo en 27015) | 27015 |
| **RCON Port** | Puerto para controlar el servidor (déjalo en 27020) | 27020 |

---

## Configuración básica

### Paso 1: Llenar los datos del servidor

En la pestaña **ARKS**:

1. **Session Name**: Escribe el nombre que quieres que aparezca en Steam
   - Ejemplo: `Servidor Casual` o `El Reino de Max`
   - Máximo 30 caracteres (no uses caracteres extraños: ñ, tildes, etc.)

2. **Server Password**: (Opcional) Si quieres que solo ciertos amigos entren
   - Deja vacío si quieres que entre cualquiera
   - Máximo 20 caracteres

3. **Server Admin Password**: ESTO ES IMPORTANTE
   - Crea una contraseña fuerte de al menos 4 caracteres
   - Esta NO la ponen tus amigos
   - LA PONES TÚ cuando quieres dar comandos en el juego
   - Ejemplo: `MiSuperContra2024!`

4. Los puertos (7777, 27015, 27020) déjalos como están
   - Esto son "canales de comunicación" para que la gente se conecte
   - Si tienes otro servidor, cambia estos números

5. Haz clic en **"SAVE SETTINGS"** (botón azul abajo)
   - Verás un mensaje: "GUARDADO ✓"

### Paso 2: Cosas opcionales (pero recomendadas)

Ve a **Pestaña "OPTIONS"** (el icono ⚙️ arriba)

**Auto-Save:**
- ✅ Actívalo (así no tienes que guardar siempre manualmente)
- La app guarda automáticamente cada minuto

**Cloud Backup:**
- Elige un proveedor si quieres copias automáticas en la nube:
  - Google Drive (gratis, recomendado)
  - OneDrive (si tienes Microsoft 365)
  - S3 (si tienes cuenta AWS)
- Esto protege tus mundos en caso de que algo falle

### Paso 3: Configuración de dificultad (opcional)

Si quieres cambiar la dificultad del juego:

1. Haz clic en **"CHOOSE DIFFICULTY"** (botón en ActionBar)
2. Una ventana emergente muestra opciones de dificultad
3. Las opciones son:
   - **Normal/Casual**: El juego como viene
   - **Hard**: Los dinosaurios son más fuertes
   - **Custom**: Tú eliges cada cosa

---

## Iniciar el servidor

### Preparar el servidor (primera vez solo)

Antes de poder jugar, necesitas descargar el servidor de ARK desde Steam. Esto es grande (~100 GB).

1. Abre una terminal (CMD o PowerShell) como administrador:
   - Presiona **Windows + R**
   - Escribe `cmd`
   - Haz clic en OK

2. Copia y pega este comando (reemplaza la ruta si cambió):
```
C:\ASA\steamcmd\steamcmd.exe +force_install_dir C:\ASA\server +login anonymous +app_update 2430930 validate +quit
```

3. Presiona Enter
4. **Espera 30-60 minutos** (depende de tu internet)
5. Cuando termine, verá un mensaje: "App '2430930' fully installed"

✅ El servidor está descargado. Ahora sí puedes usarlo.

### Iniciar el servidor desde la app

1. En la pestaña **ARKS**, haz clic en **"START SERVER"** (botón verde)
2. Verás:
   - Una ventana negra (consola) se abre
   - Dice cosas como "Loading World..." "Initializing..."
   - Tarda 2-5 minutos en arrancar completamente
3. Cuando el servidor esté listo, dice algo como:
   - "Server initialized"
   - "Ready for players"

✅ **El servidor está corriendo ahora.**

---

## Conectarse localmente

### Jugar en el mismo PC

1. Abre **ARK: Survival Ascended** (el juego normal)
2. Busca la opción:
   - En el menú principal: **"Join" → "Non-Official Servers"**
   - Busca por nombre: Escribe el nombre del servidor (Session Name)
3. Haz clic en unirse
4. Entra tu contraseña (si la pusiste)

✅ **¡Estás adentro!**

### Jugar en otro PC en la misma red (WiFi/Red local)

Si tu amigo está en la misma casa/WiFi:

1. En su PC, abre ARK
2. Va a **"Non-Official Servers"**
3. Usa la opción **"Connect with IP"**
4. Escribe la IP local de tu PC:
   - Abre CMD en tu PC servidor
   - Escribe `ipconfig`
   - Busca "IPv4 Address" — algo como `192.168.1.50`
5. Puerto: `7777`
6. Tu amigo entra

---

## Invitar amigos

### Caso 1: Amigos en el mismo WiFi (casa)

**Para el amigo:**
1. Abre ARK
2. Busca por nombre del servidor (Session Name)
3. O conecta manualmente con la IP local

**No necesita hacer nada más.**

### Caso 2: Amigos desde otro lado (por internet)

Aquí es más complicado. Necesitas:

#### a) Configurar el router
1. Entra al panel de tu router (usualmente `192.168.1.1` en el navegador)
2. Busca "Port Forwarding" o "Redirección de puertos"
3. Crea reglas para estos puertos (reemplaza `192.168.1.50` con tu IP local):

| Protocolo | Puerto Externo | Puerto Interno | Destino |
|-----------|---|---|---|
| UDP | 7777 | 7777 | 192.168.1.50 |
| UDP | 27015 | 27015 | 192.168.1.50 |
| TCP | 27020 | 27020 | 192.168.1.50 |

4. Guarda los cambios

#### b) Abrir Firewall de Windows
1. Presiona **Windows + R**
2. Escribe `wf.msc`
3. Haz clic en "New Inbound Rule"
4. Crea reglas para puertos 7777 (UDP) y 27015 (UDP)

#### c) Dar tu IP pública a tus amigos
1. Ve a https://ifconfig.me en tu navegador
2. Copia el número que aparece (tu IP pública, ejemplo: `203.45.123.90`)
3. Dásela a tu amigo
4. Tu amigo entra así:
   - ARK → "Join" → "Connect by IP"
   - Escribe tu IP pública + puerto 7777

---

## Pasos siguientes

### Cambiar el mapa
Por defecto es "The Island". Para cambiar:

1. En la pestaña **ARKS**
2. Selecciona otro mapa de la lista (The Center, Ragnarok, Aberration, etc.)
3. Los puertos se actualizan automáticamente
4. Haz clic en **"STOP SERVER"** y luego **"START SERVER"** con el nuevo mapa

### Agregar mods
1. Ve a la pestaña **"MOD SETTINGS"**
2. Pestaña **"Available Mods"**
3. Busca mods (ejemplo: "Better Stackables")
4. Haz clic en "Add"
5. El ID del mod se agrega automáticamente
6. Reinicia el servidor para que se carguen

### Ver los logs (qué está pasando)
1. En el ActionBar arriba, haz clic en **"📋 LOGS"** (o el botón de logs)
2. Verás mensajes en tiempo real del servidor
3. Si hay errores, aparecen ahí en rojo

### Cambiar reglas del juego
Ve a la pestaña **"GAME RULES"** para cosas como:
- Cuántos dinosaurios spawnear
- Velocidad de taming (cuánto tarda domesticar un dino)
- Multiplicador de XP
- Si es PvE (no hay combate entre jugadores) o PvP

Cada opción tiene una descripción de qué hace.

### Hacer backups
En **OPTIONS** → **Backup**
- Elige tu proveedor cloud (Google Drive recomendado)
- Crea backups manuales
- O déjalo automático (cada día a las 2 AM, ejemplo)

---

## ¡Ayuda! Algo no funciona

### El servidor no arranca
**Síntomas:** Ves la consola pero dice "ERROR" o cierra al tiro

**Soluciones:**
1. ¿Bajaste ARK con SteamCMD? (Paso en "Iniciar el servidor")
   - Si no, descárgalo primero
2. ¿Tienes suficiente RAM? (Panel de control → Sistema)
   - Menos de 16 GB = muy lento o no arranca
3. ¿Otro programa usa los puertos 7777/27015?
   - Cambia los puertos en la app a números diferentes (7779, 27017)

### Mis amigos no pueden entrar
**Síntomas:** El servidor aparece en la lista, pero al conectar dice "Timeout"

**Soluciones:**
1. ¿El servidor está corriendo?
   - Mira la consola negra que se abrió ¿sigue corriendo?
   - Si se cerró, reinicia con "START SERVER"

2. ¿Los puertos están abiertos en el router?
   - Verifica "Port Forwarding" en tu router (ver sección "Invitar amigos")

3. ¿El Firewall de Windows deja pasar?
   - Ve a Firewall → "Allow an app through"
   - Busca "ARK" o agrega manualmente los puertos

4. ¿Tienes CG-NAT?
   - Si tu IP WAN no coincide con tu IP pública, sí
   - Solución: Usa una VPN como Tailscale (gratis) o pide a tu ISP una IP real

### El servidor va lento / lag
**Síntomas:** El juego congela, los dinosaurios se "teleportean"

**Soluciones:**
1. ¿Tienes muchos mods?
   - Desactiva los que no uses
2. ¿Hay muchos jugadores?
   - Reduce MaxPlayers en configuración
3. ¿Tu PC está haciendo otra cosa?
   - Cierra navegador, videos, etc.
4. ¿Muchos dinosaurios?
   - Ve a "GAME RULES" → "Creature" → baja los multiplicadores

---

## Términos técnicos explicados (EN SIMPLE)

| Término | Qué es realmente |
|---------|------------------|
| **Puerto** | Un "canal" por el que el juego se comunica (como un teléfono tiene tonos distintos) |
| **RCON** | Una forma de dar comandos al servidor desde afuera (ej: "salvar el mundo" sin entrar al juego) |
| **Query Port** | El puerto que usa Steam para BUSCAR el servidor |
| **Firewall** | Un "guarda" que decide qué entra y qué no a tu PC |
| **IP Pública** | Tu dirección en internet (lo que ve el mundo exterior) |
| **IP Local** | Tu dirección dentro de tu casa (lo que ven otros PCs en tu WiFi) |
| **Port Forwarding** | Decirle al router "reenvía esto a mi PC" |
| **Timeout** | El servidor no respondió en tiempo (es como alguien que no contesta el teléfono) |
| **Crash** | El programa se cierra de repente sin que lo cierres |
| **Mod** | Un complemento que cambia el juego (más objetos, nuevos dinosaurios, etc.) |
| **INI/TOML** | Archivos de configuración (son textos con reglas) |

---

## Resumen rápido (Tldr)

1. ✅ **Descarga** el `.exe` desde GitHub Releases
2. ✅ **Instala** haciendo doble clic
3. ✅ **Abre** la app desde el escritorio
4. ✅ **Configura**: Session Name, Admin Password, guarda
5. ✅ **Descarga el servidor** con SteamCMD (primera vez)
6. ✅ **Inicia** con el botón "START SERVER"
7. ✅ **Conecta** desde tu ARK
8. ✅ **Invita amigos** (local = WiFi, remoto = IP pública + port forwarding)

---

## Próximos pasos avanzados

- [Guía de usuario completa](USER_GUIDE.md) - Todas las opciones
- [Conexión por Steam A2S](STEAM_A2S.md) - Agregar a favoritos
- [Solución de problemas](TROUBLESHOOTING.md) - Cuando algo falla
- [FAQ](FAQ.md) - Preguntas frecuentes

---

**¿Aún tienes dudas?** Abre un issue en GitHub o pregunta en el Discord del proyecto.

**Hecho con ❤️ para que cualquiera pueda tener su propio servidor ARK.**
