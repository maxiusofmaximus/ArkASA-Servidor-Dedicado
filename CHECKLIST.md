# Checklist de servidor ASA

Marca cada paso cuando lo tengas hecho.

## Instalacion

- [ ] Ejecutar `DESPLEGAR.bat`
- [ ] Revisar el estado que muestra el menu
- [ ] Usar opcion `1. Desplegar todo lo posible ahora`
- [ ] Revisar `config-ejemplos\servidor.ps1`
- [ ] Crear `config\servidor.ps1` si quieres personalizar nombre, claves o rutas
- [ ] Crear `C:\ASA\steamcmd`
- [ ] Crear `C:\ASA\server`
- [ ] Ejecutar instalacion con App ID `2430930`
- [ ] Confirmar que existe `C:\ASA\server\ShooterGame\Binaries\Win64\ArkAscendedServer.exe`

## Primer arranque

- [ ] Editar `scripts\02_iniciar_servidor_the_island.bat`
- [ ] Cambiar `SessionName`
- [ ] Cambiar `ServerPassword`
- [ ] Cambiar `ServerAdminPassword`
- [ ] Iniciar servidor
- [ ] Esperar a que genere `Saved\Config\WindowsServer`
- [ ] Cerrar servidor de forma ordenada

## Configuracion

- [ ] Ejecutar `scripts\04_aplicar_configuracion.bat` o revisar `GameUserSettings.ini` manualmente
- [ ] Activar PvE si lo quieres cooperativo
- [ ] Ajustar rates basicos
- [ ] Dejar RCON desactivado al principio
- [ ] Guardar backup de `Saved`

## Red local

- [ ] Entrar al servidor desde tu PC o desde otro PC de casa
- [ ] Verificar nombre del servidor en buscador
- [ ] Confirmar que no hay errores en consola

## Internet

- [ ] Ejecutar `scripts\06_diagnostico.bat`
- [ ] Comprobar IP local del PC con `ipconfig`
- [ ] Reservar IP local en router
- [ ] Ejecutar `scripts\05_configurar_firewall.bat` como administrador o abrir firewall manualmente
- [ ] Abrir firewall Windows para UDP `7777`, UDP `7778`, UDP `27015`
- [ ] Abrir TCP `27020` solo si usaras RCON
- [ ] Crear port forwarding en router hacia la IP local del PC
- [ ] Comprobar si hay CG-NAT
- [ ] Pedir a un amigo que pruebe entrada

## Mantenimiento

- [ ] Hacer backup antes de mods
- [ ] Actualizar servidor cuando ASA se actualice
- [ ] Guardar contrasenas fuera de capturas o streams
- [ ] No compartir `ServerAdminPassword`
