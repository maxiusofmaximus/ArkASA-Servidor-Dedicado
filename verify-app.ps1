# Automatic Application Verification Script

Write-Host "=== AUTOMATIC VERIFICATION ===" -ForegroundColor Cyan
Write-Host "Starting app and checking logs automatically..." -ForegroundColor Yellow

# Kill old processes
Get-Process ark-asa-config -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Get-Process node -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -like "*vite*" } | Stop-Process -Force -ErrorAction SilentlyContinue
Get-Process cargo -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

Write-Host "[1] Starting development server..." -ForegroundColor Yellow
$devProcess = Start-Job -ScriptBlock {
    cd 'C:\Users\Max\ArkASA-Servidor-Dedicado'
    pnpm tauri:dev 2>&1
} -Name "TauriDev"

Write-Host "Development server started" -ForegroundColor Green

Write-Host "[2] Waiting 20 seconds for app to load..." -ForegroundColor Yellow
for ($i = 1; $i -le 20; $i++) {
    Write-Progress -Activity "Loading" -PercentComplete (($i / 20) * 100) -Status "$i/20 seconds"
    Start-Sleep -Seconds 1
}
Write-Progress -Activity "Loading" -Completed

Write-Host "[3] Checking processes..." -ForegroundColor Yellow
$arkProcess = Get-Process ark-asa-config -ErrorAction SilentlyContinue
$nodeProcess = Get-Process node -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -like "*vite*" }

if ($arkProcess) {
    $mem = [Math]::Round($arkProcess.WorkingSet / 1MB, 1)
    Write-Host "PASS: ark-asa-config running (PID: $($arkProcess.Id), $mem MB)" -ForegroundColor Green
} else {
    Write-Host "FAIL: ark-asa-config NOT running" -ForegroundColor Red
}

if ($nodeProcess) {
    $mem = [Math]::Round($nodeProcess.WorkingSet / 1MB, 1)
    Write-Host "PASS: Vite dev server running (PID: $($nodeProcess.Id), $mem MB)" -ForegroundColor Green
} else {
    Write-Host "FAIL: Vite dev server NOT running" -ForegroundColor Red
}

Write-Host "[4] Testing HTTP connectivity..." -ForegroundColor Yellow
try {
    $response = curl -s -m 5 http://localhost:5173/ 2>&1
    if ($response) {
        $size = $response.Length
        Write-Host "PASS: Frontend responding at http://localhost:5173 ($size bytes)" -ForegroundColor Green
    } else {
        Write-Host "FAIL: Frontend not responding" -ForegroundColor Red
    }
} catch {
    Write-Host "FAIL: Frontend not responding - $_" -ForegroundColor Red
}

Write-Host "[5] Checking build output..." -ForegroundColor Yellow
$output = Receive-Job -Job $devProcess -Keep 2>&1 | Select-Object -Last 30
if ($output -match "error|Error|ERROR") {
    Write-Host "FAIL: Errors detected in output" -ForegroundColor Red
    $output | Where-Object { $_ -match "error|Error|ERROR" } | ForEach-Object { Write-Host "  $_" -ForegroundColor Yellow }
} else {
    Write-Host "PASS: No critical errors detected" -ForegroundColor Green
}

Write-Host "`n=== VERIFICATION COMPLETE ===" -ForegroundColor Cyan
Write-Host "Frontend: http://localhost:5173/" -ForegroundColor Cyan
Write-Host "Application is running in background" -ForegroundColor Green
