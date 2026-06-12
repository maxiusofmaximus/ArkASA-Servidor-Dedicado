@echo off
cd /d "%~dp0"
echo Building frontend...
call pnpm --dir frontend build
if %errorlevel% neq 0 (
    echo FRONTEND BUILD FAILED
    pause
    exit /b 1
)
echo Building ark-asa-config DEBUG build...
call cargo build --manifest-path src-tauri\Cargo.toml --bin ark-asa-config
if %errorlevel% neq 0 (
    echo BUILD FAILED
    pause
) else (
    echo BUILD SUCCESS - binary at src-tauri\target\debug\ark-asa-config.exe
    pause
)
