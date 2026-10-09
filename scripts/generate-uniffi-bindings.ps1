# Regenerate the Kotlin (and later Swift) bindings from the built Rust library.
#
# The generated code lands in `target/uniffi-bindings/kotlin`, which the
# Gradle verification module (`jvmTest/`) compiles as production sources, so
# `jvmTest` always tests the bindings that match the current Rust code.
#
# The version is pinned deliberately: `uniffi-bindgen` MUST match the `uniffi`
# crate version in Cargo.lock, otherwise the generated Kotlin drifts from the
# Rust ABI and fails at runtime with a checksum mismatch.
#
# Prerequisites: Rust toolchain, and `uniffi-bindgen` on PATH:
#   cargo install --version 0.32.2 uniffi --features cli --locked

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$manifest = Join-Path $repoRoot 'Cargo.toml'
$dll = Join-Path $repoRoot 'target\debug\ark_mobile_facade.dll'
$outDir = Join-Path $repoRoot 'target\uniffi-bindings\kotlin'

Write-Host "Building ark-mobile-facade (cdylib) ..."
# cargo writes progress to stderr; relax the preference so the wrapper does not
# treat it as fatal, and judge success by the exit code.
$ErrorActionPreference = 'Continue'
cargo build --manifest-path $manifest -p ark-mobile-facade
$buildCode = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
if ($buildCode -ne 0) { throw "cargo build failed (exit $buildCode)" }

if (-not (Test-Path $dll)) {
    throw "Expected the cdylib at $dll but it was not produced."
}

# The bindings MUST be generated from the library that was just built. A stale
# dll silently yields bindings missing the newest API (observed: OfflineCore
# absent while the .so on disk already had it), so touch-check the timestamp.
$dllAge = (Get-Date) - (Get-Item $dll).LastWriteTime
if ($dllAge.TotalMinutes -gt 30) {
    throw "The cdylib at $dll is $([int]$dllAge.TotalMinutes) min old; the build did not refresh it."
}
Write-Host ("Using library built {0:N0}s ago" -f $dllAge.TotalSeconds)

$generator = Get-Command uniffi-bindgen -ErrorAction SilentlyContinue
if (-not $generator) {
    throw "uniffi-bindgen not found. Install the pinned version: cargo install --version 0.32.2 uniffi --features cli --locked"
}

if (Test-Path $outDir) { Remove-Item -LiteralPath $outDir -Recurse -Force }
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

Write-Host "Generating Kotlin bindings -> $outDir"
& $generator.Source generate --library $dll --language kotlin --out-dir $outDir --no-format

Write-Host "Done. Run the host-JVM verification with:"
Write-Host "  cd crates\ark-mobile-facade\jvmTest; .\gradlew.bat test"
