# Run as Administrator.
# Removes generic ARK inbound rules (allow-all-ports created automatically by Windows)
# and replaces them with port-specific rules.
# RCON (27020) is intentionally NOT opened to the internet.

Remove-NetFirewallRule -DisplayName "ARK: Survival Ascended" -ErrorAction SilentlyContinue
Remove-NetFirewallRule -DisplayName "ArkAscendedServer"       -ErrorAction SilentlyContinue
Remove-NetFirewallRule -DisplayName "ark-asa-config.exe"      -ErrorAction SilentlyContinue

# Game ports — open to internet
New-NetFirewallRule -DisplayName "ARK-ASA-7777-Game-IN"   -Direction Inbound -Protocol UDP -LocalPort 7777  -Action Allow -Enabled True | Out-Null
New-NetFirewallRule -DisplayName "ARK-ASA-7778-Peer-IN"   -Direction Inbound -Protocol UDP -LocalPort 7778  -Action Allow -Enabled True | Out-Null
New-NetFirewallRule -DisplayName "ARK-ASA-27015-Query-IN" -Direction Inbound -Protocol UDP -LocalPort 27015 -Action Allow -Enabled True | Out-Null

# RCON — Tailscale and localhost only (NEVER open to internet)
New-NetFirewallRule -DisplayName "ARK-RCON-27020-Tailscale" -Direction Inbound -Protocol TCP -LocalPort 27020 -RemoteAddress "100.64.0.0/255.192.0.0" -Action Allow -Enabled True | Out-Null
New-NetFirewallRule -DisplayName "ARK-RCON-27020-Localhost"  -Direction Inbound -Protocol TCP -LocalPort 27020 -RemoteAddress "127.0.0.1"               -Action Allow -Enabled True | Out-Null

Write-Host "Done. ARK firewall rules are now port-specific. RCON (27020) restricted to Tailscale + localhost."
