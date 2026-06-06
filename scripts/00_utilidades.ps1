Set-StrictMode -Version Latest

$ErrorActionPreference = 'Stop'

function Get-ProjectRoot {
  return (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
}

function Import-AsaConfig {
  $root = Get-ProjectRoot
  $userConfig = Join-Path $root 'config\servidor.ps1'
  $exampleConfig = Join-Path $root 'config-ejemplos\servidor.ps1'

  if (Test-Path $userConfig) {
    . $userConfig
  } else {
    . $exampleConfig
  }

  if (-not (Get-Variable -Name AsaConfig -Scope Local -ErrorAction SilentlyContinue)) {
    throw 'No se encontro $AsaConfig en el archivo de configuracion.'
  }

  return $AsaConfig
}

function New-DirectoryIfMissing {
  param([Parameter(Mandatory)][string]$Path)

  if (-not (Test-Path $Path)) {
    New-Item -ItemType Directory -Path $Path | Out-Null
  }
}

function Get-SteamCmdExe {
  param([Parameter(Mandatory)][hashtable]$Config)
  return Join-Path $Config.SteamCmdDir 'steamcmd.exe'
}

function Get-AsaServerExe {
  param([Parameter(Mandatory)][hashtable]$Config)
  return Join-Path $Config.ServerDir 'ShooterGame\Binaries\Win64\ArkAscendedServer.exe'
}

function Get-AsaSavedDir {
  param([Parameter(Mandatory)][hashtable]$Config)
  return Join-Path $Config.ServerDir 'ShooterGame\Saved'
}

function Get-AsaConfigDir {
  param([Parameter(Mandatory)][hashtable]$Config)
  return Join-Path (Get-AsaSavedDir -Config $Config) 'Config\WindowsServer'
}

function Test-IsAdministrator {
  $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
  $principal = [Security.Principal.WindowsPrincipal]::new($identity)
  return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
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
