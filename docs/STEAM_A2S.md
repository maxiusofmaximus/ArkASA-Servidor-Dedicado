# 🔗 Conectarse al Servidor con Steam A2S

> **Para amigos que quieren agregar tu servidor a favoritos en Steam**

---

## ¿Qué es A2S?

**A2S** = **Source Engine Query Protocol**

Es la forma estándar que usa Steam para:
1. Buscar servidores
2. Ver cuántos jugadores hay
3. Ver información del servidor
4. Permitir conectarse directamente desde Steam

**Lo importante:** Tu servidor ARK usa A2S automáticamente. Tus amigos pueden agregarlo a favoritos como cualquier otro servidor de Steam.

---

## Caso 1: Conectar en la MISMA CASA (WiFi local)

### Lo más fácil: Buscar por nombre

1. **En el juego ARK:**
   - Menú Principal → **"Join"**
   - **"Non-Official Servers"**
   - Búsqueda: Escribe el nombre exacto (Session Name)
   - Espera 2-3 segundos
   - Aparece tu servidor en la lista

2. **Haz clic en el servidor**
3. **Clic en "Join"**
4. Entra la contraseña (si la pusiste)

✅ **¡Listo!**

### Si no aparece en la búsqueda

**Conectar manualmente por IP local:**

1. En tu PC servidor, abre CMD:
   - **Windows + R** → `cmd` → OK

2. Escribe: `ipconfig`

3. Busca la línea:
   ```
   IPv4 Address . . . . . . . . . : 192.168.1.50
   ```
   (El número exacto depende de tu router, pero empieza con 192.168 o 10.0)

4. Dale este número a tu amigo

5. **En el PC del amigo:**
   - ARK → "Join" → "Non-Official Servers"
   - Opción: **"Connect by IP"** o **"Add Server"**
   - Escribe: `192.168.1.50:7777`
   - (O sin puerto si la app lo pregunta aparte: IP=`192.168.1.50`, Puerto=`7777`)

6. Haz clic en "Join"

✅ **Tu amigo entra.**

---

## Caso 2: Conectar desde AFUERA (por internet)

### Paso 1: Obtén tu IP pública

1. En **cualquier navegador** (Chrome, Edge, Firefox):
   - Ve a: https://ifconfig.me

2. O ve a: https://www.whatismyipaddress.com/

3. Copia el número grande (ejemplo: `203.45.123.90`)

   ⚠️ **IMPORTANTE:** Este número es como tu dirección de casa. Cámbialo cada mes aproximadamente (algunos ISP lo hacen automáticamente).

### Paso 2: Configura el router (Port Forwarding)

1. **Abre el panel de control del router:**
   - En el navegador: `192.168.1.1` o `192.168.0.1`
   - Login: Usualmente `admin` / `admin` o `admin` / `password`
   - (Mira la parte de atrás del router, dice ahí)

2. **Busca "Port Forwarding" o "Redirección de puertos":**
   - Puede estar en: "Advanced" → "NAT" → "Port Forwarding"
   - O: "Network" → "Port Forwarding"

3. **Crea estas 3 reglas:**

| Nombre | Protocolo | Puerto Externo | Puerto Interno | IP Destino | Puerto Destino |
|--------|-----------|---|---|---|---|
| ARK Game | UDP | 7777 | 7777 | 192.168.1.50 | 7777 |
| ARK Query | UDP | 27015 | 27015 | 192.168.1.50 | 27015 |
| ARK RCON | TCP | 27020 | 27020 | 192.168.1.50 | 27020 |

   **Reemplaza `192.168.1.50` con tu IP local real** (del paso del CMD anterior)

4. **Haz clic en "Save" o "Apply"**

5. El router se reinicia (tarda 10-30 segundos)

### Paso 3: Abre el Firewall de Windows

1. **En tu PC servidor:**
   - Presiona **Windows + R**
   - Escribe: `wf.msc`
   - Presiona Enter

2. **En la ventana de Firewall:**
   - Lado izquierdo: Haz clic en **"Inbound Rules"** (Reglas de entrada)
   - Lado derecho: Haz clic en **"New Rule"** (Nueva regla)

3. **Primera regla (UDP 7777):**
   - Tipo: **"Port"** → Next
   - Protocolo: **"UDP"** → Next
   - Puerto específico: **7777** → Next
   - Acción: **"Allow"** → Next
   - Nombre: `ARK Game Port` → Finish

4. **Segunda regla (UDP 27015):**
   - Repite lo anterior pero con puerto **27015**
   - Nombre: `ARK Query Port`

5. **Tercera regla (TCP 27020):**
   - Tipo: Port
   - Protocolo: **TCP** (no UDP esta vez)
   - Puerto: **27020**
   - Nombre: `ARK RCON Port`

### Paso 4: Dales los detalles a tus amigos

Crea un mensaje tipo este:

```
Hola! Mi servidor ARK está listo.

IP del servidor: 203.45.123.90
Puerto: 7777
Contraseña: (si existe) MiPassword123

Para conectar:
1. Abre ARK
2. Menú → Join → Non-Official Servers
3. Haz clic en "Connect by IP"
4. Escribe: 203.45.123.90:7777
5. Si pide contraseña: MiPassword123
```

### Paso 5: Verifica que funciona

1. **Desde un teléfono (fuera del WiFi):**
   - Ve a: https://portchecker.co/
   - Puerto a revisar: 7777
   - Host: Tu IP pública (203.45.123.90)
   - Haz clic en "Check"
   - Si dice "open" o "reachable" = ✅ Funciona

