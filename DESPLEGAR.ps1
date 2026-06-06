param(
  [switch]$FirewallOnly,
  [switch]$StatusOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

function Get-AsaConfig {
  $defaults = @{
    SteamCmdDir = 'C:\ASA\steamcmd'
    ServerDir = 'C:\ASA\server'
    BackupDir = 'C:\ASA\backups'
    AppId = '2430930'
    Map = 'TheIsland_WP'
    SessionName = 'ServidorMax'
    ServerPassword = 'ClaveParaAmigos'
    AdminPassword = 'CambiaEstaClave'
    MaxPlayers = 8
    Port = 7777
    QueryPort = 27015
    RconPort = 27020
    ServerPlatform = 'ALL'
    EnableRcon = $false
    ServerPve = $true
    XpMultiplier = 2.0
    TamingSpeedMultiplier = 3.0
    HarvestAmountMultiplier = 2.0
    BabyMatureSpeedMultiplier = 2.0
    EggHatchSpeedMultiplier = 2.0
    MatingIntervalMultiplier = 0.5
  }

  $userConfig = Join-Path $ProjectRoot 'config\servidor.ps1'
  $exampleConfig = Join-Path $ProjectRoot 'config-ejemplos\servidor.ps1'
  if (Test-Path $userConfig) {
    . $userConfig
  } elseif (Test-Path $exampleConfig) {
    . $exampleConfig
  } else {
    $AsaConfig = $defaults
  }

  foreach ($key in $defaults.Keys) {
    if (-not $AsaConfig.ContainsKey($key)) {
      $AsaConfig[$key] = $defaults[$key]
    }
  }
  return $AsaConfig
}

function New-DirectoryIfMissing {
  param([Parameter(Mandatory)][string]$Path)
  if (-not (Test-Path $Path)) {
    New-Item -ItemType Directory -Path $Path | Out-Null
  }
}

function Test-IsAdministrator {
  $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
  $principal = [Security.Principal.WindowsPrincipal]::new($identity)
  return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Get-Paths {
  param([Parameter(Mandatory)][hashtable]$Config)
  return @{
    SteamCmd = Join-Path $Config.SteamCmdDir 'steamcmd.exe'
    ServerExe = Join-Path $Config.ServerDir 'ShooterGame\Binaries\Win64\ArkAscendedServer.exe'
    Saved = Join-Path $Config.ServerDir 'ShooterGame\Saved'
    ConfigDir = Join-Path $Config.ServerDir 'ShooterGame\Saved\Config\WindowsServer'
  }
}

function ConvertTo-ServerBool {
  param([bool]$Value)
  if ($Value) { return 'true' }
  return 'false'
}

function Set-IniValue {
  param(
    [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Lines,
    [Parameter(Mandatory)][string]$Section,
    [Parameter(Mandatory)][string]$Key,
    [Parameter(Mandatory)][string]$Value
  )

  $sectionPattern = '^\s*\[' + [regex]::Escape($Section) + '\]\s*$'
  $anySectionPattern = '^\s*\[.+\]\s*$'
  $keyPattern = '^\s*' + [regex]::Escape($Key) + '\s*='
  $sectionIndex = -1

  for ($i = 0; $i -lt $Lines.Count; $i++) {
    if ($Lines[$i] -match $sectionPattern) {
      $sectionIndex = $i
      break
    }
  }

  $list = [System.Collections.Generic.List[string]]::new()
  $list.AddRange($Lines)

  if ($sectionIndex -eq -1) {
    if ($list.Count -gt 0 -and $list[$list.Count - 1].Trim() -ne '') {
      $list.Add('')
    }
    $list.Add("[$Section]")
    $list.Add("$Key=$Value")
    return $list.ToArray()
  }

  $insertIndex = $list.Count
  for ($i = $sectionIndex + 1; $i -lt $list.Count; $i++) {
    if ($list[$i] -match $anySectionPattern) {
      $insertIndex = $i
      break
    }
    if ($list[$i] -match $keyPattern) {
      $list[$i] = "$Key=$Value"
      return $list.ToArray()
    }
  }

  $list.Insert($insertIndex, "$Key=$Value")
  return $list.ToArray()
}

function Set-IniValues {
  param(
    [Parameter(Mandatory)][string]$Path,
    [Parameter(Mandatory)][string]$Section,
    [Parameter(Mandatory)][hashtable]$Values
  )

  if (Test-Path $Path) {
    $lines = @(Get-Content -Path $Path)
  } else {
    $lines = @()
  }

  foreach ($key in $Values.Keys) {
    $lines = Set-IniValue -Lines $lines -Section $Section -Key $key -Value ([string]$Values[$key])
  }

  Set-Content -Path $Path -Value $lines -Encoding ASCII
}

function Get-AsaStatus {
  param([Parameter(Mandatory)][hashtable]$Config)
  $paths = Get-Paths -Config $Config
  return [ordered]@{
    SteamCMD = Test-Path $paths.SteamCmd
    Servidor = Test-Path $paths.ServerExe
    Saved = Test-Path $paths.Saved
    Configuracion = Test-Path $paths.ConfigDir
    Firewall = Test-FirewallRules -Config $Config
    IpLocal = Get-LocalIp
  }
}

function Test-FirewallRules {
  param([Parameter(Mandatory)][hashtable]$Config)
  try {
    $required = @(
      'ASA UDP Game 7777',
      'ASA UDP Game 7778',
      'ASA UDP Query'
    )
    foreach ($name in $required) {
      if (-not (Get-NetFirewallRule -DisplayName $name -ErrorAction SilentlyContinue)) {
        return $false
      }
    }
    return $true
  } catch {
    return $false
  }
}

function Get-LocalIp {
  try {
    $ip = Get-NetIPAddress -AddressFamily IPv4 -ErrorAction Stop |
      Where-Object { $_.IPAddress -notlike '127.*' -and $_.PrefixOrigin -ne 'WellKnown' } |
      Select-Object -First 1 -ExpandProperty IPAddress
    if ($ip) { return $ip }
  } catch {}

  $match = ipconfig | Select-String -Pattern 'IPv4.*?:\s*([0-9.]+)' | Select-Object -First 1
  if ($match -and $match.Matches.Count -gt 0) {
    return $match.Matches[0].Groups[1].Value
  }
  return 'No detectada'
}

function Show-Status {
  param([Parameter(Mandatory)][hashtable]$Config)
  $status = Get-AsaStatus -Config $Config
  Write-Host ''
  Write-Host 'Estado actual'
  Write-Host '-------------'
  Write-Host ("SteamCMD:       {0}" -f $(if ($status.SteamCMD) { 'OK' } else { 'FALTA' }))
  Write-Host ("Servidor ASA:   {0}" -f $(if ($status.Servidor) { 'OK' } else { 'FALTA' }))
  Write-Host ("Saved:          {0}" -f $(if ($status.Saved) { 'OK' } else { 'FALTA' }))
  Write-Host ("Configuracion:  {0}" -f $(if ($status.Configuracion) { 'OK' } else { 'FALTA' }))
  Write-Host ("Firewall:       {0}" -f $(if ($status.Firewall) { 'OK' } else { 'FALTA O SIN PERMISO' }))
  Write-Host ("IP local:       {0}" -f $status.IpLocal)
  Write-Host ("Nombre:         {0}" -f $Config.SessionName)
  Write-Host ("Mapa:           {0}" -f $Config.Map)
  Write-Host ''
}

function Install-OrUpdateServer {
  param([Parameter(Mandatory)][hashtable]$Config)
  $paths = Get-Paths -Config $Config
  $downloadedSteamCmd = $false

  New-DirectoryIfMissing -Path $Config.SteamCmdDir
  New-DirectoryIfMissing -Path $Config.ServerDir

  if (-not (Test-Path $paths.SteamCmd)) {
    $zipPath = Join-Path $env:TEMP 'steamcmd.zip'
    Write-Host 'Descargando SteamCMD...'
    Invoke-WebRequest -Uri 'https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip' -OutFile $zipPath
    Expand-Archive -Path $zipPath -DestinationPath $Config.SteamCmdDir -Force
    $downloadedSteamCmd = $true
  }

  if ($downloadedSteamCmd) {
    Write-Host 'Inicializando SteamCMD...'
    & $paths.SteamCmd +quit
  }

  Write-Host 'Instalando o actualizando servidor ASA. Esto puede tardar bastante...'
  & $paths.SteamCmd +force_install_dir $Config.ServerDir +login anonymous +app_update $Config.AppId validate +quit

  if (-not (Test-Path $paths.ServerExe)) {
    Write-Host 'Reintentando una vez porque SteamCMD pudo haberse actualizado en el primer intento...'
    & $paths.SteamCmd +force_install_dir $Config.ServerDir +login anonymous +app_update $Config.AppId validate +quit
  }

  if (-not (Test-Path $paths.ServerExe)) {
    throw "No se encontro el ejecutable del servidor: $($paths.ServerExe)"
  }
}

function Backup-Saved {
  param([Parameter(Mandatory)][hashtable]$Config)
  $paths = Get-Paths -Config $Config
  if (-not (Test-Path $paths.Saved)) {
    Write-Host 'No existe Saved todavia; no hay backup que hacer.'
    return
  }

  New-DirectoryIfMissing -Path $Config.BackupDir
  $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
  $destination = Join-Path $Config.BackupDir "Saved-$stamp"
  Write-Host "Creando backup: $destination"
  robocopy $paths.Saved $destination /E /R:2 /W:2 | Out-Host
  if ($LASTEXITCODE -gt 7) {
    throw "Robocopy fallo con codigo $LASTEXITCODE"
  }
}

function Apply-ServerConfig {
  param([Parameter(Mandatory)][hashtable]$Config)
  $paths = Get-Paths -Config $Config

  if (Test-Path $paths.Saved) {
    Backup-Saved -Config $Config
  }

  New-DirectoryIfMissing -Path $paths.ConfigDir
  Set-IniValues -Path (Join-Path $paths.ConfigDir 'GameUserSettings.ini') -Section 'ServerSettings' -Values @{
    ServerPassword = $Config.ServerPassword
    ServerAdminPassword = $Config.AdminPassword
    ServerPVE = ConvertTo-ServerBool $Config.ServerPve
    AllowThirdPersonPlayer = 'true'
    ShowMapPlayerLocation = 'true'
    ServerCrosshair = 'true'
    RCONEnabled = ConvertTo-ServerBool $Config.EnableRcon
    RCONPort = $Config.RconPort
    MaxPlayers = $Config.MaxPlayers
    XPMultiplier = $Config.XpMultiplier
    TamingSpeedMultiplier = $Config.TamingSpeedMultiplier
    HarvestAmountMultiplier = $Config.HarvestAmountMultiplier
  }
  Set-IniValues -Path (Join-Path $paths.ConfigDir 'Game.ini') -Section '/script/shootergame.shootergamemode' -Values @{
    BabyMatureSpeedMultiplier = $Config.BabyMatureSpeedMultiplier
    EggHatchSpeedMultiplier = $Config.EggHatchSpeedMultiplier
    MatingIntervalMultiplier = $Config.MatingIntervalMultiplier
  }
}

function Configure-Firewall {
  param([Parameter(Mandatory)][hashtable]$Config)
  if (-not (Test-IsAdministrator)) {
    Write-Host 'Abriendo una ventana elevada para configurar firewall...'
    Start-Process powershell -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -FirewallOnly"
    return
  }

  $rules = @(
    @{ Name = 'ASA UDP Game 7777'; Protocol = 'UDP'; Port = $Config.Port },
    @{ Name = 'ASA UDP Game 7778'; Protocol = 'UDP'; Port = ($Config.Port + 1) },
    @{ Name = 'ASA UDP Query'; Protocol = 'UDP'; Port = $Config.QueryPort }
  )
  if ($Config.EnableRcon) {
    $rules += @{ Name = 'ASA TCP RCON'; Protocol = 'TCP'; Port = $Config.RconPort }
  }

  foreach ($rule in $rules) {
    if (Get-NetFirewallRule -DisplayName $rule.Name -ErrorAction SilentlyContinue) {
      Write-Host "Ya existe: $($rule.Name)"
      continue
    }
    New-NetFirewallRule -DisplayName $rule.Name -Direction Inbound -Protocol $rule.Protocol -LocalPort $rule.Port -Action Allow | Out-Null
    Write-Host "Creada: $($rule.Name) $($rule.Protocol)/$($rule.Port)"
  }
}

function Start-AsaServer {
  param([Parameter(Mandatory)][hashtable]$Config)
  $paths = Get-Paths -Config $Config
  if (-not (Test-Path $paths.ServerExe)) {
    throw 'El servidor no esta instalado todavia.'
  }

  $rconFlag = if ($Config.EnableRcon) { 'true' } else { 'false' }
  $url = '{0}?listen?SessionName={1}?ServerPassword={2}?ServerAdminPassword={3}?MaxPlayers={4}?RCONEnabled={5}' -f `
    $Config.Map, $Config.SessionName, $Config.ServerPassword, $Config.AdminPassword, $Config.MaxPlayers, $rconFlag
  $args = @(
    $url,
    '-server',
    '-log',
    "-port=$($Config.Port)",
    "-QueryPort=$($Config.QueryPort)",
    "-RCONPort=$($Config.RconPort)",
    "-ServerPlatform=$($Config.ServerPlatform)"
  )
  & $paths.ServerExe @args
}

function Invoke-FullDeploy {
  param([Parameter(Mandatory)][hashtable]$Config)
  Install-OrUpdateServer -Config $Config
  Apply-ServerConfig -Config $Config
  Configure-Firewall -Config $Config
  Show-Status -Config $Config
  Write-Host 'Despliegue listo. Si el firewall pidio UAC, acepta la ventana elevada.'
  Write-Host 'Para jugar por internet, falta reservar la IP local en el router y redirigir UDP 7777, UDP 7778 y UDP 27015.'
}

$config = Get-AsaConfig
if ($FirewallOnly) {
  Configure-Firewall -Config $config
  Read-Host 'Pulsa Enter para cerrar'
  exit
}
if ($StatusOnly) {
  Show-Status -Config $config
  exit
}

while ($true) {
  Clear-Host
  Write-Host 'ARK: Survival Ascended - Despliegue dedicado'
  Write-Host '============================================'
  Show-Status -Config $config
  Write-Host '1. Desplegar todo lo posible ahora'
  Write-Host '2. Instalar o actualizar servidor'
  Write-Host '3. Aplicar configuracion'
  Write-Host '4. Configurar firewall'
  Write-Host '5. Iniciar servidor'
  Write-Host '6. Hacer backup de Saved'
  Write-Host '7. Ver estado'
  Write-Host '0. Salir'
  Write-Host ''
  $option = Read-Host 'Elige una opcion'

  try {
    switch ($option) {
      '1' { Invoke-FullDeploy -Config $config }
      '2' { Install-OrUpdateServer -Config $config }
      '3' { Apply-ServerConfig -Config $config }
      '4' { Configure-Firewall -Config $config }
      '5' { Start-AsaServer -Config $config }
      '6' { Backup-Saved -Config $config }
      '7' { Show-Status -Config $config }
      '0' { exit 0 }
      default { Write-Host 'Opcion no valida.' }
    }
  } catch {
    Write-Host ''
    Write-Host "Error: $($_.Exception.Message)"
  }

  Write-Host ''
  Read-Host 'Pulsa Enter para continuar'
}
