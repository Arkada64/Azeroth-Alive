# Append RAM samples of the running servers to logs\measure-<Label>.txt. Usage: sample.ps1 -Label bots50 -Minutes 5
param([string]$Label = 'run', [int]$Minutes = 5, [int]$Every = 20)
$out = "C:\SoloWoW\logs\measure-$Label.txt"
$end = (Get-Date).AddMinutes($Minutes)
while ((Get-Date) -lt $end) {
    $w = Get-Process worldserver -ErrorAction SilentlyContinue
    if (-not $w) { Add-Content $out "[$(Get-Date -f HH:mm:ss)] worldserver not running"; break }
    $up = ((Get-Date) - $w.StartTime).TotalSeconds
    $m = (Get-Process mysqld | Where-Object { (Get-CimInstance Win32_Process -Filter "ProcessId=$($_.Id)").CommandLine -like '*SoloWoW*' } | Measure-Object WorkingSet64 -Sum).Sum / 1MB
    Add-Content $out ("[{0:HH:mm:ss}] up {1,4:n0}s  world WS {2:n0} MB  mysqld {3:n0} MB  CPU {4:n0}s" -f (Get-Date), $up, ($w.WorkingSet64/1MB), $m, $w.CPU)
    Start-Sleep $Every
}
