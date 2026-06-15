# DuckDNS automatic IP updater — template
# Copy this file to C:\ASA\scripts\duckdns-updater.ps1 and fill in your values.
# Then install as a scheduled task (see docs/NETWORK_SETUP.md, section "DDNS con DuckDNS").

$DUCK_DOMAIN = "tu-subdominio"                       # e.g. "ark-miservidor" (without .duckdns.org)
$DUCK_TOKEN  = "TU-TOKEN-AQUI"                       # from duckdns.org dashboard
$logFile     = "C:\ASA\scripts\duckdns.log"

$url = "https://www.duckdns.org/update?domains=$DUCK_DOMAIN&token=$DUCK_TOKEN&ip="
try {
    $response = (Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 15).Content.Trim()
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Add-Content -Path $logFile -Value "$timestamp $response"
} catch {
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Add-Content -Path $logFile -Value "$timestamp ERROR: $_"
}