2. **Desde otro PC (fuera de tu red):**
   - Abre ARK
   - Join → Non-Official Servers
   - Connect by IP: TU_IP:7777
   - Debería funcionar

---

## ¿Por qué Timeout? (Conexión rechazada)

Si tu amigo dice: **"Timeout waiting for server"** o **"Connection refused"**

Significa que el servidor no respondió. Causas:

### 1. El servidor no está corriendo
**Solución:** En tu PC, verifica que la consola negra del servidor sigue abierta

### 2. El puerto forwarding no está configurado
**Solución:** 
- Ve a tu router nuevamente
- Verifica que creaste las 3 reglas
- Asegúrate de haber puesto **tu IP local correcta** (192.168.1.50)

### 3. El Firewall de Windows lo bloquea
**Solución:**
- Abre otra vez `wf.msc`
- Verifica que las 3 reglas estén ahí
- Haz clic derecho en cada regla → **"Enable Rule"**

### 4. Tienes CG-NAT (tu ISP no te da IP pública real)
**Síntomas:**
- Tu IP WAN (en el router) NO coincide con la IP de https://ifconfig.me
- Tu IP empieza en `100.64.x.x`

**Soluciones:**
- **Opción 1:** Pedir al ISP una IP pública "real" (algunos lo hacen gratis)
- **Opción 2:** Usar una VPN gratuita como **Tailscale**
  - Instalas en tu PC servidor
  - Instalas en PC de tus amigos
  - Te da una IP "virtual" que funciona en todas partes
  - Más seguro y confiable que port forwarding

---

## Usar Tailscale (Alternativa a Port Forwarding)

Si el port forwarding no te funciona, **Tailscale es la mejor opción.**

### Instalación

1. Ve a: https://tailscale.com/download/windows
2. Descarga e instala (es pequeño, 10 MB)
3. Abre Tailscale
4. Haz clic en "Log in"
5. Se abre el navegador → Entra con una cuenta Google o Microsoft
6. Autoriza el acceso
7. ¡Listo! Tailscale corre en background

### Usar

1. **En tu PC servidor:**
   - Abre Tailscale
   - Ve a la IP que dice al lado de tu nombre (ej: `100.100.100.50`)
   - Esta es tu IP privada pero accesible desde CUALQUIER LADO
   - **Memoriza esta IP o cópiala**

2. **Dásela a tus amigos:**
   ```
   Mi IP Tailscale: 100.100.100.50
   Puerto: 7777
   Para conectar: 100.100.100.50:7777
   ```

3. **En el PC del amigo:**
   - Instala Tailscale también (desde https://tailscale.com)
   - Abre ARK
   - Join → Non-Official Servers → Connect by IP
   - Escribe: `100.100.100.50:7777`
   - ¡Funciona!

### Ventajas de Tailscale
✅ No necesita port forwarding
✅ No depende de tu ISP
✅ Más seguro (encriptado)
✅ Mismo resultado (conexión directa)
✅ Gratis para hasta 3 dispositivos

---

## Caso especial: A2S y "Obelisco"

En ARK hay un lugar llamado **"Obelisco"** donde puedes:
- Cambiar de mapa sin recargar
- Transferir dinosaurios entre servidores
- Guardar cosas

### Cómo funciona

Si tienes un **cluster** (varios servidores/mapas):
1. Ve a "ARKS" tab en la app
2. Agrega múltiples mapas (The Island, The Center, etc.)
3. Cada uno en puerto diferente (7777, 7779, 7781, etc.)
4. En el juego, puedes ir al Obelisco y cambiar de mapa

El Obelisco automáticamente detecta los otros mapas si están corriendo en los puertos correctos.

---

## Resumen de direcciones

### Local (WiFi casa)
```
IP: 192.168.1.50:7777
```

### Internet (Port Forwarding)
```
IP: 203.45.123.90:7777
Requires: Abrir puertos en router + Firewall Windows
```

### Internet (Tailscale - RECOMENDADO)
```
IP: 100.100.100.50:7777
Requires: Instalar Tailscale en ambos PCs
```

---

## Verificar qué está funcionando

**Herramienta: Port Checker**
- https://portchecker.co/
- Escribe tu IP y puerto
- Si dice "open" = ✅ Funciona

**Herramienta: nmap (avanzado)**
```
En CMD:
nmap -p 7777 TU_IP_PUBLICA
```
Si ve el puerto abierto = ✅ Funciona

---

## ¡No funciona aún?

Pasos de diagnóstico:

1. **¿El servidor está corriendo?**
   - ¿Ves la consola negra abierta en tu PC?
   - ¿Dice algo en la consola o solo está esperando?

2. **¿Puedes conectar localmente?**
   - Abre ARK en tu mismo PC
   - ¿Puedes entrar? Si sí, tu servidor está bien
   - Si no, problema es el servidor, no la red

3. **¿Los puertos están abiertos?**
   - Ve a https://portchecker.co/
   - Verifica puertos 7777 y 27015
   - Si dice "closed" = problema en router/firewall

4. **¿IP pública real?**
   - https://ifconfig.me (copia)
   - Router → WAN IP (copia)
   - ¿Son iguales? Si no, tienes CG-NAT (usa Tailscale)

---

**¿Sigue sin funcionar?** Ve a [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

