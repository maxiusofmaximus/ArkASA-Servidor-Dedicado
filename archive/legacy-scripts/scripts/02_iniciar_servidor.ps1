. "$PSScriptRoot\00_utilidades.ps1"

$config = Import-AsaConfig
$serverExe = Get-AsaServerExe -Config $config

if (-not (Test-Path $serverExe)) {
  throw "No se encontro el servidor en $serverExe. Ejecuta scripts\01_instalar_o_actualizar_servidor.bat primero."
}

$rconFlag = if ($config.EnableRcon) { 'true' } else { 'false' }
$url = '{0}?listen?SessionName={1}?ServerPassword={2}?ServerAdminPassword={3}?MaxPlayers={4}?RCONEnabled={5}' -f `
  $config.Map, $config.SessionName, $config.ServerPassword, $config.AdminPassword, $config.MaxPlayers, $rconFlag

$args = @(
  $url,
  '-server',
  '-log',
  "-port=$($config.Port)",
  "-QueryPort=$($config.QueryPort)",
  "-RCONPort=$($config.RconPort)",
  "-ServerPlatform=$($config.ServerPlatform)"
)

Write-Host 'Iniciando servidor ASA...'
Write-Host "Nombre: $($config.SessionName)"
Write-Host "Mapa: $($config.Map)"
Write-Host "Puerto juego UDP: $($config.Port)"
Write-Host "Query UDP: $($config.QueryPort)"
Write-Host ''

& $serverExe @args
