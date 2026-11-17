@echo off
rem SoloWoW Manager - start/stop the database, login and world servers, play, and stop everything cleanly.
cd /d "%~dp0"
start "" powershell.exe -NoProfile -ExecutionPolicy Bypass -STA -WindowStyle Hidden -File "%~dp0scripts\manager.ps1"
