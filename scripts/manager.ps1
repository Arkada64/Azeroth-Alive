<#
    SoloWoW Manager - one window that owns the servers (Nov 17).
    Database (MySQL on 3310), login server, world server: status, RAM, uptime, start / stop / restart each, start all,
    stop all, play. The world shuts down cleanly over SOAP ('server shutdown 1' saves players and the living world)
    and is only force-closed after 4 minutes, with a question first. Closing the window offers to stop everything,
    so nothing is left running behind your back. The start/stop logic follows the user's own AzerothManager.
    Run it with "SoloWoW Manager.bat" next to the SoloWoW folder.
#>
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'common.ps1')                 # $Bin, Sql, Ensure-Account, Invoke-Soap, Test-Port
Add-Type -AssemblyName System.Windows.Forms, System.Drawing, Microsoft.VisualBasic
[Windows.Forms.Application]::EnableVisualStyles()

$AppName = 'SoloWoW Manager'
$Root    = 'C:\SoloWoW'
$MySqlD  = 'C:\Program Files\MySQL\MySQL Server 8.4\bin\mysqld.exe'
$MyIni   = "$Root\db\my.ini"
$Logs    = "$Root\logs"
$DbPort  = 3310
$Client  = @("$Root\WOTLK Client", "$Root\client", 'C:\WOTLK Client', 'D:\WOTLK Client') | Where-Object { Test-Path (Join-Path $_ 'Wow.exe') } | Select-Object -First 1

# only one manager at a time: two would fight over the same servers
$mutex = New-Object Threading.Mutex($false, 'SoloWoWManager')
if (-not $mutex.WaitOne(0)) { [Windows.Forms.MessageBox]::Show('The SoloWoW Manager is already open.', $AppName) | Out-Null; return }

function Show-Msg([string]$text, [string]$icon = 'Information', [string]$buttons = 'OK') { [Windows.Forms.MessageBox]::Show($text, $AppName, $buttons, $icon) }
function Safe([scriptblock]$sb) { try { & $sb } catch { Show-Msg $_.Exception.Message 'Error' | Out-Null } }
function New-Ctl([string]$type, [hashtable]$props) { $c = New-Object "System.Windows.Forms.$type"; foreach ($k in $props.Keys) { $c.$k = $props[$k] }; $c }
function Pt([int]$x, [int]$y) { New-Object Drawing.Point($x, $y) }
function Sz([int]$w, [int]$h) { New-Object Drawing.Size($w, $h) }
function Pump { [Windows.Forms.Application]::DoEvents() }
function Wait-Until([scriptblock]$cond, [int]$seconds) {
    $end = (Get-Date).AddSeconds($seconds)
    while ((Get-Date) -lt $end) { if (& $cond) { return $true }; Pump; Start-Sleep -Milliseconds 200 }
    & $cond
}

# ---------------------------------------------------------------- processes
$Servers = [ordered]@{
    auth  = @{ Name = 'Login server'; Exe = 'authserver.exe';  Conf = 'authserver.conf';  Log = 'Auth.log';   Port = 3724; Proc = $null; Desired = $false; LogPos = -1 }
    world = @{ Name = 'World server'; Exe = 'worldserver.exe'; Conf = 'worldserver.conf'; Log = 'Server.log'; Port = 8085; Proc = $null; Desired = $false; LogPos = -1 }
}
function Test-Alive($p) { if (-not $p) { return $false }; try { -not $p.HasExited } catch { $true } }
function Get-ServerProc([string]$k) {
    $s = $Servers[$k]
    if (Test-Alive $s.Proc) { return $s.Proc }
    $exe = Join-Path $Bin $s.Exe
    $s.Proc = Get-Process -Name ($s.Exe -replace '\.exe$', '') -ErrorAction SilentlyContinue |
              Where-Object { try { $_.Path -eq $exe } catch { $false } } | Select-Object -First 1
    $s.Proc
}
function Get-MySqlProc {
    $id = Get-NetTCPConnection -LocalPort $DbPort -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty OwningProcess
    if ($id) { Get-Process -Id $id -ErrorAction SilentlyContinue }
}

