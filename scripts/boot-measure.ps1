# SoloWoW - start authserver + worldserver hidden, time until the world port listens,
# then sample RAM. Leaves servers running. Usage: boot-measure.ps1 -Label bots50 -SampleSec 120
param([string]$Label = 'run', [int]$SampleSec = 90)
. C:\SoloWoW\scripts\common.ps1
$env:AC_DISABLE_INTERACTIVE = '1'   # never block on 'create database?' prompts
$out = "C:\SoloWoW\logs\measure-$Label.txt"
function Log($s) { $l = "[{0:HH:mm:ss}] $s" -f (Get-Date); Write-Host $l; Add-Content $out $l }

if (Test-Port 8085) { Log 'worldserver already running - stop it first'; exit 1 }
if (-not (Test-Port 3724)) {
    Start-Process "$Bin\authserver.exe" -ArgumentList '-c', "$Bin\configs\authserver.conf" -WorkingDirectory $Bin -WindowStyle Hidden `
        -RedirectStandardOutput 'C:\SoloWoW\logs\auth.out' -RedirectStandardError 'C:\SoloWoW\logs\auth.err'
}
$t0 = Get-Date
$w = Start-Process "$Bin\worldserver.exe" -ArgumentList '-c', "$Bin\configs\worldserver.conf" -WorkingDirectory $Bin -WindowStyle Hidden -PassThru `
        -RedirectStandardOutput 'C:\SoloWoW\logs\world.out' -RedirectStandardError 'C:\SoloWoW\logs\world.err'
Log "worldserver PID $($w.Id) started ($Label)"
while (-not (Test-Port 8085)) {
    if ($w.HasExited) { Log "worldserver EXITED with code $($w.ExitCode) - see logs\world.out"; exit 1 }
    Start-Sleep -Milliseconds 250
}
$ready = ((Get-Date) - $t0).TotalSeconds
Log ("READY: world port open after {0:n1} s" -f $ready)

$myd = Get-Process mysqld -ErrorAction SilentlyContinue | Where-Object { $_.Path -like '*MySQL Server 8.4*' } |
       Where-Object { (Get-CimInstance Win32_Process -Filter "ProcessId=$($_.Id)").CommandLine -like '*SoloWoW*' }
$peak = 0
for ($s = 0; $s -le $SampleSec; $s += 15) {
    $w.Refresh()
    if ($w.HasExited) { Log 'worldserver exited during sampling'; break }
    $wm = $w.WorkingSet64 / 1MB; $wp = $w.PrivateMemorySize64 / 1MB
    $mm = if ($myd) { ($myd | ForEach-Object { $_.Refresh(); $_.WorkingSet64 } | Measure-Object -Sum).Sum / 1MB } else { 0 }
    $am = (Get-Process authserver -ErrorAction SilentlyContinue | Measure-Object WorkingSet64 -Sum).Sum / 1MB
    $tot = $wm + $mm + $am; if ($tot -gt $peak) { $peak = $tot }
    Log ("t+{0,3}s  world WS {1:n0} MB (private {2:n0})  mysqld {3:n0} MB  auth {4:n0} MB  total {5:n0} MB" -f $s, $wm, $wp, $mm, $am, $tot)
    if ($s -lt $SampleSec) { Start-Sleep 15 }
}
Log ("SUMMARY $Label : ready {0:n1} s, peak server RAM {1:n0} MB" -f $ready, $peak)
