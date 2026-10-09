# Build the Rust core for Android and drop it into the app's jniLibs.
#
# Produces one `.so` per ABI in `android-app/app/src/main/jniLibs/<abi>/`,
# which the Android build packages. 16 KB page alignment is handled by
# NDK r28+/cargo-ndk and verified with `zipalign -c -P 16` on the APK.
#
# Prerequisites (one-off):
#   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
#   cargo install cargo-ndk --locked
#   Android SDK + NDK (set ANDROID_HOME / ANDROID_NDK_HOME, or let cargo-ndk
#   auto-detect the default SDK location).

param(
    # Compiled ABIs. arm64-v8a is the shipping target; x86_64 is kept for
    # emulator runs; armeabi-v7a stays available but is not built by default to
    # keep build time and APK size down.
    [string[]]$Abis = @('arm64-v8a', 'x86_64')
)

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$manifest = Join-Path $repoRoot 'Cargo.toml'
$jniLibs = Join-Path $repoRoot 'android-app\app\src\main\jniLibs'

foreach ($abi in $Abis) {
    Write-Host "Building ark-mobile-facade for $abi ..."
    # cargo writes progress to stderr; with $ErrorActionPreference='Stop' the
    # native-command wrapper would treat that as fatal, so relax it locally and
    # judge success by the exit code instead.
    $ErrorActionPreference = 'Continue'
    cargo ndk -t $abi -o $jniLibs build --manifest-path $manifest -p ark-mobile-facade --release
    $code = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    if ($code -ne 0) { throw "cargo ndk failed for $abi (exit $code)" }
}

Write-Host ""
Write-Host "Native libraries:"
Get-ChildItem $jniLibs -Recurse -File -Filter '*.so' |
    ForEach-Object { "  {0}  {1:N2} MB" -f $_.FullName.Substring($jniLibs.Length + 1), ($_.Length / 1MB) }

Write-Host ""
Write-Host "Next: regenerate bindings, then build the APK:"
Write-Host "  .\scripts\generate-uniffi-bindings.ps1"
Write-Host "  cd android-app; .\gradlew.bat assembleDebug"
