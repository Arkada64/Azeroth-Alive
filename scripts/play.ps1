# SoloWoW - one-click play (stopgap until the real launcher, milestone 4).
# Starts MySQL, authserver and worldserver hidden, waits for the world, opens WoW,
# and shuts everything down cleanly when WoW closes.
param([switch]$NoClient, [switch]$WoWee)   # -NoClient: server only (testing). -WoWee: open WoWee instead of Wow.exe
. C:\SoloWoW\scripts\common.ps1
$env:AC_DISABLE_INTERACTIVE = '1'
# The player's own 3.3.5a client: first folder here that has Wow.exe (add more places if it moves again).
$Client = @('C:\SoloWoW\WOTLK Client', 'C:\SoloWoW\client', 'C:\WOTLK Client', 'C:\Games\WOTLK Client', 'D:\WOTLK Client') |
    Where-Object { Test-Path (Join-Path $_ 'Wow.exe') } | Select-Object -First 1
if (-not $Client -and -not $NoClient) {
    Write-Host '  Could not find your WoW 3.3.5a client (Wow.exe). Put it in C:\SoloWoW\client, or add its folder'
    Write-Host '  to the list at the top of C:\SoloWoW\scripts\play.ps1.'; Read-Host '  Press Enter to close'; exit 1
}
$MysqlD = 'C:\Program Files\MySQL\MySQL Server 8.4\bin\mysqld.exe'
$Host.UI.RawUI.WindowTitle = 'SoloWoW'
function Say($s) { Write-Host "  $s" }
Write-Host "`n  SoloWoW`n  -------"

# Another server (AzerothManager) on the same ports would steal the login.
$other = Get-Process worldserver, authserver -ErrorAction SilentlyContinue | Where-Object { $_.Path -notlike 'C:\SoloWoW\*' }
if ($other) { Say 'AzerothManager''s server is running. Stop it first, then run Play again.'; Read-Host '  Press Enter to close'; exit 1 }

if (-not (Test-Port 3310)) {
    Say 'Starting database...'
    Start-Process $MysqlD -ArgumentList '--defaults-file=C:\SoloWoW\db\my.ini' -WindowStyle Hidden
    $t = 0; while (-not (Test-Port 3310) -and $t -lt 60) { Start-Sleep 1; $t++ }
    if (-not (Test-Port 3310)) { Say 'The database did not start. See C:\SoloWoW\db\data\*.err'; Read-Host '  Press Enter to close'; exit 1 }
}
if (-not (Test-Port 3724)) {
    Start-Process "$Bin\authserver.exe" -ArgumentList '-c', "$Bin\configs\authserver.conf" -WorkingDirectory $Bin -WindowStyle Hidden `
        -RedirectStandardOutput 'C:\SoloWoW\logs\auth.out' -RedirectStandardError 'C:\SoloWoW\logs\auth.err'
}
if (-not (Test-Port 8085)) {
    Say 'Waking the world (about 25 seconds)...'
    $w = Start-Process "$Bin\worldserver.exe" -ArgumentList '-c', "$Bin\configs\worldserver.conf" -WorkingDirectory $Bin -WindowStyle Hidden -PassThru `
        -RedirectStandardOutput 'C:\SoloWoW\logs\world.out' -RedirectStandardError 'C:\SoloWoW\logs\world.err'
    $t0 = Get-Date
    while (-not (Test-Port 8085)) {
        if ($w.HasExited) { Say 'The world server stopped while starting. Log: C:\SoloWoW\logs\world.out'; & C:\SoloWoW\scripts\stop.ps1 -StopMysql | Out-Null; Read-Host '  Press Enter to close'; exit 1 }
        Start-Sleep -Milliseconds 500
    }
    Say ("World is up after {0:n0} s." -f ((Get-Date) - $t0).TotalSeconds)
}
Ensure-Account
Sql "UPDATE realmlist SET icon = 8 WHERE id = 1" acore_auth | Out-Null   # RP-PvP, matches GameType in worldserver.conf

# Point the client at us and prefill the account name.
if ($Client) {
    Set-Content "$Client\Data\enUS\realmlist.wtf" 'set realmlist 127.0.0.1'
    $cfg = "$Client\WTF\Config.wtf"
    $lines = if (Test-Path $cfg) { @(Get-Content $cfg | Where-Object { $_ -notmatch '^SET (accountName|realmList|realmName) ' }) } else { @() }
    $lines += "SET accountName `"$Acct`""
    New-Item -ItemType Directory -Force (Split-Path $cfg) | Out-Null
    Set-Content $cfg $lines
}

if ($NoClient) { Say 'Server is up (NoClient).'; exit 0 }
if ($WoWee) {
    # WoWee lives in C:\SoloWoW\client; the server is chosen on its login screen (127.0.0.1)
    $wd = 'C:\SoloWoW\client'
    # Known WoWee bug (issues #154/#157): MSAA loses the GPU at character select on NVIDIA (RTX 3060 Ti here).
    # Workaround: antialiasing=0 in %APPDATA%\wowee\settings.cfg, and no MSAA in the character preview.
    $sc = "$env:APPDATA\wowee\settings.cfg"
    if (-not (Test-Path $sc) -or -not (Select-String $sc -Pattern '^antialiasing=0' -Quiet)) {
        $lines = if (Test-Path $sc) { @(Get-Content $sc | Where-Object { $_ -notmatch '^antialiasing\s*=' }) } else { @() }
        New-Item -ItemType Directory -Force (Split-Path $sc) | Out-Null
        Set-Content $sc ($lines + 'antialiasing=0')
    }
    $env:WOWEE_PREVIEW_MSAA = '1'
    Say "Opening WoWee. Server: 127.0.0.1   Account: $Acct   Password: $Pass"
    $wow = Start-Process "$wd\wowee.exe" -WorkingDirectory $wd -PassThru
} else {
    Say "Opening WoW. Account: $Acct   Password: $Pass"
    # the client's item/name cache: generated items and townsfolk re-use ids, so old entries would show stale names
    Remove-Item (Join-Path $Client 'Cache\WDB') -Recurse -Force -ErrorAction SilentlyContinue
    $wow = Start-Process "$Client\Wow.exe" -WorkingDirectory $Client -PassThru
}
Say 'Keep this window open - it saves and shuts the server down when you close WoW.'
Add-Type -Name W -Namespace U -MemberDefinition '[DllImport("kernel32.dll")] public static extern System.IntPtr GetConsoleWindow(); [DllImport("user32.dll")] public static extern bool ShowWindow(System.IntPtr h, int c);'
[U.W]::ShowWindow([U.W]::GetConsoleWindow(), 6) | Out-Null   # minimize
$wow.WaitForExit()
[U.W]::ShowWindow([U.W]::GetConsoleWindow(), 9) | Out-Null   # restore
Say 'WoW closed - saving and stopping the server...'
& C:\SoloWoW\scripts\stop.ps1 -StopMysql
Say 'All stopped. Bye!'
Start-Sleep 3
