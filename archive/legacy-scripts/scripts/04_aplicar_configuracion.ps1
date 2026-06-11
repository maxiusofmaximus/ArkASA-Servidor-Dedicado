. "$PSScriptRoot\00_utilidades.ps1"

$config = Import-AsaConfig
$configDir = Get-AsaConfigDir -Config $config
$saved = Get-AsaSavedDir -Config $config

if (Test-Path $saved) {
  & "$PSScriptRoot\03_backup_saved.ps1"
}

New-DirectoryIfMissing -Path $configDir

$gus = Join-Path $configDir 'GameUserSettings.ini'
$game = Join-Path $configDir 'Game.ini'

Set-IniValues -Path $gus -Section 'ServerSettings' -Values @{
  ServerPassword = $config.ServerPassword
  ServerAdminPassword = $config.AdminPassword
  ServerPVE = ConvertTo-ServerBool $config.ServerPve
  AllowThirdPersonPlayer = 'true'
  ShowMapPlayerLocation = 'true'
  ServerCrosshair = 'true'
  RCONEnabled = ConvertTo-ServerBool $config.EnableRcon
  RCONPort = $config.RconPort
  MaxPlayers = $config.MaxPlayers
  XPMultiplier = $config.XpMultiplier
  TamingSpeedMultiplier = $config.TamingSpeedMultiplier
  HarvestAmountMultiplier = $config.HarvestAmountMultiplier
}

Set-IniValues -Path $game -Section '/script/shootergame.shootergamemode' -Values @{
  BabyMatureSpeedMultiplier = $config.BabyMatureSpeedMultiplier
  EggHatchSpeedMultiplier = $config.EggHatchSpeedMultiplier
  MatingIntervalMultiplier = $config.MatingIntervalMultiplier
}

Write-Host "Configuracion aplicada en: $configDir"