function Start-MySql {
    if (Test-Port $DbPort) { return }
    Write-Log 'world' '[manager] starting the database...'
    Start-Process $MySqlD -ArgumentList "--defaults-file=`"$MyIni`"" -WindowStyle Hidden | Out-Null
    if (-not (Wait-Until { Test-Port $DbPort } 60)) { throw "The database did not start. See $Root\db\data\*.err" }
}
function Stop-MySql {
    if (-not (Test-Port $DbPort)) { return }
    if ((Get-ServerProc 'world') -or (Get-ServerProc 'auth')) { throw 'Stop the login and world servers first.' }
    try { Sql 'SHUTDOWN;' | Out-Null } catch { }
    if (Wait-Until { -not (Test-Port $DbPort) } 30) { Write-Log 'world' '[manager] database stopped.'; return }
    if ($p = Get-MySqlProc) { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
    if (-not (Wait-Until { -not (Test-Port $DbPort) } 15)) { throw "The database is still running on port $DbPort." }
}

# The login server won't start while every realm is flagged offline+mismatch (the world sets flag 3 when it stops).
function Reset-RealmFlags {
    if (-not (Test-Port $DbPort)) { return }
    try { Sql 'UPDATE realmlist SET flag = 2 WHERE flag = 3' 'acore_auth' | Out-Null } catch { }
}

function Start-Server([string]$k) {
    if (Get-ServerProc $k) { return }
    Start-MySql
    $s = $Servers[$k]
    if ($k -eq 'auth') { Reset-RealmFlags; Ensure-Account }
    $env:AC_DISABLE_INTERACTIVE = '1'
    $s.LogPos = 0
    $s.Proc = Start-Process (Join-Path $Bin $s.Exe) -ArgumentList '-c', "$Bin\configs\$($s.Conf)" -WorkingDirectory $Bin -WindowStyle Hidden -PassThru
    $s.Desired = $true
    Write-Log $k "[manager] started the $($s.Name.ToLower()) (pid $($s.Proc.Id))"
}

function Stop-Server([string]$k) {
    $s = $Servers[$k]; $s.Desired = $false
    $p = Get-ServerProc $k
    if (-not $p) { return }
    if ($k -eq 'world' -and (Test-Port $s.Port)) {
        # 'server shutdown' saves every player and the living world itself; a separate saveall first can outlast
        # the request and used to end in a forced kill (lost progress since the last autosave)
        Write-Log 'world' '[manager] shutting the world down (it saves first - this can take a minute)...'
        $sent = $false
        try { Invoke-Soap 'server shutdown 1' | Out-Null; $sent = $true } catch { Write-Log 'world' "[manager] $($_.Exception.Message)" }
        if ($sent -and (Wait-Until { -not (Test-Alive $p) } 240)) { Write-Log 'world' '[manager] world server stopped cleanly.'; Reset-RealmFlags; return }
        $r = Show-Msg "The world server did not shut down within 4 minutes.`n`nForce-close it? The living world loses what happened since its last save (every 10 minutes)." 'Warning' 'YesNo'
        if ($r -ne 'Yes') { $s.Desired = $true; return }
    }
    try { $p.Kill() } catch { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
    [void](Wait-Until { -not (Test-Alive $p) } 15)
    if ($k -eq 'world') { Reset-RealmFlags }
    Write-Log $k "[manager] $($s.Name.ToLower()) stopped."
}
function Restart-Server([string]$k) { Stop-Server $k; Start-Sleep -Milliseconds 500; Start-Server $k }
function Start-All { Start-MySql; Start-Server 'auth'; Start-Server 'world' }
function Stop-All  { Stop-Server 'world'; Stop-Server 'auth' }
function Stop-Everything { Stop-All; if (-not (Get-ServerProc 'world') -and -not (Get-ServerProc 'auth')) { Stop-MySql } }

# ---------------------------------------------------------------- the client
$script:Wow = $null
function Start-Play {
    if (-not $Client) { throw "No WoW 3.3.5a client found. Put it in $Root\WOTLK Client." }
    Start-All
    Write-Log 'world' '[manager] waiting for the world to open its doors...'
    if (-not (Wait-Until { Test-Port 8085 } 300)) { throw 'The world did not come up within 5 minutes - see the World console.' }
    Sql "UPDATE realmlist SET icon = 8 WHERE id = 1" acore_auth | Out-Null    # RP-PvP, matches GameType
    Set-Content "$Client\Data\enUS\realmlist.wtf" 'set realmlist 127.0.0.1'
    $cfg = "$Client\WTF\Config.wtf"
    $lines = if (Test-Path $cfg) { @(Get-Content $cfg | Where-Object { $_ -notmatch '^SET (accountName|realmList|realmName) ' }) } else { @() }
    New-Item -ItemType Directory -Force (Split-Path $cfg) | Out-Null
    Set-Content $cfg ($lines + "SET accountName `"$Acct`"")
    $wdb = "$Client\Cache\WDB"                       # townsfolk and generated names reuse ids: start the cache fresh
    if (Test-Path $wdb) { Remove-Item $wdb -Recurse -Force -ErrorAction SilentlyContinue }
    $script:Wow = Start-Process "$Client\Wow.exe" -WorkingDirectory $Client -PassThru
    Write-Log 'world' "[manager] WoW is open. Account: $Acct  Password: $Pass"
}

