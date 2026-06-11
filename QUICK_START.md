# ⚡ QUICK START - EN 5 MINUTOS

```
╔════════════════════════════════════════════════════╗
║  SI ESTÁS APURADO, LEE ESTO Y PUNTO              ║
╚════════════════════════════════════════════════════╝
```

---

## ✅ ¿QUÉ SE HIZO?

| Lo que querías | Estado |
|---|---|
| Mods funcionando | ✅ LISTO |
| Spawn de dinos x2 | ✅ LISTO |
| Configuración correcta | ✅ LISTO |

---

## 🚀 3 COMANDOS Y LISTO

Abre **PowerShell** en la carpeta del proyecto y ejecuta:

### Comando 1: Copiar configuración
```powershell
Copy-Item config-ejemplos\servidor.ps1 config\servidor.ps1
```

### Comando 2: Ejecutar despliegue
```powershell
.\DESPLEGAR.ps1
```

### Comando 3: Selecciona cuando aparezca el menú
```
Opción 1 (Instalación completa) ← Si no tienes nada
O
Opción 4 (Solo aplicar config) ← Si ya tienes servidor
```

**FIN. Espera a que termine.**

---

## ✔️ CÓMO SABER QUE FUNCIONA

Cuando termine, abre este archivo:
```
C:\ASA\server\ShooterGame\Saved\Logs\ShooterGame.log
```

Busca `LoadGameMods` con Ctrl+F:

✅ **Deberías ver:**
```
[OK] UShooterEngine::LoadGameMods with 11 mods
```

❌ **Si ves:**
```
[ERROR] UShooterEngine::LoadGameMods with 0 mods
```
→ Ve a [GUIA_VALIDACION_MODS.md](GUIA_VALIDACION_MODS.md) sección Troubleshooting

---

## 📖 DOCUMENTOS (ELIGE UNO)

### Si tienes 3 minutos:
→ [CORRECCIONES_REALIZADAS.md](CORRECCIONES_REALIZADAS.md)

### Si tienes 5 minutos:
→ [PROXIMOS_PASOS.md](PROXIMOS_PASOS.md)

### Si tienes 15 minutos:
→ [ANALISIS_ERRORES_OFICIAL.md](ANALISIS_ERRORES_OFICIAL.md)

### Si quieres TODOS los documentos:
→ [INDICE_DOCUMENTACION.md](INDICE_DOCUMENTACION.md)

---

## ⚠️ SI ALGO FALLA

**Problema:** "LoadGameMods with 0 mods"  
**Solución:** [GUIA_VALIDACION_MODS.md](GUIA_VALIDACION_MODS.md) → Troubleshooting

**Problema:** Spawn de dinos normal (no el doble)  
**Solución:** [GUIA_VALIDACION_MODS.md](GUIA_VALIDACION_MODS.md) → Validación Completa

---

```
✅ LISTO. Ahora ejecuta los 3 comandos arriba.
```
