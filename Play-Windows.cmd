@echo off
setlocal
set "player=%~dp0bad_apple.exe"
if not exist "%player%" set "player=%~dp0target\release\bad_apple.exe"
if not exist "%player%" (
  echo Build first: cargo build --locked --release
  pause
  exit /b 1
)
start "" "%player%" --mode windows %*