# ---------------------------------------------------------------- window
$form = New-Ctl Form @{ Text = "$AppName  -  $Root"; Size = (Sz 1000 720); StartPosition = 'CenterScreen'; MinimumSize = (Sz 900 600) }
$mono = New-Object Drawing.Font('Consolas', 9)
$bold = New-Object Drawing.Font('Segoe UI', 10, [Drawing.FontStyle]::Bold)

$grp = New-Ctl GroupBox @{ Text = 'Servers'; Location = (Pt 10 8); Size = (Sz 965 165); Anchor = 'Top,Left,Right' }
$form.Controls.Add($grp)
$Rows = @{}; $y = 22
foreach ($k in 'mysql', 'auth', 'world') {
    $r = @{
        Name   = New-Ctl Label  @{ Text = @{ mysql = 'Database'; auth = 'Login server'; world = 'World server' }[$k]; Font = $bold; Location = (Pt 12 ($y + 4)); Size = (Sz 125 22) }
        Status = New-Ctl Label  @{ Text = '...'; Location = (Pt 140 ($y + 6)); Size = (Sz 190 20) }
        Ram    = New-Ctl Label  @{ Text = ''; Location = (Pt 335 ($y + 6)); Size = (Sz 130 20) }
        Up     = New-Ctl Label  @{ Text = ''; Location = (Pt 470 ($y + 6)); Size = (Sz 110 20) }
        Start  = New-Ctl Button @{ Text = 'Start';   Location = (Pt 600 $y); Size = (Sz 75 28) }
        Stop   = New-Ctl Button @{ Text = 'Stop';    Location = (Pt 680 $y); Size = (Sz 75 28) }
        Re     = New-Ctl Button @{ Text = 'Restart'; Location = (Pt 760 $y); Size = (Sz 75 28) }
    }
    foreach ($c in $r.Values) { $grp.Controls.Add($c) }
    $Rows[$k] = $r; $y += 36
}
$Rows.mysql.Start.Add_Click({ Safe { Start-MySql } })
$Rows.mysql.Stop.Add_Click({ Safe { Stop-MySql } })
$Rows.mysql.Re.Add_Click({ Safe { Stop-MySql; Start-MySql } })
foreach ($k in 'auth', 'world') {            # the row key rides on Tag: closures can't see this script's functions
    foreach ($b in $Rows[$k].Start, $Rows[$k].Stop, $Rows[$k].Re) { $b.Tag = $k }
    $Rows[$k].Start.Add_Click({ param($sender, $e) $k = [string]$sender.Tag; Safe { Start-Server $k } })
    $Rows[$k].Stop.Add_Click({ param($sender, $e) $k = [string]$sender.Tag; Safe { Stop-Server $k } })
    $Rows[$k].Re.Add_Click({ param($sender, $e) $k = [string]$sender.Tag; Safe { Restart-Server $k } })
}
$btnPlay = New-Ctl Button @{ Text = 'Play'; Font = $bold; Location = (Pt 12 $y); Size = (Sz 120 32) }
$btnAll  = New-Ctl Button @{ Text = 'Start all'; Location = (Pt 140 ($y + 2)); Size = (Sz 100 28) }
$btnNone = New-Ctl Button @{ Text = 'Stop all';  Location = (Pt 245 ($y + 2)); Size = (Sz 100 28) }
$chkWow  = New-Ctl CheckBox @{ Text = 'Stop everything when I close WoW'; Checked = $true; Location = (Pt 365 ($y + 6)); Size = (Sz 260 22) }
foreach ($c in $btnPlay, $btnAll, $btnNone, $chkWow) { $grp.Controls.Add($c) }
$btnPlay.Add_Click({ Safe { Start-Play } })
$btnAll.Add_Click({ Safe { Start-All } })
$btnNone.Add_Click({ Safe { Stop-Everything } })

