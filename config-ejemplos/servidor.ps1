# Configuracion central del kit ASA.
# Copia este archivo a config\servidor.ps1 si quieres personalizar sin tocar ejemplos.

$AsaConfig = @{
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
