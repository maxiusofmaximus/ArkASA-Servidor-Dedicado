@echo off
net session >nul 2>&1
if %errorlevel% neq 0 (
  echo Solicitando permisos de administrador para configurar firewall...
  powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Process powershell -Verb RunAs -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File ""%~dp005_configurar_firewall.ps1""'"
  exit /b
)

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp005_configurar_firewall.ps1"
pause
