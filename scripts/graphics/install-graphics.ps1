# SoloWoW: the graphics stack for our 3.3.5a client (Nov 17) - Project Reforged's HD models, LuxVelum (fog, light and
# water), DXVK, and the Wow.exe patches they need. Everything is fetched from its own project at install time, always
# the latest release; nothing of theirs is kept in our repo. Run again to update (only what changed is downloaded).
#   powershell -ExecutionPolicy Bypass -File scripts\graphics\install-graphics.ps1 [-Client <folder>] [-Extras] [-Interface] [-SD]
#     -Extras     also Reforged's optional packs: maps and loading screens (M), darker nights (N), spell effects (P),
#                 upright orcs (U), sound and music (S)
#     -Interface  also Reforged's interface pack (I) - off by default: our character creation screen (Worgen/Goblin) is a
#                 glue interface of its own
#     -SD         the standard-definition character models instead of HD
#     -NoModels / -NoLux / -NoDxvk / -NoExe   leave that part alone (-NoLux also skips the light shaders and Lexara)
# Afterwards scripts\graphics\merge-dbc.py fits our own client patches (patch-W) to the models (see docs\GRAPHICS.md).
param(
    [string]$Client = 'C:\SoloWoW\WOTLK Client',
    [switch]$Extras, [switch]$Interface, [switch]$SD,
    [switch]$NoModels, [switch]$NoLux, [switch]$NoDxvk, [switch]$NoExe
)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$ua = @{ 'User-Agent' = 'SoloWoW-installer' }
$work = Join-Path $env:TEMP 'SoloWoW-graphics'
New-Item -ItemType Directory -Force $work | Out-Null
$ORIGINAL = 'AA63A5750D60EF16746C686B3D5E26876D98953EAB08B1C026CD0FAF78E88CB8'   # Wow.exe 3.3.5a (12340), unmodified

