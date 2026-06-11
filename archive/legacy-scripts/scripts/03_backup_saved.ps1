. "$PSScriptRoot\00_utilidades.ps1"

$config = Import-AsaConfig
$saved = Get-AsaSavedDir -Config $config

if (-not (Test-Path $saved)) {
  throw "No se encontro la carpeta Saved: $saved"
}

New-DirectoryIfMissing -Path $config.BackupDir

$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$destination = Join-Path $config.BackupDir "Saved-$stamp"

Write-Host "Copiando backup a: $destination"
robocopy $saved $destination /E /R:2 /W:2
$code = $LASTEXITCODE

if ($code -le 7) {
  Write-Host 'Backup terminado.'
  exit 0
}

throw "Robocopy fallo con codigo $code"
