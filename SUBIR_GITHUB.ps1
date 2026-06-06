param(
  [string]$RepoName = 'ArkASA-Servidor-Dedicado',
  [switch]$Public
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ProjectRoot = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Path }
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

& $Git -c safe.directory="$ProjectRoot" rev-parse --is-inside-work-tree | Out-Null
if ($LASTEXITCODE -ne 0) {
  throw "Esta carpeta no es un repositorio Git: $ProjectRoot"
}

& $Git -c safe.directory="$ProjectRoot" rev-parse --verify HEAD | Out-Null
if ($LASTEXITCODE -ne 0) {
  throw 'El repositorio Git no tiene commits. Haz commit antes de subir.'
}

$login = (& $Gh api user --jq '.login').Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($login)) {
  throw 'No pude leer el usuario activo de GitHub CLI.'
}

$repoFullName = "$login/$RepoName"
$remoteUrl = "https://github.com/$repoFullName.git"

& $Gh repo view $repoFullName --json nameWithOwner,url | Out-Null
if ($LASTEXITCODE -ne 0) {
  Write-Host "Creando repositorio remoto: $repoFullName"
  & $Gh repo create $repoFullName $Visibility
  if ($LASTEXITCODE -ne 0) {
    throw "No se pudo crear el repositorio remoto: $repoFullName"
  }
} else {
  Write-Host "Repositorio remoto existente: $repoFullName"
}

& $Git -c safe.directory="$ProjectRoot" remote get-url origin | Out-Null
if ($LASTEXITCODE -ne 0) {
  & $Git -c safe.directory="$ProjectRoot" remote add origin $remoteUrl
} else {
  & $Git -c safe.directory="$ProjectRoot" remote set-url origin $remoteUrl
}
if ($LASTEXITCODE -ne 0) {
  throw "No se pudo configurar origin: $remoteUrl"
}

& $Git -c safe.directory="$ProjectRoot" branch -M main
if ($LASTEXITCODE -ne 0) {
  throw 'No se pudo renombrar la rama local a main.'
}

& $Git -c safe.directory="$ProjectRoot" push -u origin main
if ($LASTEXITCODE -ne 0) {
  throw 'El push a GitHub fallo. Revisa el error anterior.'
}

Write-Host "Repositorio subido correctamente: https://github.com/$repoFullName"