function Say($m) { Write-Host "[graphics] $m" }
function Latest($repo) { Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest" -Headers $ua -TimeoutSec 60 }
function Fetch($url, $out) {
    # resumable, follows redirects, retries; curl ships with Windows 10 and 11
    & curl.exe -L --fail --retry 5 --retry-delay 5 -C - -s -S -o $out $url
    if ($LASTEXITCODE -ne 0 -and $LASTEXITCODE -ne 33) { throw "download failed ($LASTEXITCODE): $url" }
}
function RemoteSize($url) {
    try { [int64](Invoke-WebRequest -Uri $url -Method Head -UseBasicParsing -TimeoutSec 60).Headers['Content-Length'] } catch { -1 }
}

if (-not (Test-Path (Join-Path $Client 'Wow.exe'))) { throw "No Wow.exe in $Client" }
if (Get-Process Wow -ErrorAction SilentlyContinue) { throw 'Close WoW first (the exe and the patch files are in use).' }

# ---- 1. Wow.exe: St0ny's AIO EXE Patcher with Project Reforged's preset (4GB, custom GlueXML for our creation screen,
#         the LuxVelum loader, fixes). It starts from the original exe: ours is restored from Backup\ first.
if (-not $NoExe) {
    $exe = Join-Path $Client 'Wow.exe'
    $backup = Join-Path $Client 'Backup\Wow.exe'
    if ((Get-FileHash $exe -Algorithm SHA256).Hash -ne $ORIGINAL) {
        if (-not (Test-Path $backup) -or (Get-FileHash $backup -Algorithm SHA256).Hash -ne $ORIGINAL) {
            # already patched by St0ny's patcher? then it can re-patch its own exe; anything else needs the original
            $state = Join-Path $work 'St0nys-AIO-WoW-EXE-Patcher\patcher_state.ini'
            if (-not (Test-Path $state)) { throw "Wow.exe is modified and there is no original in $backup - put the unmodified 3.3.5a Wow.exe there." }
        } else {
            Copy-Item $exe (Join-Path $Client 'Backup\Wow.exe.before-graphics') -Force
            Copy-Item $backup $exe -Force
            Say 'Wow.exe restored to the original (the previous one is in Backup\Wow.exe.before-graphics)'
        }
    }
    $rel = Latest 'Raz0r1337/St0nys-AIO-WoW-EXE-Patcher'
    $asset = $rel.assets | Where-Object { $_.name -like '*.zip' } | Select-Object -First 1
    $zip = Join-Path $work $asset.name
    if (-not (Test-Path $zip)) { Fetch $asset.browser_download_url $zip }
    Expand-Archive $zip -DestinationPath $work -Force
    $bat = Get-ChildItem $work -Recurse -Filter 'patcher-win.bat' | Select-Object -First 1
    Say "EXE patcher $($rel.tag_name): preset Reforged"
    & cmd /c "`"$($bat.FullName)`" -Language en -Select reforged -Unattended -Path `"$exe`""
    if ($LASTEXITCODE -ne 0) { throw "the EXE patcher failed ($LASTEXITCODE)" }
}

# ---- 2. DXVK (the WoW build Project Reforged recommends): d3d9.dll next to Wow.exe
if (-not $NoDxvk) {
    $rel = Latest 'Raz0r1337/dxvk'
    $asset = $rel.assets | Where-Object { $_.name -match 'x86' -and $_.name -like '*.zip' } | Select-Object -First 1
    $zip = Join-Path $work $asset.name
    Fetch $asset.browser_download_url $zip
    $dir = Join-Path $work 'dxvk'; Remove-Item $dir -Recurse -Force -ErrorAction SilentlyContinue
    Expand-Archive $zip -DestinationPath $dir -Force
    $dll = Get-ChildItem $dir -Recurse -Filter 'd3d9.dll' | Select-Object -First 1
    $dest = Join-Path $Client 'd3d9.dll'
    if ((Test-Path $dest) -and -not (Test-Path "$dest.before-graphics")) { Copy-Item $dest "$dest.before-graphics" }
    Copy-Item $dll.FullName $dest -Force
    Get-ChildItem $dir -Recurse -Filter '*.conf' | ForEach-Object { if (-not (Test-Path (Join-Path $Client $_.Name))) { Copy-Item $_.FullName $Client } }
    Say "DXVK $($rel.tag_name)"
}

# ---- 3. LuxVelum: LuxVelum.dll + its data next to Wow.exe (the patched exe loads it - no version.dll), Reforged's
#         settings on a first install (an existing LuxVelum.ini is kept)
if (-not $NoLux) {
    $rel = Latest 'Raz0r1337/LuxVelum'
    $asset = $rel.assets | Where-Object { $_.name -eq 'LuxVelum-Win32.zip' } | Select-Object -First 1
    $zip = Join-Path $work $asset.name
    Remove-Item $zip -ErrorAction SilentlyContinue
    Fetch $asset.browser_download_url $zip
    $dir = Join-Path $work 'luxvelum'; Remove-Item $dir -Recurse -Force -ErrorAction SilentlyContinue
    Expand-Archive $zip -DestinationPath $dir -Force
    $hadIni = Test-Path (Join-Path $Client 'LuxVelum.ini')
    foreach ($f in Get-ChildItem $dir -Recurse -File) {
        if ($f.Name -eq 'LuxVelum.ini' -and $hadIni) { continue }
        if ($f.Name -eq 'version.dll') { continue }                      # not needed with the patched exe
        Copy-Item $f.FullName (Join-Path $Client $f.Name) -Force
    }
    foreach ($old in 'CoAVolFog.dll', 'Luxveil.dll', 'version.dll') {    # an older fog DLL's loader stops the client starting
        $p = Join-Path $Client $old
        if (Test-Path $p) { Move-Item $p "$p.disabled" -Force; Say "set aside $old" }
    }
    if (-not $hadIni) {
        try { Fetch 'https://pub-0f05631d243e4046993fc02ca7be9542.r2.dev/Wrath%20of%20the%20Lich%20King/mods/presets/LuxVelum.ini' (Join-Path $Client 'LuxVelum.ini') }
        catch { Say 'Reforged LuxVelum preset unavailable - using the defaults' }
    }
    Say "LuxVelum $($rel.tag_name)"
}

# ---- 3b. the shaders for 8 point lights (the exe preset turns that on) into Data\patch-5.MPQ\, and Lexara's HD fonts
#          (the patched exe loads Lexara.dll itself)
if (-not $NoLux) {
    $rel = Latest 'Raz0r1337/WoW-lights-patch-shader'
    $asset = $rel.assets | Where-Object { $_.name -eq 'patch-5.MPQ.zip' } | Select-Object -First 1
    $zip = Join-Path $work $asset.name
    Remove-Item $zip -ErrorAction SilentlyContinue
    Fetch $asset.browser_download_url $zip
    $data = Join-Path $Client 'Data'
    Expand-Archive $zip -DestinationPath $data -Force
    if (-not (Test-Path (Join-Path $data 'patch-5.MPQ\Shaders'))) { Say 'warning: the light shaders did not unpack where expected (Data\patch-5.MPQ\Shaders)' }
    Say "light shaders $($rel.tag_name)"
    $rel = Latest 'Stormhand-dev/Lexara---HD-Font-Renderer-for-WoW-3.3.5'
    foreach ($name in 'lexara.dll', 'skia.dll') {
        $asset = $rel.assets | Where-Object { $_.name -eq $name } | Select-Object -First 1
        if (-not $asset) { continue }
        $dest = Join-Path $Client ($(if ($name -eq 'lexara.dll') { 'Lexara.dll' } else { $name }))
        Remove-Item "$dest.new" -ErrorAction SilentlyContinue
        Fetch $asset.browser_download_url "$dest.new"
        Move-Item "$dest.new" $dest -Force
    }
    Say "Lexara $($rel.tag_name)"
}

# ---- 4. Project Reforged's HD packs: the links on their downloads page (always the current ones), into Data\
if (-not $NoModels) {
    $page = (Invoke-WebRequest 'https://projectreforged.github.io/wotlk/downloads/' -UseBasicParsing -TimeoutSec 60).Content
    $links = [regex]::Matches($page, 'href="([^"]+?\.mpq)"', 'IgnoreCase') | ForEach-Object { $_.Groups[1].Value } | Select-Object -Unique
    $want = @('A', 'B', 'C', 'D', 'E', 'G')                                   # characters, buildings, creatures, doodads, environment, gear
    if ($Interface) { $want += 'I' }
    if ($Extras) { $want += 'M', 'N', 'P', 'U', 'S' }
    $data = Join-Path $Client 'Data'
    foreach ($letter in $want) {
        $cands = $links | Where-Object { $_ -match "/patch-$letter\.mpq$" }
        $extra = $cands | Where-Object { $_ -match '/extras/' }
        $main = $cands | Where-Object { $_ -notmatch '/extras/' }
        $url = if ($letter -eq 'A' -and $SD) { $extra } elseif ($letter -eq 'S' -and -not $Extras) { $extra } else { $main }
        $url = @($url)[0]
        if (-not $url) { Say "patch-$letter is not on the downloads page any more - skipped"; continue }
        $dest = Join-Path $data "patch-$letter.MPQ"
        $size = RemoteSize $url
        if ((Test-Path $dest) -and $size -gt 0 -and (Get-Item $dest).Length -eq $size) { Say "patch-$letter is up to date"; continue }
        $part = "$dest.part"
        if ((Test-Path $dest) -and -not (Test-Path $part)) { Remove-Item $dest }   # a changed pack: fetched anew
        Say ("patch-{0}: downloading {1:N0} MB" -f $letter, ($size / 1MB))
        Fetch $url $part
        if ($size -gt 0 -and (Get-Item $part).Length -ne $size) { throw "patch-$letter came down short" }
        Move-Item $part $dest -Force
    }
    Say 'Reforged packs in place'
}
Say 'done - now run scripts\graphics\merge-dbc.py so our own patches fit the models'
