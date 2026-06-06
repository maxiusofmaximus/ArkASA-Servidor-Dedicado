param(
  [string]$RepoName = 'ArkASA-Servidor-Dedicado',
  [switch]$Public
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$Git = 'C:\Program Files\Git\cmd\git.exe'
$Gh = 'C:\Program Files\GitHub CLI\gh.exe'
$Visibility = if ($Public) { '--public' } else { '--private' }

if (-not (Test-Path $Git)) {
  throw "No se encontro Git en $Git"
}
if (-not (Test-Path $Gh)) {
  throw "No se encontro GitHub CLI en $Gh"
}

Set-Location $ProjectRoot

& $Gh auth status
if ($LASTEXITCODE -ne 0) {
  throw 'GitHub CLI no esta autenticado. Ejecuta: gh auth login -h github.com'
}

& $Git -c safe.directory="$ProjectRoot" status --short
& $Gh repo create $RepoName $Visibility --source . --remote origin --push

Write-Host "Repositorio creado y subido: $RepoName"
