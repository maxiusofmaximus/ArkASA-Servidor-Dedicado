# 📚 Documentación Completa - ARK ASA Configuration Manager

> **Una guía exhaustiva explicada para cualquiera, sin importar el nivel técnico**

---

## 🎯 Elige tu punto de partida

### 🚀 **Acabo de descargar, ¿por dónde empiezo?**

→ **[GETTING_STARTED.md](GETTING_STARTED.md)**

- Instalación paso a paso
- Configuración básica inicial
- Arrancar el servidor por primera vez
- Conectarse localmente
- Invitar amigos

**Tiempo:** 30 minutos de lectura + 1-2 horas de setup

---

### 🎮 **Quiero conectar a amigos desde internet**

→ **[STEAM_A2S.md](STEAM_A2S.md)**

- Cómo obtener tu IP pública
- Configurar port forwarding en el router
- Abrir firewall de Windows
- Usar Tailscale como alternativa fácil
- Verificar que funciona

**Tiempo:** 20 minutos de lectura + 30 min configuración

---

### 🔧 **Algo no funciona, necesito ayuda**

→ **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)**

- Problema: La app no abre
- Problema: El servidor no arranca
- Problema: Amigos no pueden conectar
- Problema: Timeout / conexión rechazada
- Problema: El servidor va lentísimo
- ...y 10+ problemas más

**Busca tu síntoma, encontrarás la solución.**

---

### ❓ **Tengo una pregunta rápida**

→ **[FAQ.md](FAQ.md)**

- ¿Qué requisitos necesito?
- ¿Cuántos jugadores puedo tener?
- ¿Cuántos mods puedo agregar?
- ¿Cómo hago backup?
- ...y 50+ preguntas más

---

### 📖 **Quiero dominar TODAS las opciones**

→ **[USER_GUIDE.md](USER_GUIDE.md)**

- Explicación detallada de cada pestaña
- Qué hace cada ajuste
- Valores recomendados
- Ejemplos prácticos
- Flujo completo de creación de servidor

**Tiempo:** 1-2 horas de lectura (referencia)

---

## 📋 Mapa del Contenido

```
📚 DOCUMENTACIÓN
├── 🚀 GETTING_STARTED.md
│   ├─ Requisitos
│   ├─ Instalación
│   ├─ Configuración básica
│   ├─ Arrancar servidor
│   ├─ Conectarse localmente
│   ├─ Invitar amigos
│   ├─ Glosario simple
│   └─ Troubleshooting rápido
│
├── 🔗 STEAM_A2S.md
│   ├─ Conexión local (WiFi)
│   ├─ Conexión remota (IP pública)
│   ├─ Port Forwarding paso a paso
│   ├─ Firewall de Windows
│   ├─ Diagnóstico de conectividad
│   ├─ Tailscale (alternativa fácil)
│   ├─ Troubleshooting de conexión
│   └─ Obelisco y cluster
│
├── 🔧 TROUBLESHOOTING.md
│   ├─ La app no abre
│   ├─ El servidor no arranca
│   ├─ Amigos no pueden entrar
│   ├─ Timeout / conexión rechazada
│   ├─ Servidor lento
│   ├─ Error al guardar
│   ├─ Mods no cargan
│   ├─ Servidor crashea
│   ├─ No puedo cambiar mapa
│   ├─ Logs vacíos
│   └─ Tabla de referencia rápida
│
├── ❓ FAQ.md
│   ├─ Instalación y requisitos
│   ├─ Funcionalidad general
│   ├─ Múltiples servidores
│   ├─ Límites de jugadores y mods
│   ├─ Networking y Port Forwarding
│   ├─ CG-NAT y seguridad
│   ├─ Performance
│   ├─ Backups
│   ├─ Mods
│   ├─ Problemas comunes
│   ├─ Uso avanzado
│   ├─ Soporte
│   └─ Glosario de términos
│
└── 📖 USER_GUIDE.md
    ├─ Interfaz general
    ├─ Pestaña ARKS (identidad, mapa, puertos)
    ├─ Pestaña MOD SETTINGS (gestión de mods)
    ├─ Pestaña GAME RULES (dinosaurios, jugadores, dificultad)
    ├─ Pestaña ADVANCED (PvE/PvP, estadísticas)
    ├─ Pestaña ENGRAMS (tecnologías)
    ├─ Options Modal (settings globales, backup)
    ├─ Server Logs Panel (monitoreo)
    ├─ Comandos de administrador
    ├─ Keyboard Shortcuts
    ├─ Flujo completo de configuración
    └─ Pro Tips
```

---

## 🎓 Flujo de aprendizaje recomendado

### Para principiantes completos

1. Leer **GETTING_STARTED.md** (la guía principal)
2. Instalar siguiendo los pasos
3. Configurar básicamente
4. Si algo falla, buscar en **TROUBLESHOOTING.md**

**Objetivo:** Tener servidor corriendo localmente en ~2 horas

---

### Para invitar amigos desde internet

1. Leer **STEAM_A2S.md** (sección "Conectar desde AFUERA")
2. Configurar router y firewall
3. Probar con herramientas (Port Checker)
4. Dar IP y puerto a amigos

