# SoloWoW - clean shutdown: save, stop worldserver via SOAP, stop authserver. -StopMysql also stops MySQL.
param([switch]$StopMysql)
. C:\SoloWoW\scripts\common.ps1
if (Test-Port 8085) {
    try { Invoke-Soap '.saveall' | Out-Null; Invoke-Soap '.server shutdown 1' | Out-Null } catch { Write-Host "SOAP failed: $_" }
    $t = 0; while ((Get-Process worldserver -ErrorAction SilentlyContinue) -and $t -lt 60) { Start-Sleep 1; $t++ }
    Get-Process worldserver -ErrorAction SilentlyContinue | Stop-Process -Force
}
Get-Process authserver -ErrorAction SilentlyContinue | Stop-Process -Force
if ($StopMysql) { Sql 'SHUTDOWN;' | Out-Null }
Write-Host 'stopped'
