# Azeroth Alive installer. Steps follow docs/INSTALLER.md; the ones marked "not yet" only say what they will do.
#   install.ps1 [-Client <folder>] [-Target <folder>] [-DryRun]
#   install.ps1 -Uninstall -Client <folder>      (puts the client back from the backups)
param([string]$Client = '', [string]$Target = 'C:\AzerothAlive', [switch]$DryRun, [switch]$Uninstall)
$ErrorActionPreference = 'Stop'
function Say([string]$t) { Write-Host "  $t" }
function Step([string]$t) { Write-Host ""; Write-Host "== $t" -ForegroundColor Cyan }

# ---- 1. where
Step 'Where'
if (-not $Client) {
    $Client = @('C:\SoloWoW\WOTLK Client', 'C:\WOTLK Client', 'C:\Games\WOTLK Client', 'D:\WOTLK Client') | Where-Object { Test-Path (Join-Path $_ 'Wow.exe') } | Select-Object -First 1
    if (-not $Client) { $Client = Read-Host '  Your WoW 3.3.5a client folder (the one with Wow.exe)' }
}
$wow = Join-Path $Client 'Wow.exe'
if (-not (Test-Path $wow)) { Say "No Wow.exe in $Client."; exit 1 }
Say "Client:  $Client"
Say "Install: $Target"

# ---- 2. check the client
Step 'Checking the client'
$ver = (Get-Item $wow).VersionInfo
$build = "$($ver.FileVersion)" -replace '[^0-9]+', '.'
if ($build -notmatch '12340') { Say "This Wow.exe is version $($ver.FileVersion) - Azeroth Alive needs 3.3.5a (build 12340)."; exit 1 }
Say "Wow.exe is 3.3.5a (build 12340)."
$locale = Get-ChildItem (Join-Path $Client 'Data') -Directory -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '^[a-z]{2}[A-Z]{2}$' } | Select-Object -First 1
if (-not $locale) { Say 'No locale folder (like Data\enUS) found - is this a complete client?'; exit 1 }
Say "Locale: $($locale.Name)"

# client patch helpers (step 9) -------------------------------------------------------------------------
function Get-Laa([string]$exe) {
    $b = [IO.File]::ReadAllBytes($exe); $pe = [BitConverter]::ToInt32($b, 0x3C)
    return ([BitConverter]::ToUInt16($b, $pe + 4 + 18) -band 0x20) -ne 0
}
function Set-Laa([string]$exe) {
    $b = [IO.File]::ReadAllBytes($exe); $pe = [BitConverter]::ToInt32($b, 0x3C); $at = $pe + 4 + 18
    if ([BitConverter]::ToUInt32($b, $pe) -ne 0x4550) { throw 'Wow.exe does not look like a Windows program (no PE header).' }
    $ch = [BitConverter]::ToUInt16($b, $at) -bor 0x20
    $b[$at] = [byte]($ch -band 0xFF); $b[$at + 1] = [byte]($ch -shr 8)
    [IO.File]::WriteAllBytes($exe, $b)
}
function Backup([string]$file) { $bak = "$file.aa-backup"; if ((Test-Path $file) -and -not (Test-Path $bak)) { Copy-Item $file $bak } }
function Restore([string]$file) { $bak = "$file.aa-backup"; if (Test-Path $bak) { Move-Item $bak $file -Force; Say "Restored $(Split-Path $file -Leaf)" } }

$realmlist = Join-Path $locale.FullName 'realmlist.wtf'
$config = Join-Path $Client 'WTF\Config.wtf'

if ($Uninstall) {
    Step 'Putting the client back'
    Restore $wow; Restore $realmlist; Restore $config
    Say 'Done. (The install folder itself is not removed yet - delete it by hand if you want it gone.)'
    exit 0
}

# ---- 3-8: not yet
Step 'Runtime (Visual C++ redistributable, private MySQL 8.4)';  Say 'not yet - see docs/INSTALLER.md step 3'
Step 'Server (release download)';                                 Say 'not yet - step 4'
Step 'Game data (extracted from your client)';                    Say 'not yet - step 5'
Step 'Databases';                                                 Say 'not yet - step 6'
Step 'Configuration';                                             Say 'not yet - step 7'
Step 'Launcher';                                                  Say 'not yet - step 8'

# ---- 9. patch the client (backups first)
Step 'Patching the client'
$rl = if (Test-Path $realmlist) { (Get-Content $realmlist -Raw).Trim() } else { '' }
if ($rl -match '^set realmlist 127\.0\.0\.1$') { Say 'realmlist already points at 127.0.0.1' }
elseif ($DryRun) { Say "would set realmlist to 127.0.0.1 (now: '$rl')" }
else { Backup $realmlist; Set-Content -Path $realmlist -Value 'set realmlist 127.0.0.1' -Encoding ASCII; Say 'realmlist -> 127.0.0.1' }
if (Test-Path $config) {
    $cfg = Get-Content $config
    $line = $cfg | Where-Object { $_ -match '^SET realmlist ' }
    if ($line -and $line -notmatch '"127\.0\.0\.1"') {
        if ($DryRun) { Say "would change Config.wtf's realmlist ($line)" }
        else { Backup $config; ($cfg | ForEach-Object { if ($_ -match '^SET realmlist ') { 'SET realmlist "127.0.0.1"' } else { $_ } }) | Set-Content $config -Encoding ASCII; Say 'Config.wtf realmlist -> 127.0.0.1' }
    }
}
if (Get-Laa $wow) { Say 'Wow.exe can already use 4 GB (Large Address Aware)' }
elseif ($DryRun) { Say 'would let Wow.exe use 4 GB of memory (Large Address Aware), keeping Wow.exe.aa-backup' }
else { Backup $wow; Set-Laa $wow; Say 'Wow.exe can now use 4 GB of memory (backup: Wow.exe.aa-backup)' }

Step 'Done'
if ($DryRun) { Say 'Dry run - nothing was changed.' } else { Say 'Client patched. The server steps are still to come (see docs/INSTALLER.md).' }