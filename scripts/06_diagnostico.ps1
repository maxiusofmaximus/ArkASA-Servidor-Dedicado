. "$PSScriptRoot\00_utilidades.ps1"

$config = Import-AsaConfig
$steamCmd = Get-SteamCmdExe -Config $config
$serverExe = Get-AsaServerExe -Config $config
$saved = Get-AsaSavedDir -Config $config
$configDir = Get-AsaConfigDir -Config $config

Write-Host 'Diagnostico ASA'
Write-Host '==============='
Write-Host "SteamCMD:      $(if (Test-Path $steamCmd) { 'OK' } else { 'FALTA' }) - $steamCmd"
Write-Host "Servidor:      $(if (Test-Path $serverExe) { 'OK' } else { 'FALTA' }) - $serverExe"
Write-Host "Saved:         $(if (Test-Path $saved) { 'OK' } else { 'FALTA' }) - $saved"
Write-Host "Config:        $(if (Test-Path $configDir) { 'OK' } else { 'FALTA' }) - $configDir"
Write-Host "Mapa:          $($config.Map)"
Write-Host "Nombre:        $($config.SessionName)"
Write-Host "Puertos:       UDP $($config.Port), UDP $($config.Port + 1), UDP $($config.QueryPort), TCP $($config.RconPort) si RCON esta activo"
Write-Host ''

Write-Host 'IP local detectada:'
try {
  Get-NetIPAddress -AddressFamily IPv4 |
    Where-Object { $_.IPAddress -notlike '127.*' -and $_.PrefixOrigin -ne 'WellKnown' } |
    Select-Object IPAddress, InterfaceAlias |
    Format-Table -AutoSize
} catch {
  Write-Host 'No se pudo leer con Get-NetIPAddress. Salida de ipconfig:'
  ipconfig | Select-String -Pattern 'IPv4|Adaptador|adapter|Gateway|Puerta'
}
