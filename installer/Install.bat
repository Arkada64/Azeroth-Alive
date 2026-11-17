@echo off
rem Azeroth Alive - one-click install. See docs\INSTALLER.md.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1" %*
pause