**Objetivo:** Amigos conectados remotamente en ~1 hora

---

### Para usuarios avanzados

1. Leer **USER_GUIDE.md** (entender cada opción)
2. Explorar **ADVANCED** tab
3. Agregar mods desde **MOD SETTINGS**
4. Configurar cluster (múltiples mapas)
5. Usar RCON para administración

**Objetivo:** Servidor profesional con múltiples mapas y mods

---

## 🔍 Búsqueda rápida por problema

| Síntoma | Documento | Sección |
|---------|-----------|---------|
| ¿Qué necesito para empezar? | GETTING_STARTED | Requisitos previos |
| ¿Cómo instalo? | GETTING_STARTED | Descargar e instalar |
| ¿Cómo arranco el servidor? | GETTING_STARTED | Iniciar el servidor |
| ¿Cómo se conecta alguien? | STEAM_A2S | Caso 1: WiFi local / Caso 2: Internet |
| La app no abre | TROUBLESHOOTING | La app no abre |
| El servidor no arranca | TROUBLESHOOTING | El servidor no arranca |
| Amigos no pueden conectar | TROUBLESHOOTING / STEAM_A2S | Secciones de conectividad |
| Lag, servidor lento | TROUBLESHOOTING | El servidor va lento |
| ¿Cuántos jugadores? | FAQ | ¿Cuál es el límite de jugadores? |
| ¿Qué mods puedo usar? | FAQ | ¿Qué mods puedo usar? |
| ¿Cómo agrego un mod? | FAQ / USER_GUIDE | Mods y MOD SETTINGS |
| Backup automático | USER_GUIDE | OPTIONS Modal → Backup |
| Comandos admin | USER_GUIDE | Comandos de administrador |
| Cambiar dificultad | USER_GUIDE | GAME RULES tab |
| Múltiples mapas (cluster) | FAQ / USER_GUIDE | Cluster Mode |

---

## 📊 Tiempo estimado

| Actividad | Tiempo |
|-----------|--------|
| Leer GETTING_STARTED | 30 min |
| Instalar app | 5 min |
| Descargar servidor (SteamCMD) | 1-2 horas |
| Configuración inicial | 15 min |
| Primer arranque | 5 min |
| Invitar amigos (local) | 5 min |
| Port forwarding (remoto) | 30 min |
| Leer USER_GUIDE completo | 2 horas |

**Total para servidor básico:** ~2-3 horas
**Total para servidor remoto con amigos:** ~4-5 horas

---

## 💡 Pro Tips

- Comienza simple (sin mods, 1 mapa)
- Agrega complejidad gradualmente
- Haz backup ANTES de cambios grandes
- Revisa logs regularmente
- Reinicia el servidor diariamente (automático)

---

## 🆘 Nivel de dificultad por tópico

| Tópico | Dificultad | Documento |
|--------|-----------|-----------|
| Instalación | ⭐ Muy fácil | GETTING_STARTED |
| Configuración básica | ⭐ Muy fácil | GETTING_STARTED |
| Conectarse localmente | ⭐ Muy fácil | GETTING_STARTED |
| Invitar amigos (remoto) | ⭐⭐ Fácil | STEAM_A2S |
| Agregar mods | ⭐⭐ Fácil | FAQ |
| Cambiar configuración | ⭐⭐ Fácil | USER_GUIDE |
| Troubleshooting | ⭐⭐⭐ Medio | TROUBLESHOOTING |
| Port forwarding | ⭐⭐⭐ Medio | STEAM_A2S |
| Cluster (multi-mapa) | ⭐⭐⭐⭐ Avanzado | USER_GUIDE |
| RCON / Administración | ⭐⭐⭐⭐ Avanzado | USER_GUIDE |
| Optimización | ⭐⭐⭐⭐⭐ Muy avanzado | TROUBLESHOOTING |

---

## 🔗 Enlaces rápidos

### Descarga
- **Latest Release:** https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases

### Reportar problemas
- **GitHub Issues:** https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/issues

### Recursos externos
- **CurseForge Mods:** https://www.curseforge.com/ark-survival-ascended/mods
- **ARK Wiki:** https://ark.wiki.gg/
- **ARK Console Commands:** https://ark.wiki.gg/wiki/Console_Commands
- **Tailscale:** https://tailscale.com/

---

## 📝 Notas de versión

Esta documentación corresponde a:
- **Aplicación:** ARK ASA Configuration Manager v1.1+
- **Plataforma:** Windows 10/11
- **Última actualización:** 2026-06-14

---

## 🙋 ¿Aún necesitas ayuda?

1. **Busca aquí primero:** FAQ.md
2. **No encontraste:** TROUBLESHOOTING.md
3. **Aún no:** GitHub Issues
4. **Emergencia:** Pregunta en comunidad (Discord/Reddit)

---

## 📄 Licencia

Toda esta documentación está bajo licencia MIT (libre para usar, modificar, compartir).

---

**¡Felicitaciones por decidirte a alojar tu propio servidor ARK! 🎮**

Esta documentación está diseñada para que cualquiera, sin importar su experiencia técnica, pueda tener un servidor funcionando y listo para jugar con amigos.

**¡A disfrutar!**