$tabs = New-Ctl TabControl @{ Location = (Pt 10 180); Size = (Sz 965 490); Anchor = 'Top,Bottom,Left,Right' }
$form.Controls.Add($tabs)
function New-Tab([string]$t) { $p = New-Ctl TabPage @{ Text = $t; Padding = New-Object Windows.Forms.Padding(6) }; $tabs.TabPages.Add($p); $p }
$tWorld = New-Tab 'World console'
$logWorld = New-Ctl TextBox @{ Multiline = $true; ReadOnly = $true; ScrollBars = 'Vertical'; Font = $mono; BackColor = [Drawing.Color]::Black; ForeColor = [Drawing.Color]::Gainsboro; Location = (Pt 6 6); Size = (Sz 945 380); Anchor = 'Top,Bottom,Left,Right' }
$cmdBox  = New-Ctl TextBox @{ Font = $mono; Location = (Pt 6 428); Size = (Sz 850 24); Anchor = 'Bottom,Left,Right' }
$cmdSend = New-Ctl Button  @{ Text = 'Send'; Location = (Pt 862 426); Size = (Sz 89 27); Anchor = 'Bottom,Right' }
$tWorld.Controls.AddRange(@($logWorld, $cmdBox, $cmdSend))
$tAuth = New-Tab 'Login log'
$logAuth = New-Ctl TextBox @{ Multiline = $true; ReadOnly = $true; ScrollBars = 'Vertical'; Font = $mono; BackColor = [Drawing.Color]::Black; ForeColor = [Drawing.Color]::Gainsboro; Dock = 'Fill' }
$tAuth.Controls.Add($logAuth)
$LogBoxes = @{ world = $logWorld; auth = $logAuth }
function Write-Log([string]$k, [string]$text) {
    $tb = $LogBoxes[$k]; if (-not $tb) { return }
    if ($tb.TextLength -gt 300000) { $tb.Text = $tb.Text.Substring($tb.TextLength - 150000) }
    $tb.AppendText(($text -replace "`r?`n", "`r`n").TrimEnd("`r", "`n") + "`r`n")
}
function Send-WorldCommand([string]$c) {
    $c = $c.Trim().TrimStart('.')
    if (-not $c) { return }
    Write-Log 'world' "> $c"
    if (-not (Test-Port 8085)) { Write-Log 'world' '[error] The world server is not running.'; return }
    try { $out = Invoke-Soap $c; Write-Log 'world' $(if ($out) { $out.TrimEnd() } else { '(ok)' }) } catch { Write-Log 'world' "[error] $($_.Exception.Message)" }
}
$cmdSend.Add_Click({ $c = $cmdBox.Text; $cmdBox.Clear(); Send-WorldCommand $c })
$cmdBox.Add_KeyDown({ if ($_.KeyCode -eq 'Enter') { $_.SuppressKeyPress = $true; $c = $cmdBox.Text; $cmdBox.Clear(); Send-WorldCommand $c } })
$qx = 6
foreach ($q in @(@('Living world', 'sim status'), @('Who is out', 'sim people'), @('Bounties', 'sim bounties'), @('Save all', 'saveall'), @('Server info', 'server info'))) {
    $b = New-Ctl Button @{ Text = $q[0]; Tag = $q[1]; Location = (Pt $qx 392); Size = (Sz 110 28); Anchor = 'Bottom,Left' }
    $b.Add_Click({ param($sender, $e) Send-WorldCommand ([string]$sender.Tag) })
    $tWorld.Controls.Add($b); $qx += 115
}
$status = New-Ctl Label @{ Text = 'Ready.'; Location = (Pt 12 675); Size = (Sz 960 20); Anchor = 'Bottom,Left,Right'; ForeColor = [Drawing.Color]::DimGray }
$form.Controls.Add($status)

