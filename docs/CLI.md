# CLI Tool Documentation

The ARK ASA Config Manager includes a powerful command-line tool for server management and configuration without needing the GUI.

## Installation

The CLI is built as part of the Rust backend:

```bash
cargo build --release --bin ark-config
```

The binary will be available at:
```
target/release/ark-config.exe
```

## Usage

### Basic Syntax
```bash
ark-config <COMMAND> [OPTIONS]
```

### Start Server

Start the server with a specific configuration file:

```bash
ark-config start config.toml
```

Outputs:
```
Starting server with config: "config.toml"
```

### Stop Server

Stop the running server:

```bash
ark-config stop
```

### Restart Server

Restart with a new or updated configuration:

```bash
ark-config restart config.toml
```

### Check Status

Get current server status:

```bash
ark-config status
```

Outputs:
```
Server: RUNNING (PID: 1234, Uptime: 1h)
```

### Install Server

Install ARK server using SteamCMD:

```bash
ark-config install C:\steamcmd C:\ASA\server
```

### Configuration Management

#### Show Current Config
```bash
ark-config config show
```

#### Edit Configuration
```bash
ark-config config edit gameplay.max_players 100
```

#### Validate Configuration
```bash
ark-config config validate
```

Outputs:
```
✓ Configuration is valid
```

#### Generate Configuration
```bash
ark-config config generate toml
ark-config config generate ini
ark-config config generate json
```

### View Logs

Show last 50 log lines:

```bash
ark-config logs 50
```

Filter logs by keyword:

```bash
ark-config logs 100 error
```

Show only warning logs:

```bash
ark-config logs 50 warning
```

### System Metrics

Display current system metrics:

```bash
ark-config metrics
```

Outputs:
```
CPU:     45.0%
Memory:  8192 MB (50%)
Network: 2.5 MB/s ↓, 1.8 MB/s ↑
```

### Backup & Restore

#### Create Backup

Create an automatic backup:

```bash
ark-config backup
```

Create a named backup:

```bash
ark-config backup pre-update-backup
```

#### Restore from Backup

Restore to a specific version:

```bash
ark-config restore 2
```

## Examples

### Workflow: Update and Restart

```bash
# Show current config
ark-config config show

# Edit max players
ark-config config edit gameplay.max_players 100

# Validate changes
ark-config config validate

# Create backup before restart
ark-config backup before-restart

# Restart server
ark-config restart config.toml

# Check status
ark-config status

# View recent logs
ark-config logs 20
```

### Workflow: Monitor Server

```bash
# Check if server is running
ark-config status

# Get real-time metrics
ark-config metrics

# Check for errors in last hour
ark-config logs 1000 error

# View warnings
ark-config logs 500 warning
```

### Workflow: Backup and Restore

```bash
# Create backup before major change
ark-config backup major-update

# View backup history
ark-config config show

# Restore if something goes wrong
ark-config restore 1
```

## Exit Codes

- `0` - Success
- `1` - Error (check output for details)

## Environment Variables

### RUST_LOG
Control log verbosity:

```bash
$env:RUST_LOG = "debug"
ark-config status
```

Levels: `error`, `warn`, `info` (default), `debug`, `trace`

## Tips & Tricks

### Batch Operations

Create a PowerShell script for common tasks:

```powershell
# restart-server.ps1
Write-Host "Creating backup..."
ark-config backup pre-restart

Write-Host "Restarting server..."
ark-config restart config.toml

Start-Sleep -Seconds 5

Write-Host "Checking status..."
ark-config status
```

Then run:
```powershell
.\restart-server.ps1
```

### Scheduled Tasks

Set up Windows Task Scheduler to run periodic backups:

```powershell
# In PowerShell as Administrator
$action = New-ScheduledTaskAction -Execute "ark-config" -Argument "backup"
$trigger = New-ScheduledTaskTrigger -Daily -At 02:00AM
Register-ScheduledTask -Action $action -Trigger $trigger -TaskName "ARK Daily Backup"
```

### Log Monitoring

Stream logs continuously (requires tail or similar):

```bash
ark-config logs 100 error
# Repeat every 5 seconds
Get-Item -Path (Get-Location) | Select-Object -ExpandProperty FullName | ForEach-Object {
    Start-Sleep -Seconds 5
    ark-config logs 100 error
}
```

## Troubleshooting

### Command Not Found
Ensure `ark-config.exe` is in your PATH or run with full path:

```bash
C:\Users\Max\ArkASA-Servidor-Dedicado\target\release\ark-config.exe status
```

### Permission Denied
Make sure you have permissions to execute the binary:

```bash
# In PowerShell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Configuration Errors
Always validate before restarting:

```bash
ark-config config validate
```

If validation fails, check the error message and correct the config.

## Advanced Usage

### Custom Aliases

Create PowerShell function aliases for commonly used commands:

```powershell
# In your $PROFILE
function Start-ArkServer { ark-config start config.toml }
function Stop-ArkServer { ark-config stop }
function Get-ArkStatus { ark-config status }
function Get-ArkMetrics { ark-config metrics }
function Backup-Ark { ark-config backup }

# Now you can use:
Start-ArkServer
Get-ArkStatus
Get-ArkMetrics
```

### Log Analysis

Export logs for analysis:

```bash
# Get all errors from last run
ark-config logs 10000 error > errors.log

# Get warnings
ark-config logs 5000 warning > warnings.log
```

## Performance Notes

- Commands complete in < 100ms
- Log reading is cached for performance
- Metrics collection is non-blocking
- Backup creation runs asynchronously

---

For more information, see:
- [README.md](../README.md)
- [API.md](API.md)
- [CONTRIBUTING.md](CONTRIBUTING.md)
