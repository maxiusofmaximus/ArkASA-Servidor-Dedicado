. "$PSScriptRoot\00_utilidades.ps1"

$config = Import-AsaConfig

if (-not (Test-IsAdministrator)) {
  throw 'Ejecuta este script en PowerShell como administrador para crear reglas de firewall.'
}

$rules = @(
  @{ Name = 'ASA UDP Game 7777'; Protocol = 'UDP'; Port = $config.Port },
  @{ Name = 'ASA UDP Game 7778'; Protocol = 'UDP'; Port = ($config.Port + 1) },
  @{ Name = 'ASA UDP Query'; Protocol = 'UDP'; Port = $config.QueryPort }
)

if ($config.EnableRcon) {
  $rules += @{ Name = 'ASA TCP RCON'; Protocol = 'TCP'; Port = $config.RconPort }
}

foreach ($rule in $rules) {
  $existing = Get-NetFirewallRule -DisplayName $rule.Name -ErrorAction SilentlyContinue
  if ($existing) {
    Write-Host "Ya existe regla: $($rule.Name)"
    continue
  }

  New-NetFirewallRule `
    -DisplayName $rule.Name `
    -Direction Inbound `
    -Protocol $rule.Protocol `
    -LocalPort $rule.Port `
    -Action Allow | Out-Null

  Write-Host "Regla creada: $($rule.Name) $($rule.Protocol)/$($rule.Port)"
}
