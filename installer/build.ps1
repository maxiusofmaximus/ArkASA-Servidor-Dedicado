# ============================================================
# Compila el instalador completo ARK ASA Setup
# Requiere: NSIS 3.x instalado
# ============================================================

$ErrorActionPreference = "Stop"

Write-Host "`n=== ARK ASA Full Setup - Builder ===" -ForegroundColor Cyan

# Buscar NSIS en rutas comunes
$nsisExe = @(
    "C:\Program Files (x86)\NSIS\makensis.exe",
    "C:\Program Files\NSIS\makensis.exe"
) | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $nsisExe) {
    # Intentar encontrarlo en PATH
    $nsisExe = (Get-Command makensis -ErrorAction SilentlyContinue)?.Source
}

if (-not $nsisExe) {
    Write-Host "`nNSIS no encontrado. Instalando via winget..." -ForegroundColor Yellow
    winget install NSIS.NSIS --accept-source-agreements --accept-package-agreements
    $nsisExe = "C:\Program Files (x86)\NSIS\makensis.exe"
    if (-not (Test-Path $nsisExe)) {
        Write-Error "No se pudo instalar NSIS. Instálalo manualmente desde https://nsis.sourceforge.io/"
        exit 1
    }
}

Write-Host "Usando NSIS: $nsisExe" -ForegroundColor Green

# Verificar que el ícono existe
$iconPath = "..\src-tauri\icons\icon.ico"
if (-not (Test-Path $iconPath)) {
    Write-Error "Ícono no encontrado en $iconPath"
    exit 1
}

# Compilar
Write-Host "`nCompilando installer..." -ForegroundColor Cyan
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

& $nsisExe setup.nsi

if ($LASTEXITCODE -eq 0) {
    $output = Join-Path $scriptDir "ARK-ASA-Full-Setup.exe"
    $size   = (Get-Item $output).Length / 1MB
    Write-Host "`n✅ Compilado exitosamente!" -ForegroundColor Green
    Write-Host "   Archivo: $output" -ForegroundColor White
    Write-Host "   Tamaño: $([Math]::Round($size, 2)) MB" -ForegroundColor White
} else {
    Write-Error "La compilación falló con código $LASTEXITCODE"
    exit $LASTEXITCODE
}
