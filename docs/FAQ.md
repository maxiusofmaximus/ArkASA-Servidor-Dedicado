# ❓ Preguntas Frecuentes - ARK ASA Configuration Manager

---

## Instalación y Setup

### ¿Qué requisitos tiene?

**Mínimos:**
- Windows 10 o Windows 11
- 16 GB de RAM
- 200 GB de espacio libre
- Conexión a internet

**Recomendados:**
- Windows 11
- 32 GB de RAM
- 300 GB de espacio
- Conexión 100 Mbps o mejor

---

### ¿Puedo instalar en Mac o Linux?

**No directamente.** La app está compilada solo para Windows.

**Alternativas:**
- **Mac:** Usa una máquina virtual Windows (VirtualBox, Parallels)
- **Linux:** Usa Wine o máquina virtual Windows
- **Hosting:** Renta un servidor dedicado en AWS/Azure/DigitalOcean

---

### ¿Cuánto espacio necesita el servidor?

- **Instalación base:** ~100 GB
- **Mientras corre:** +20-30 GB para logs y saves
- **Con backups locales:** +50 GB por backup

**Total recomendado:** 300-500 GB

---

### ¿Puedo mover el servidor a otra carpeta?

Sí, pero con cuidado:

1. Para el servidor completamente
2. Mueve `C:\ASA\` a la nueva ubicación
3. En la app, actualiza los paths (si es necesario)
4. Reinicia

**No recomendado:** Mueve en caiente, perderás datos.

---

## Funcionalidad

### ¿Puedo tener múltiples servidores?

Sí, pero necesitas:
- Puertos diferentes (7777, 7779, 7781, etc.)
- Suficiente RAM (32+ GB recomendado)
- La app soporta "Cluster Mode"

En Cluster Mode, todos los mapas corren simultáneamente y pueden transferir jugadores entre ellos.

---

### ¿Cuál es el límite de jugadores?

**Técnico:**
- ARK soporta máximo 127 jugadores por servidor
- La app permite configurar 1-127

**Práctico:**
- 8-15 jugadores: 16 GB RAM
- 20-30 jugadores: 32 GB RAM
- 40-60 jugadores: 64 GB RAM
- 60+ jugadores: 128+ GB RAM + CPU muy buena

La limitación es **hardware, no software.**

---

### ¿Qué mods puedo usar?

Cualquier mod de CurseForge:
- https://www.curseforge.com/ark-survival-ascended/mods

**Limitaciones:**
- Máximo ~200 mods (depende de RAM)
- No soporta mods de Steam Workshop
- Solo CurseForge

**Compatibilidad:**
- Si un mod tiene restricción "PC-only", solo lo descarga el servidor (bien)
- Los clientes lo cargan automáticamente

---

### ¿Cuántos mods puedo agregar?

Teóricamente: Hasta que se quede sin RAM

**Prácticamente:**
- 10-20 mods: Sin problemas
- 30-50 mods: Posible, carga lenta
- 50-100 mods: Requiere 32+ GB RAM
- 100+ mods: Requiere 64+ GB y servidor muy potente

**Consejo:** Comienza con 5-10 mods populares.

---

## Conectividad y Networking

### ¿Necesito abrir puertos en el router?

**Para amigos en el WiFi (local):**
- **No,** funciona automáticamente

**Para amigos desde internet:**
- **Sí,** debes abrir puertos en el router (Port Forwarding)

Ver: [STEAM_A2S.md](STEAM_A2S.md)

---

### ¿Qué es Port Forwarding?

Es decirle al router: "Cuando alguien intente conectar al puerto 7777, redirige a mi PC."

Sin port forwarding:
- El router recibe la conexión
- No sabe qué hacer
- Rechaza

Con port forwarding:
- El router recibe la conexión en puerto 7777
- La redirige a tu PC (ej: 192.168.1.50:7777)
- ✅ Tu PC la recibe

---

### ¿Puedo jugar en el mismo PC donde corre el servidor?

**Sí,** pero con limitaciones:

- Requiere más RAM (32+ GB ideal)
- El juego y server comparten recursos
- Puede haber lag
- No es recomendado para servidores públicos

**Mejor:** Usa otro PC para jugar.

---

### ¿Qué es CG-NAT? ¿Cómo lo detecto?

**CG-NAT** = Tu ISP no te da IP pública "real"

**Síntomas:**
- Tu IP WAN (router) ≠ Tu IP pública (ifconfig.me)
- Tu IP empieza con `100.64.x.x`
- Port forwarding no funciona desde afuera

**Solución:**
- Pedir al ISP una IP real (a veces gratis)
- Usar Tailscale (VPN gratuita, más fácil)

Ver: [STEAM_A2S.md](STEAM_A2S.md#usar-tailscale-alternativa-a-port-forwarding)

---

### ¿Es seguro dejar el servidor en internet?

**Riesgos:**
- Jugadores malos pueden atacar estructuras (PvP)
- Pueden llevar loot
- Pueden crashear el servidor con mods

**Mitígalo:**
- Whitelist de jugadores (si es posible)
- Backup regular (cloud backup)
- Monitorea los logs
- Autos bans por exploit

---

## Performance y Optimización

### El servidor va lento, ¿qué hago?

1. Cierra otros programas en tu PC
2. Baja cantidad de dinosaurios (DinoCountMultiplier)
3. Baja Max Players si hay muchos jugadores
4. Quita mods innecesarios
5. Reinicia el servidor

Ver: [TROUBLESHOOTING.md](TROUBLESHOOTING.md#el-servidor-va-lento--lag)

---

### ¿Qué debo monitorear?

1. **CPU:** Debe estar < 80%
2. **RAM:** Debe quedar libre > 5 GB
3. **Disco:** Debe quedar libre > 50 GB
4. **Logs:** Busca errores rojo
5. **Jugadores:** Monitorea cantidad

Si algo se acerca al límite, reduce settings.

---

### ¿Cada cuánto debo reiniciar?

**Opciones:**
- **Diariamente:** Mejor (limpia memoria)
- **Semanalmente:** Aceptable
- **Nunca:** Malo (memory leaks)

**Recomendación:**
- Reinicia cada 24 horas a la 1 AM (cuando menos gente juega)

En la app, puedes programar auto-restart.

---

## Backups y Seguridad

### ¿Cómo hago backup?

1. Automáticamente: OPTIONS → Backup → Elige proveedor
2. Manualmente: Click en "Backup Now"

**Proveedores:**
- Google Drive (gratis, recomendado)
- OneDrive (si tienes Microsoft)
- S3 (AWS, pago)
- Carpeta local (en tu PC)

---

### ¿Puedo restaurar un backup?

Sí:

1. OPTIONS → Backup
2. Ver lista de backups disponibles
3. Click en uno → "Restore"
4. Espera a que descargue y restaure
5. Reinicia servidor

**Advertencia:** Pierdes todo lo hecho después del backup.

---

### ¿Qué protege un backup?

- Mundo (mapa, estructuras, dinos salvajes)
- Personajes de jugadores
- Datos de tribus
- Configuración del servidor

**Qué NO protege:**
- La app en sí (está instalada aparte)
- Mods (se reinstalan automáticamente)

---

## Mods y Configuración

### ¿Cómo agrego un mod?

1. Ve a: https://www.curseforge.com/ark-survival-ascended/mods
2. Busca el mod
3. Copia su ID (número en la URL)
4. En la app → MOD SETTINGS → Active Mods
5. Click "Add Mod"
6. Pega el ID
7. SAVE

El mod se descarga cuando arranca el servidor.

---

### ¿Cómo quito un mod?

1. MOD SETTINGS → Active Mods
2. Busca el mod en la lista
3. Click "Remove"
4. SAVE
5. Reinicia servidor

**Advertencia:** Los dinos/objetos del mod desaparecerán del mapa.

---

### ¿Puedo cambiar mods en vivo?

**No recomendado.** Los jugadores actuales pueden crashear.

**Mejor:**
1. Anuncia que vas a parar el servidor
2. Para el servidor
3. Agrega/quita mods
4. Guarda
5. Reinicia

Los jugadores se reconectan automáticamente.

---

### ¿Cómo cambio la dificultad después de crear el servidor?

Click en "CHOOSE DIFFICULTY" y ajusta el offset.

**Nota:** Afecta dinos nuevos, no los existentes.

---

## Problemas Comunes

### Mi amigo no puede entrar, ¿por qué?

**Posibles causas:**

| Síntoma | Causa | Solución |
|---------|-------|----------|
| "Connection timeout" | Servidor no corriendo | START SERVER |
| "Access denied" | Contraseña incorrecta | Verifica Server Password |
| "Server not responding" | Port forwarding no abierto | Configura router |
| Aparece en lista pero no entra | Firewall bloquea | Abre puertos en Windows Firewall |
| No aparece en lista | Ports malos o IP pública | Revisa ifconfig.me y router |
| No aparece en lista tras update de ARK | Falta `[Internationalization] Culture=en` / cert EOS / build-id desactualizado | **Options → "Diagnóstico y reparación de lista in-game" → REPARAR TODO** (ver [TROUBLESHOOTING.md](TROUBLESHOOTING.md#el-servidor-no-aparece-en-la-lista-in-game-tras-una-actualización-de-ark)) |
| No aparece en lista y usaste `IP:puerto` en Connection Manager | ARK no admite `:port` en `-ip=` | Quita el `:NNNNN` del campo Address o usa el tipo "Playit.gg" (ver [NETWORK_SETUP.md](NETWORK_SETUP.md#ip-no-port)) |

Ver: [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

---

### El servidor crashea sin razón

**Causas:**

| Síntoma | Causa | Solución |
|---------|-------|----------|
| Consola dice "OutOfMemory" | Sin RAM | Cierra programas / Agrega RAM |
| Consola dice "Bad mod ID" | Mod no existe | Quita ese mod |
| Sin mensaje de error | Memory leak | Reinicia servidor |
| Crashea cada X horas | Bug en mod o app | Quita mods uno a uno |

---

### Mis dinosaurios desaparecieron, ¿dónde están?

**Causas posibles:**

1. **Cambio de mapa:** Si cambias de mapa, pierdes TODO
2. **Decay:** Si no visitas la zona, estructuras se destruyen
3. **Nivel muy bajo:** Dinos débiles mueren de hambre
4. **Mod removido:** Dinos del mod desaparecen

**Recuperación:**
- Restaura un backup anterior
- En OPTIONS → Backup → Elige fecha antigua

---

## Uso Avanzado

### ¿Puedo tener un cluster de servidores (multi-mapa)?

Sí:

1. ARKS → Cluster Mode: ON
2. Agrega múltiples mapas
3. Los puertos se auto-ajustan
4. Todos corren simultáneamente
5. Jugadores usan Obelisco para cambiar

**Requiere:**
- 64+ GB RAM
- CPU muy buena
- Mayor tráfico de red

---

### ¿Cómo uso Tailscale?

1. Ambos descargan: https://tailscale.com/download
2. Ambos inician sesión
3. Tu PC servidor: Abre Tailscale → copia tu IP (100.x.x.x)
4. Tu amigo: Escribe esa IP en ARK → Join

Más detalles: [STEAM_A2S.md](STEAM_A2S.md#usar-tailscale-alternativa-a-port-forwarding)

---

### ¿Cómo uso RCON?

RCON = Remote Console (comandos sin entrar al juego)

Herramientas:
- Rcon web client (en navegador)
- ARK rcon tools (third-party)
- En-juego: Tab → enablecheats CONTRASEÑA

---

### ¿Cómo programo auto-restart?

1. OPTIONS → Server Settings
2. "Auto-restart on schedule"
3. Elige hora (ej: 2 AM)
4. El servidor se reinicia automáticamente

---

## Soporte y Comunidad

### ¿Dónde reporto bugs?

GitHub Issues:
- https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/issues

Incluye:
- Descripción del problema
- Error exacto (cópialo de LOGS)
- Tu hardware (RAM, CPU)
- Qué pasos hiciste antes de que falle

---

### ¿Hay comunidad de jugadores?

Sí:

- **Discord:** [Link si existe]
- **GitHub:** Discussions / Issues
- **Reddit:** r/ARK

---

### ¿Será de pago en el futuro?

**No.** La aplicación es y será de código abierto y gratuita.

Está bajo licencia MIT (libre para usar y modificar).

---

### ¿Puedo contribuir código?

Sí, es open-source:

- Fork el repositorio
- Haz cambios
- Envía Pull Request
- El equipo revisa y mergeea

---

## Glosario Rápido

| Término | Explicación |
|---------|---|
| **Port** | "Canal" de comunicación (como un teléfono) |
| **IP Local** | Tu dirección en casa (192.168.x.x) |
| **IP Pública** | Tu dirección en internet |
| **Port Forwarding** | Redirigir puertos en el router |
| **RCON** | Controlar servidor desde afuera |
| **Mod** | Complemento que cambia el juego |
| **CurseForge** | Sitio donde están los mods |
| **Cluster** | Múltiples servidores conectados |
| **PvE** | Jugador vs Ambiente |
| **PvP** | Jugador vs Jugador |
| **Decay** | Auto-destrucción de estructuras |
| **Taming** | Domesticar un dinosaurio |
| **Engram** | Tecnología/receta desbloqueada |
| **Tailscale** | VPN gratis (mejor alternativa a port forwarding) |
| **A2S** | Protocolo de Steam para buscar servidores |
| **Timeout** | Server no respondió a tiempo |
| **Crash** | El programa se cierra inesperadamente |

---

## Guías Completas

- 📘 **[GETTING_STARTED.md](GETTING_STARTED.md)** - Para principiantes
- 📖 **[USER_GUIDE.md](USER_GUIDE.md)** - Todas las opciones
- 🔗 **[STEAM_A2S.md](STEAM_A2S.md)** - Conectarse remotamente
- 🔧 **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Problemas y soluciones

---

¿Tu pregunta no está aquí? Abre un issue en GitHub:
https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/issues

