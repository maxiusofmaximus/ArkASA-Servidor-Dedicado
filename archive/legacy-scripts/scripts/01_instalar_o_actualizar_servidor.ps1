. "$PSScriptRoot\00_utilidades.ps1"

$config = Import-AsaConfig
$steamCmd = Get-SteamCmdExe -Config $config
$downloadedSteamCmd = $false

New-DirectoryIfMissing -Path $config.SteamCmdDir
New-DirectoryIfMissing -Path $config.ServerDir

if (-not (Test-Path $steamCmd)) {
  $zipPath = Join-Path $env:TEMP 'steamcmd.zip'
  $extractPath = $config.SteamCmdDir

  Write-Host 'Descargando SteamCMD desde Valve...'
  Invoke-WebRequest -Uri 'https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip' -OutFile $zipPath
  Expand-Archive -Path $zipPath -DestinationPath $extractPath -Force
  $downloadedSteamCmd = $true
}

if (-not (Test-Path $steamCmd)) {
  throw "No se encontro SteamCMD en $steamCmd"
}

if ($downloadedSteamCmd) {
  Write-Host 'Inicializando SteamCMD...'
  & $steamCmd +quit
}

Write-Host 'Instalando o actualizando ARK: Survival Ascended Dedicated Server...'
& $steamCmd +force_install_dir $config.ServerDir +login anonymous +app_update $config.AppId validate +quit

$serverExe = Get-AsaServerExe -Config $config
if (-not (Test-Path $serverExe)) {
  Write-Host 'No aparecio el ejecutable despues del primer intento. Reintentando una vez...'
  & $steamCmd +force_install_dir $config.ServerDir +login anonymous +app_update $config.AppId validate +quit
}

if (Test-Path $serverExe) {
  Write-Host "Servidor listo: $serverExe"
} else {
  throw "SteamCMD termino, pero no se encontro: $serverExe"
}
