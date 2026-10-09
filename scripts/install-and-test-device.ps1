# Install the release APK on a connected Android device and smoke-test it.
#
# Requires USB debugging enabled and the RSA dialog accepted on the phone.
# Launches the app, waits, then captures logcat + a screenshot for evidence.

param(
    [string]$Apk = "",
    [string]$Package = "com.arkasa.mobile",
    [string]$Activity = "com.arkasa.mobile/.MainActivity",
    [int]$WaitSeconds = 6
)

$ErrorActionPreference = 'Continue'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$adb = Join-Path $env:LOCALAPPDATA 'Android\Sdk\platform-tools\adb.exe'
if (-not (Test-Path $adb)) { throw "adb not found at $adb" }

if (-not $Apk) {
    $Apk = Join-Path $repoRoot 'android-app\app\build\outputs\apk\release\app-release.apk'
}
if (-not (Test-Path $Apk)) { throw "APK not found at $Apk - run assembleRelease first" }

Write-Host "== Devices =="
& $adb devices -l

$serial = (& $adb devices | Select-String -Pattern '\tdevice$' | ForEach-Object { ($_ -split '\t')[0] } | Select-Object -First 1)
if (-not $serial) {
    throw "No authorised device. Enable USB debugging and accept the RSA prompt on the phone."
}
Write-Host "Using device: $serial"

Write-Host "== Installing $Apk =="
& $adb -s $serial install -r $Apk

Write-Host "== Clearing logcat =="
& $adb -s $serial logcat -c

Write-Host "== Launching $Activity =="
& $adb -s $serial shell am start -n $Activity

Start-Sleep -Seconds $WaitSeconds

Write-Host "== Rust / native / crash lines in logcat =="
& $adb -s $serial logcat -d | Select-String -Pattern 'ark_mobile_facade|UnsatisfiedLink|AndroidRuntime|FATAL|uniffi|com\.arkasa' |
    Select-Object -Last 40

Write-Host "== Process alive? =="
& $adb -s $serial shell pidof $Package

$safeSerial = $serial -replace '[:.]', '_'
$shot = Join-Path $env:TEMP "ark-asa-device-$safeSerial.png"
Write-Host "== Screenshot -> $shot =="
# PowerShell cannot redirect raw bytes to a path containing ':'; use cmd.exe.
cmd /c "`"$adb`" -s $serial exec-out screencap -p > `"$shot`""

Write-Host ""
Write-Host "Open the screenshot above to confirm the dashboard rendered the Rust profile list."