# ---------------------------------------------------------------- the 2-second tick
function Format-Bytes([long]$b) { if ($b -ge 1GB) { '{0:N2} GB' -f ($b / 1GB) } else { '{0:N0} MB' -f ($b / 1MB) } }
function Format-Uptime($p) { try { $t = (Get-Date) - $p.StartTime; if ($t.TotalHours -ge 24) { '{0}d {1}h' -f $t.Days, $t.Hours } else { '{0:00}:{1:00}:{2:00}' -f [int][math]::Floor($t.TotalHours), $t.Minutes, $t.Seconds } } catch { '' } }
function Set-Row([string]$k, $p, [bool]$up, [string]$text) {
    $r = $Rows[$k]
    $r.Status.Text = $text; $r.Status.ForeColor = $(if ($up) { [Drawing.Color]::Green } elseif ($p) { [Drawing.Color]::DarkOrange } else { [Drawing.Color]::Firebrick })
    if ($p) { try { $p.Refresh(); $r.Ram.Text = "RAM $(Format-Bytes $p.WorkingSet64)"; $r.Up.Text = Format-Uptime $p } catch { } } else { $r.Ram.Text = ''; $r.Up.Text = '' }
    $r.Start.Enabled = -not $p; $r.Stop.Enabled = [bool]$p; $r.Re.Enabled = [bool]$p
}
function Read-NewLog([string]$k) {
    $s = $Servers[$k]; $f = Join-Path $Logs $s.Log
    if (-not (Test-Path $f)) { return }
    $fs = [IO.File]::Open($f, 'Open', 'Read', 'ReadWrite, Delete')
    try {
        if ($s.LogPos -lt 0) { $s.LogPos = [Math]::Max(0, $fs.Length - 20000) }
        if ($fs.Length -lt $s.LogPos) { $s.LogPos = 0 }
        if ($fs.Length -eq $s.LogPos) { return }
        $fs.Position = $s.LogPos
        $txt = (New-Object IO.StreamReader($fs)).ReadToEnd(); $s.LogPos = $fs.Length
        if ($txt) { Write-Log $k $txt }
    } finally { $fs.Close() }
}
function Update-Ui {
    $db = Get-MySqlProc
    Set-Row 'mysql' $db ([bool]$db) $(if ($db) { "Running (port $DbPort)" } else { 'Stopped' })
    foreach ($k in 'auth', 'world') {
        $s = $Servers[$k]; $p = Get-ServerProc $k
        $open = $p -and (Test-Port $s.Port)
        Set-Row $k $p $open $(if ($open) { 'Running' } elseif ($p) { 'Starting...' } else { 'Stopped' })
        try { Read-NewLog $k } catch { }
        if (-not $p -and $s.Desired) { $s.Desired = $false; Write-Log $k "[manager] the $($s.Name.ToLower()) exited unexpectedly - see the log above." }
    }
    if ($script:Wow -and $script:Wow.HasExited) {
        $script:Wow = $null
        if ($chkWow.Checked) { $status.Text = 'WoW closed - stopping everything...'; Pump; Stop-Everything; $status.Text = 'WoW closed - everything is stopped.' }
    }
}
$timer = New-Object Windows.Forms.Timer
$timer.Interval = 2000
$timer.Add_Tick({ $timer.Stop(); try { Update-Ui } catch { $status.Text = "Status error: $($_.Exception.Message)" } finally { $timer.Start() } })
$form.Add_Shown({ Update-Ui; $timer.Start() })
$form.Add_FormClosing({
    if ((Get-ServerProc 'world') -or (Get-ServerProc 'auth') -or (Test-Port $DbPort)) {
        $r = Show-Msg "The servers are still running.`n`nYes = stop everything and close`nNo = close and leave them running`nCancel = stay open" 'Question' 'YesNoCancel'
        if ($r -eq 'Cancel') { $_.Cancel = $true; return }
        if ($r -eq 'Yes') { $timer.Stop(); try { Stop-Everything } catch { Show-Msg $_.Exception.Message 'Error' | Out-Null } }
    }
    $timer.Stop()
})
try { [void]$form.ShowDialog() } catch { Show-Msg "Manager error: $($_.Exception.Message)" 'Error' | Out-Null } finally { $mutex.ReleaseMutex() }
