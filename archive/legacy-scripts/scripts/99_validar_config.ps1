# Script de validacion y reparacion de GameUserSettings.ini
# Uso: .\99_validar_config.ps1

param(
  [switch]$Repair  # Si se activa, intenta reparar el archivo
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ConfigPath = "C:\ASA\server\ShooterGame\Saved\Config\WindowsServer\GameUserSettings.ini"

Write-Host "=============================================="
Write-Host "  VALIDACION DE CONFIGURACION"
Write-Host "=============================================="
Write-Host ""

if (-not (Test-Path $ConfigPath)) {
  Write-Host "[ERROR] Archivo no encontrado: $ConfigPath"
  exit 1
}

Write-Host "[INFO] Verificando: $ConfigPath"
Write-Host ""

# Leer el archivo
$content = Get-Content $ConfigPath -Raw

# Validaciones
$issues = @()

# 1. Validar ActiveMods
Write-Host "[1/4] Validacion: ActiveMods (Mods)"
if ($content -match 'ActiveMods\s*=\s*(.+?)(?:\r?\n|$)') {
  $activeMods = $matches[1]
  Write-Host "  Valor encontrado: ActiveMods=$activeMods"

  # Verificar espacios despues de comas
  if ($activeMods -match ',\s') {
    $issues += "ActiveMods tiene espacios despues de comas"
    Write-Host "  [ERROR] Hay espacios despues de comas"
  } else {
    Write-Host "  [OK] Formato correcto"
  }

  # Verificar que no hay valores vacios
  $modIds = @($activeMods -split ',').Trim()
  $emptyMods = @($modIds | Where-Object { [string]::IsNullOrWhiteSpace($_) })
  if ($emptyMods.Count -gt 0) {
    $issues += "ActiveMods contiene valores vacios"
    Write-Host "  [ERROR] Contiene valores vacios"
  }

  $zeroBit = @($modIds | Where-Object { $_ -eq '0' })
  if ($zeroBit.Count -gt 0) {
    $issues += "ActiveMods contiene mod id 0"
    Write-Host "  [ERROR] Contiene mod id 0 - PROBLEMA ENCONTRADO"
  }
} else {
  $issues += "No se encontro ActiveMods"
  Write-Host "  [ERROR] No encontrado"
}
Write-Host ""

# 2. Validar DinoCountMultiplier
Write-Host "[2/4] Validacion: DinoCountMultiplier (Spawn)"
if ($content -match 'DinoCountMultiplier\s*=\s*(.+?)(?:\r?\n|$)') {
  $dino = $matches[1]
  Write-Host "  Valor encontrado: DinoCountMultiplier=$dino"
  if ($dino -eq "2.0" -or $dino -eq "2") {
    Write-Host "  [OK] Spawn duplicado"
  } else {
    Write-Host "  [INFO] Spawn: $dino (esperado 2.0)"
  }
} else {
  $issues += "No se encontro DinoCountMultiplier"
  Write-Host "  [ERROR] No encontrado"
}
Write-Host ""

# 3. Validar Cryopod settings
Write-Host "[3/4] Validacion: Cryopod Settings"
$cryoParams = @(
  @{Name="DisableCryopodFridgeRequirement"; Value="true"},
  @{Name="DisableCryopodEnemyCheck"; Value="true"},
  @{Name="EnableCryoSicknessPVE"; Value="false"}
)

foreach ($param in $cryoParams) {
  if ($content -match "$($param.Name)\s*=\s*(.+?)(?:\r?\n|$)") {
    $value = $matches[1]
    Write-Host "  [OK] $($param.Name) = $value"
  } else {
    $issues += "$($param.Name) no encontrado"
    Write-Host "  [ERROR] $($param.Name) no encontrado"
  }
}
Write-Host ""

# 4. Validar lineas mal formadas
Write-Host "[4/4] Validacion: Lineas mal formadas"
$lines = $content -split "`r`n"
$malFormed = @()

foreach ($line in $lines) {
  if ($line.Trim() -and -not $line.StartsWith('[')) {
    if ($line -notmatch '^\s*[a-zA-Z0-9_]+\s*=') {
      $malFormed += $line
    }
  }
}

if ($malFormed.Count -gt 0) {
  Write-Host "  [ERROR] Lineas mal formadas encontradas:"
  foreach ($line in $malFormed | Select-Object -First 5) {
    Write-Host "    > $line"
  }
} else {
  Write-Host "  [OK] No hay lineas mal formadas"
}
Write-Host ""

# Resumen
Write-Host "=============================================="
Write-Host "  RESUMEN"
if ($issues.Count -eq 0) {
  Write-Host "  [OK] ARCHIVO VALIDO - Sin problemas"
} else {
  Write-Host "  [ERROR] PROBLEMAS: $($issues.Count)"
  Write-Host "=============================================="
  foreach ($issue in $issues) {
    Write-Host "  * $issue"
  }
}
Write-Host "=============================================="
Write-Host ""

# Ofrecer reparacion
if ($issues.Count -gt 0 -and -not $Repair) {
  Write-Host "Para reparar, ejecuta:"
  Write-Host "  .\99_validar_config.ps1 -Repair"
  Write-Host ""
}

# Funcion de reparacion
if ($Repair -and $issues.Count -gt 0) {
  Write-Host "INTENTANDO REPARACION..."
  Write-Host ""

  # Backup
  $backup = "$ConfigPath.backup_$(Get-Date -Format 'yyyyMMdd_HHmmss')"
  Copy-Item $ConfigPath $backup
  Write-Host "[OK] Backup creado: $backup"
  Write-Host ""

  # Reparar espacios en ActiveMods
  if ($content -match 'ActiveMods\s*=\s*([0-9,\s]+)') {
    $modsString = $matches[1] -replace '\s+', ''
    $newContent = $content -replace 'ActiveMods\s*=\s*[0-9,\s]+', "ActiveMods=$modsString"
  } else {
    $newContent = $content
  }

  # Eliminar 'mod id 0' de ActiveMods
  $newContent = $newContent -replace 'ActiveMods\s*=\s*0,', 'ActiveMods='
  $newContent = $newContent -replace 'ActiveMods\s*=\s*0`$', 'ActiveMods='

  # Escribir archivo reparado
  Set-Content -Path $ConfigPath -Value $newContent -Encoding ASCII

  Write-Host "[OK] Archivo reparado: $ConfigPath"
  Write-Host ""
  Write-Host "[IMPORTANTE] Reinicia el servidor para que los cambios tomen efecto"
  Write-Host ""
}

Write-Host "RECOMENDACION:"
Write-Host "Si la validacion aun falla, ejecuta DESPLEGAR.ps1 opcion 4"
Write-Host "para regenerar completamente la configuracion."
