# SoloWoW - build the server (AzerothCore + NPCBots fork in core\) and install it into server\.
# Recipe from AzerothManager (Build-Core / Copy-RuntimeDlls): VS 2022, Boost 1.81, OpenSSL 3.6.5,
# RAM-capped compile jobs. Release config, no extractor tools (map data lives in server\data).
param([string]$Config = 'Release')
$ErrorActionPreference = 'Continue'
$Root   = 'C:\SoloWoW'
$Src    = "$Root\core"
$Build  = "$Root\build"
$Server = "$Root\server"
$CMake  = 'C:\Program Files\CMake\bin\cmake.exe'
$MySql  = 'C:\Program Files\MySQL\MySQL Server 8.4'
$Ssl    = 'C:\Program Files\OpenSSL-Win64'
$Boost  = 'C:/local/boost_1_81_0'
function Fwd($p) { $p -replace '\\', '/' }

$t0 = Get-Date
# Be a good neighbour: idle priority (cmake -> MSBuild -> cl.exe inherit it), so the PC stays responsive.
try { (Get-Process -Id $PID).PriorityClass = 'Idle' } catch { }
$env:BOOST_ROOT = $Boost
& $CMake -S $Src -B $Build -G 'Visual Studio 17 2022' -A x64 `
    -DTOOLS_BUILD=none -DSCRIPTS=static -DMODULES=static `
    "-DBOOST_ROOT=$Boost" "-DOPENSSL_ROOT_DIR=$(Fwd $Ssl)" `
    "-DMYSQL_INCLUDE_DIR=$(Fwd $MySql)/include" "-DMYSQL_LIBRARY=$(Fwd $MySql)/lib/libmysql.lib"
if ($LASTEXITCODE -ne 0) { Write-Host 'BUILD-RESULT: CONFIGURE FAILED'; exit 1 }

# Each cl.exe loads a ~1 GB PCH; cap parallel jobs by RAM, halve and retry on failure.
$ramGb = [math]::Floor((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB)
$jobs  = [math]::Max(1, [math]::Min([math]::Floor([Environment]::ProcessorCount / 2), [math]::Floor($ramGb / 2.5)))   # at most half the threads
$ok = $false
foreach ($try in 1..4) {
    Write-Host "== Compiling $Config with $jobs jobs, attempt $try"
    Get-ChildItem $Build -Recurse -Filter '*.vcxproj' | ForEach-Object {
        $t = [IO.File]::ReadAllText($_.FullName)
        $n = [regex]::Replace($t, '(?<=[\s>;"])/MP\d*(?=[\s<;"])', "/MP$jobs")
        $n = [regex]::Replace($n, '<ProcessorNumber>\d*</ProcessorNumber>', "<ProcessorNumber>$jobs</ProcessorNumber>")
        if ($n -ne $t) { [IO.File]::WriteAllText($_.FullName, $n) }
    }
    & $CMake --build $Build --config $Config -- /m:1 "/p:CL_MPCount=$jobs" /nologo /v:minimal
    if ($LASTEXITCODE -eq 0) { $ok = $true; break }
    $jobs = [math]::Max(1, [math]::Floor($jobs / 2))
}
if (-not $ok) { Write-Host 'BUILD-RESULT: COMPILE FAILED'; exit 1 }

# Install: binaries + runtime DLLs into server\, config templates into server\configs\.
# Existing server\configs\*.conf are never overwritten (configure.ps1 owns them).
$bin = "$Build\bin\$Config"
New-Item -ItemType Directory -Force $Server, "$Server\configs" | Out-Null
Copy-Item "$bin\*.exe", "$bin\*.dll" $Server -Force -ErrorAction SilentlyContinue
Copy-Item "$bin\configs\*.dist" "$Server\configs" -Force
Copy-Item "$MySql\lib\libmysql.dll", "$Ssl\bin\libssl-3-x64.dll", "$Ssl\bin\libcrypto-3-x64.dll" $Server -Force
$legacy = Get-ChildItem $Ssl -Recurse -Filter legacy.dll -ErrorAction SilentlyContinue | Select-Object -First 1
if ($legacy) { Copy-Item $legacy.FullName $Server -Force } else { Write-Host 'WARNING: legacy.dll not found' }
Write-Host ("BUILD-RESULT: OK in {0:n1} min -> {1}" -f ((Get-Date) - $t0).TotalMinutes, $Server)
