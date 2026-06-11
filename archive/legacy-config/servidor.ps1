# Configuracion central del kit ASA.
# Copia este archivo a config\servidor.ps1 si quieres personalizar sin tocar ejemplos.
# NOTA: Todos estos valores se pueden configurar también en DESPLEGAR.ini

$AsaConfig = @{
  # ===== PATHS Y BASICOS =====
  SteamCmdDir = 'C:\ASA\steamcmd'
  ServerDir = 'C:\ASA\server'
  BackupDir = 'C:\ASA\backups'
  AppId = '2430930'

  # ===== IDENTIFICACIÓN =====
  Map = 'TheIsland_WP'
  SessionName = 'ServidorMax'
  ServerPassword = 'bhahyvdhavd9954485'
  AdminPassword = 'Bafbv/aHdvhZ*w956545*'
  MaxPlayers = 70

  # ===== PUERTOS =====
  Port = 7777
  QueryPort = 27015
  RconPort = 27020
  ServerPlatform = 'ALL'
  EnableRcon = $false

  # ===== MODO DE SERVIDOR =====
  ServerPve = $true

  # ===== MODS (IDs de CurseForge) =====
  # IMPORTANTE: Sin espacios después de comas (955131,1102729,1306435 NO 955131, 1102729, 1306435)
  # OFICIAL: Los mods deben estar en [ServerSettings] de GameUserSettings.ini como: ActiveMods=ID1,ID2,ID3
  ActiveMods = '955131,1102729,1306435,958001,1182795,932756,930494,1262693,928650,953154,947033'
  
  # ===== SPAWN DE DINOS (Multiplicador de spawn) =====
  # 2.0 = doble spawn de dinos | 1.0 = normal | 0.5 = mitad
  DinoCountMultiplier = 2.0

  # ===== CONFIGURACIÓN GENERAL =====
  DifficultyOffset = 2
  MaxDifficulty = $false
  DayCycleSpeedScale = 0.50387001
  DayTimeSpeedScale = 1.57114995
  NightTimeSpeedScale = 1.45879996
  StartTimeHour = 10.0000095

  # ===== CRYOPOD SETTINGS =====
  DisableCryopodFridgeRequirement = $true
  DisableCryopodEnemyCheck = $true
  EnableCryoSicknessPVE = $false

  # ===== UI Y GAMEPLAY =====
  ShowMapPlayerLocation = $true
  ServerCrosshair = $true
  AllowThirdPersonPlayer = $true
  ShowFloatingDamageText = $true
  ProximityChat = $false
  GlobalVoiceChat = $false
  AllowHitMarkers = $true

  # ===== MULTIPLICADORES DE TAMING =====
  TamingSpeedMultiplier = 15
  BabyMatureSpeedMultiplier = 40
  BabyImprintingStatScaleMultiplier = 2.03296995
  BabyCuddleIntervalMultiplier = 0.0739300027
  BabyCuddleGracePeriodMultiplier = 0.999989986
  BabyCuddleLoseImprintQualitySpeedMultiplier = 0.999989986
  EggHatchSpeedMultiplier = 20
  LayEggIntervalMultiplier = 5.0387702
  MatingIntervalMultiplier = 2.97509003
  PoopIntervalMultiplier = 0.999989986
  BabyFoodConsumptionSpeedMultiplier = 3.99998999

  # ===== RECURSOS Y COSECHA =====
  HarvestAmountMultiplier = 8
  CropGrowthSpeedMultiplier = 20
  CropDecaySpeedMultiplier = 6
  ResourcesRespawnPeriodMultiplier = 1.00112998
  ResourceNoReplenishRadiusPlayers = 0.50158
  ResourceNoReplenishRadiusStructures = 0.999989986

  # ===== DAÑO Y RESISTENCIA =====
  PlayerDamageMultiplier = 3
  DinoDamageMultiplier = 0.999989986
  StructureDamageMultiplier = 0.999989986
  PlayerResistanceMultiplier = 0.999989986
  DinoResistanceMultiplier = 0.999989986
  StructureResistanceMultiplier = 0.263700008
  DinoTurretDamageMultiplier = 0.999989986
  DinoHarvestingDamageMultiplier = 5
  PlayerHarvestingDamageMultiplier = 7

  # ===== XP Y CRAFTING =====
  XpMultiplier = 3
  KillXPMultiplier = 2.99998999
  HarvestXPMultiplier = 2.99998999
  CraftXPMultiplier = 2.99998999
  GenericXPMultiplier = 2.99998999
  SpecialXPMultiplier = 2.99998999
  ExplorerNoteXPMultiplier = 1.99998999
  BossKillXPMultiplier = 0.999989986
  AlphaKillXPMultiplier = 0.999989986
  WildKillXPMultiplier = 0.999989986
  CaveKillXPMultiplier = 0.999989986
  TamedKillXPMultiplier = 0.999989986
  UnclaimedKillXPMultiplier = 0.999989986
  CraftingSkillBonusMultiplier = 3
  CustomRecipeEffectivenessMultiplier = 2.99998999
  CustomRecipeSkillMultiplier = 2.99998999

  # ===== CONSUMO (JUGADOR) =====
  PlayerCharacterWaterDrainMultiplier = 0.251819998
  PlayerCharacterFoodDrainMultiplier = 0.245879993
  PlayerCharacterStaminaDrainMultiplier = 1.25432003
  PlayerCharacterHealthRecoveryMultiplier = 3

  # ===== CONSUMO (DINO) =====
  DinoCharacterFoodDrainMultiplier = 5
  DinoCharacterStaminaDrainMultiplier = 0.24786
  DinoCharacterHealthRecoveryMultiplier = 3

  # ===== TIEMPOS DE DESCOMPOSICIÓN =====
  GlobalSpoilingTimeMultiplier = 0
  GlobalItemDecompositionTimeMultiplier = 0
  GlobalCorpseDecompositionTimeMultiplier = 6
  PvEStructureDecayPeriodMultiplier = 0.999989986
  StructurePickupHoldDuration = 0.5
  StructurePickupTimeAfterPlacement = 30

  # ===== COMBUSTIBLE Y LOOT =====
  FuelConsumptionIntervalMultiplier = 3.99975991
  SupplyCrateLootQualityMultiplier = 2
  FishingLootQualityMultiplier = 19.9999599

  # ===== PVP =====
  PvPZoneStructureDamageMultiplier = 6
  bIncreasePvPRespawnInterval = $true
  IncreasePvPRespawnIntervalCheckPeriod = 300.000031
  IncreasePvPRespawnIntervalMultiplier = 1.99998999
  IncreasePvPRespawnIntervalBaseAmount = 59.9999809
  StructureDamageRepairCooldown = 1.69646001

  # ===== LÍMITES =====
  MaxTamedDinos = 5000
  TheMaxStructuresInRange = 10500

  # ===== PERMISOS Y RESTRICCIONES =====
  bDisableFriendlyFire = $true
  bPvEAllowTribeWar = $true
  bPvEAllowTribeWarCancel = $false
  bAllowCustomRecipes = $true
  bAllowSpeedLeveling = $false
  bAllowFlyerSpeedLeveling = $false
  bDisableDinoRiding = $false
  bDisableDinoTaming = $false
  bDisableDefaultDinoTaming = $false
  bHardLimitTurretsInRange = $true
  bDisableStructurePlacementCollision = $true
  bAllowUnlimitedRespecs = $true
  bAllowPlatformSaddleMultiFloors = $false
  bPassiveDefensesDamageRiderlessDinos = $true
  AllowRaidDinoFeeding = $true
  bAllowRidingDinosInsideBunkers = $true
  bDisableLootCrates = $false

  # ===== MULTIPLICADORES DE STATS POR NIVEL =====
  # Jugador
  PerLevelStatsMultiplier_Player_0 = 2.99998999
  PerLevelStatsMultiplier_Player_1 = 2.99998999
  PerLevelStatsMultiplier_Player_2 = 2.99998999
  PerLevelStatsMultiplier_Player_3 = 2.99998999
  PerLevelStatsMultiplier_Player_4 = 2.99998999
  PerLevelStatsMultiplier_Player_5 = 2.99998999
  PerLevelStatsMultiplier_Player_6 = 2.99998999
  PerLevelStatsMultiplier_Player_7 = 2.99998999
  PerLevelStatsMultiplier_Player_8 = 2.99998999
  PerLevelStatsMultiplier_Player_9 = 2.99998999
  PerLevelStatsMultiplier_Player_10 = 5

  # Dino Salvaje
  PerLevelStatsMultiplier_DinoWild_0 = 0.999989986
  PerLevelStatsMultiplier_DinoWild_1 = 0.995419979
  PerLevelStatsMultiplier_DinoWild_2 = 0.999989986
  PerLevelStatsMultiplier_DinoWild_3 = 0.999989986
  PerLevelStatsMultiplier_DinoWild_4 = 0.999989986
  PerLevelStatsMultiplier_DinoWild_5 = 0.999989986
  PerLevelStatsMultiplier_DinoWild_6 = 0.999989986
  PerLevelStatsMultiplier_DinoWild_7 = 0.999989986
  PerLevelStatsMultiplier_DinoWild_8 = 0.999989986
  PerLevelStatsMultiplier_DinoWild_9 = 0.999989986
  PerLevelStatsMultiplier_DinoWild_10 = 0.999989986

  # Dino Domado
  PerLevelStatsMultiplier_DinoTamed_0 = 2.99998999
  PerLevelStatsMultiplier_DinoTamed_1 = 6
  PerLevelStatsMultiplier_DinoTamed_2 = 2.99998999
  PerLevelStatsMultiplier_DinoTamed_3 = 5
  PerLevelStatsMultiplier_DinoTamed_4 = 5
  PerLevelStatsMultiplier_DinoTamed_5 = 2.99998999
  PerLevelStatsMultiplier_DinoTamed_6 = 2.99998999
  PerLevelStatsMultiplier_DinoTamed_7 = 6
  PerLevelStatsMultiplier_DinoTamed_8 = 1.28339005
  PerLevelStatsMultiplier_DinoTamed_9 = 1.99998999
  PerLevelStatsMultiplier_DinoTamed_10 = 2.99998999

  # Dino Domado - Add
  PerLevelStatsMultiplier_DinoTamed_Add_0 = 2.99998999
  PerLevelStatsMultiplier_DinoTamed_Add_1 = 5
  PerLevelStatsMultiplier_DinoTamed_Add_2 = 2.99998999
  PerLevelStatsMultiplier_DinoTamed_Add_3 = 5
  PerLevelStatsMultiplier_DinoTamed_Add_4 = 5
  PerLevelStatsMultiplier_DinoTamed_Add_5 = 2.99998999
  PerLevelStatsMultiplier_DinoTamed_Add_6 = 2.99998999
  PerLevelStatsMultiplier_DinoTamed_Add_7 = 5
  PerLevelStatsMultiplier_DinoTamed_Add_8 = 2.01574993
  PerLevelStatsMultiplier_DinoTamed_Add_9 = 2.99998999
  PerLevelStatsMultiplier_DinoTamed_Add_10 = 2.99998999

  # Dino Domado - Affinity
  PerLevelStatsMultiplier_DinoTamed_Affinity_0 = 5
  PerLevelStatsMultiplier_DinoTamed_Affinity_1 = 6
  PerLevelStatsMultiplier_DinoTamed_Affinity_2 = 6
  PerLevelStatsMultiplier_DinoTamed_Affinity_3 = 6
  PerLevelStatsMultiplier_DinoTamed_Affinity_4 = 6
  PerLevelStatsMultiplier_DinoTamed_Affinity_5 = 6
  PerLevelStatsMultiplier_DinoTamed_Affinity_6 = 6
  PerLevelStatsMultiplier_DinoTamed_Affinity_7 = 6
  PerLevelStatsMultiplier_DinoTamed_Affinity_8 = 0.999989986
  PerLevelStatsMultiplier_DinoTamed_Affinity_9 = 0.999989986
  PerLevelStatsMultiplier_DinoTamed_Affinity_10 = 6
}