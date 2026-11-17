@echo off
echo Saving and stopping the SoloWoW server...
powershell -NoProfile -ExecutionPolicy Bypass -File "C:\SoloWoW\scripts\stop.ps1" -StopMysql
timeout /t 3 >nul
