# SoloWoW - start the living world over (or bring back the one before).
#   reset-sim.ps1         stop the world; set the current living world aside (prev_sim_* tables); archive its chronicle.
#                         The next start builds a new world and lets it live a fortnight before you arrive.
#   reset-sim.ps1 -Undo   stop the world; throw away the current one and bring back the one that was set aside.
# Only the living world (sim_* tables) changes: your characters, their bags, gold and mail stay as they are.
# There is one level of undo: starting fresh again replaces whatever was set aside before.
param([switch]$Undo)
$ErrorActionPreference = 'Stop'
. C:\SoloWoW\scripts\common.ps1

if ((Test-Port 8085) -or (Get-Process worldserver, authserver -ErrorAction SilentlyContinue)) {
    & C:\SoloWoW\scripts\stop.ps1 | Out-Null
}
if (Get-Process worldserver -ErrorAction SilentlyContinue) { Write-Host 'The world would not stop - nothing was changed.'; exit 1 }

$MysqlD = 'C:\Program Files\MySQL\MySQL Server 8.4\bin\mysqld.exe'
if (-not (Test-Port 3310)) {
    Start-Process $MysqlD -ArgumentList '--defaults-file=C:\SoloWoW\db\my.ini' -WindowStyle Hidden
    $t = 0; while (-not (Test-Port 3310) -and $t -lt 60) { Start-Sleep 1; $t++ }
    if (-not (Test-Port 3310)) { Write-Host 'The database did not start - nothing was changed.'; exit 1 }
}

function Tables([string]$prefix) {
    @(Sql "SELECT table_name FROM information_schema.tables WHERE table_schema='acore_characters' AND table_name LIKE '$($prefix.Replace('_','\_'))%'" | Where-Object { $_ -and "$_" -notmatch 'ERROR' })
}
$current = Tables 'sim_'
$previous = Tables 'prev_sim_'
$logs = 'C:\SoloWoW\logs'
$stamp = Get-Date -Format 'yyyyMMdd-HHmm'

if ($Undo) {
    if (-not $previous) { Write-Host 'There is no earlier world to bring back.'; exit 1 }
    foreach ($t in $current) { Sql "DROP TABLE acore_characters.$t" | Out-Null }
    foreach ($t in $previous) { Sql "RENAME TABLE acore_characters.$t TO acore_characters.$($t.Substring(5))" | Out-Null }
    if (Test-Path "$logs\chronicle.txt") { Move-Item "$logs\chronicle.txt" "$logs\chronicle-discarded-$stamp.txt" -Force }
    $old = Get-ChildItem "$logs\chronicle-before-*.txt" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime | Select-Object -Last 1
    if ($old) { Move-Item $old.FullName "$logs\chronicle.txt" -Force }
    Write-Host "The earlier world is back ($($previous.Count) tables). Start the world to carry on where it was."
    exit 0
}

if (-not $current) { Write-Host 'There is no living world yet - the next start builds one.'; exit 0 }
foreach ($t in $previous) { Sql "DROP TABLE acore_characters.$t" | Out-Null }
foreach ($t in $current) { Sql "RENAME TABLE acore_characters.$t TO acore_characters.prev_$t" | Out-Null }
$left = Tables 'sim_'
if ($left) { Write-Host "Could not set aside: $($left -join ', ')"; exit 1 }
foreach ($n in 'chronicle.txt', 'sim-full.log') {
    if (Test-Path "$logs\$n") { Move-Item "$logs\$n" "$logs\$([IO.Path]::GetFileNameWithoutExtension($n))-before-$stamp$([IO.Path]::GetExtension($n))" -Force }
}
Write-Host "Fresh start ready: the old world is set aside ($($current.Count) tables). Start the world and a new one is built."
