# SoloWoW - write authserver.conf / worldserver.conf from the .dist files.
# -Bots      how many wandering bots. With the bubble (default) they follow the player's zone;
#            the launcher's one bot number. Max 368 (bot templates in the DB).
# -NoBubble  old behaviour: -Bots spread over -Continent for the whole session.
# -Continent (NoBubble only) EK, Kalimdor, Outland, Northrend or All. Maps and level spread are set
#            together: a spread asking for levels the maps lack trips an assert (botdatamgr.cpp:411).
param([int]$Bots = 150, [switch]$NoBubble, [switch]$NoEra, [switch]$RandomBots,
      [ValidateSet('EK','Kalimdor','Outland','Northrend','All')][string]$Continent = 'All',
      [string]$Config = 'Release', [switch]$Fresh)
$Bin  = "C:\SoloWoW\server"
$Conf = "$Bin\configs"
$Db   = '127.0.0.1;3310;acore;acore'

# Brackets: 0-9,10-19,...,70-79,80. Must total 100 and only use brackets the maps have.
# EK verified; Kalimdor/Outland/Northrend spreads are untested guesses from zone level ranges.
$Spread = @{
    EK        = @{ Maps = '0';           Levels = '25,20,20,15,10,10,0,0,0' }
    Kalimdor  = @{ Maps = '1';           Levels = '25,20,20,15,10,10,0,0,0' }
    Outland   = @{ Maps = '530';         Levels = '0,0,0,0,0,40,60,0,0' }
    Northrend = @{ Maps = '571';         Levels = '0,0,0,0,0,0,40,60,0' }
    All       = @{ Maps = '0,1,530,571'; Levels = '25,15,15,10,5,10,20,0,0' }
}

# Replace "key = value"; append it if the key isn't in the file (our own SoloWoW keys).
function Set-Conf([string]$file, [string]$key, [string]$value) {
    $txt = [IO.File]::ReadAllText($file)
    $pat = '(?m)^([ \t]*)' + [regex]::Escape($key) + '[ \t]*=[^\r\n]*'
    if ($txt -match $pat) { $txt = [regex]::Replace($txt, $pat, { param($m) $m.Groups[1].Value + "$key = $value" }) }
    else { $txt = $txt.TrimEnd() + "`r`n$key = $value`r`n" }
    [IO.File]::WriteAllText($file, $txt)
}

$auth = "$Conf\authserver.conf"; $world = "$Conf\worldserver.conf"
if ($Fresh -or -not (Test-Path $auth))  { Copy-Item "$Conf\authserver.conf.dist"  $auth -Force }
if ($Fresh -or -not (Test-Path $world)) { Copy-Item "$Conf\worldserver.conf.dist" $world -Force }

$common = [ordered]@{
    'LoginDatabaseInfo' = "`"$Db;acore_auth`""
    'LogsDir'           = '"C:/SoloWoW/logs"'
    'BindIP'            = '"127.0.0.1"'
    'Updates.AutoSetup' = '1'
}
foreach ($k in $common.Keys) { Set-Conf $auth $k $common[$k]; Set-Conf $world $k $common[$k] }

$w = [ordered]@{
    'WorldDatabaseInfo'      = "`"$Db;acore_world`""
    'CharacterDatabaseInfo'  = "`"$Db;acore_characters`""
    'DataDir'                = '"C:/SoloWoW/server/data"'
    'Console.Enable'         = '0'
    'SOAP.Enabled'           = '1'
    'SOAP.IP'                = '"127.0.0.1"'
    'SOAP.Port'              = '7879'
    'Expansion'              = '2'
    'PreloadAllNonInstancedMapGrids' = '0'
    # Lean offline world: these three kept grids loaded / CPU busy with nobody online.
    'IsContinentTransport.Enabled' = '0'    # boats/zeppelins - revisit: players will want them
    'Wintergrasp.Enable'     = '0'
    'Warden.Enabled'         = '0'
    # NB: this AzerothCore never unloads open-world grids during a session (no GridUnload option).
    'NpcBot.Enable'          = '1'
    # RP-PvP standard (8). Never FFA (16): the bot generator turns all wanderers hostile-to-all on FFA realms.
    # play.ps1 sets realmlist.icon to match.
    'GameType'               = '8'
}
if ($NoBubble) {
    $w['NpcBot.Bubble.Enable'] = '0'
    $w['NpcBot.WanderingBots.Continents.Count']  = "$Bots"
    $w['NpcBot.WanderingBots.Continents.Maps']   = $Spread[$Continent].Maps
    $w['NpcBot.WanderingBots.Continents.Levels'] = $Spread[$Continent].Levels
} else {
    # Bubble: the stock generator spawns nothing; botbubble.cpp keeps -Bots around the player's zone.
    $w['NpcBot.WanderingBots.Continents.Count']  = '0'
    $w['NpcBot.WanderingBots.Continents.Maps']   = '0,1,530,571'
    $w['NpcBot.WanderingBots.Continents.Levels'] = $Spread['All'].Levels
    $w['NpcBot.Bubble.Enable']    = '1'
    $w['NpcBot.Bubble.Count']     = "$Bots"
    $w['NpcBot.Bubble.Neighbors'] = '1'
    $w['NpcBot.Bubble.HomeZoneShare']      = '75'    # % of new bots spawned in the player's own zone
    $w['NpcBot.Bubble.Interval']           = '2000'
    $w['NpcBot.Bubble.MaxRecyclePerCheck'] = '5'
    $w['NpcBot.Bubble.MaxSpawnPerCheck']   = '4'
    $w['NpcBot.Bubble.SafeDistance']       = '150'
    $w['NpcBot.Bubble.SpawnAwayDistance']  = '120'
    # sim = the bubble embodies the SoloSim people (named, levelled, going about their lives);
    # random = anonymous random wanderers (the old bubble). -RandomBots picks random.
    $w['NpcBot.Bubble.Source'] = if ($RandomBots) { 'random' } else { 'sim' }
    $w['NpcBot.Bubble.Townsfolk'] = '100'  # townsfolk bodies around the player (Nov 17: 30 -> 100; capitals x1.5) (plain creatures, sim source only)
}
# SoloSim living world: runs alongside the server in real time. While the server is off it catches up
# (up to CatchUpHours) on the next start; at TimeScale 1 the sim's hour of day follows the wall clock.
$w['SoloSim.Live.Enable']       = '1'
$w['SoloSim.Live.TimeScale']    = '1'
$w['SoloSim.Live.CatchUpHours'] = '72'
# The fork's Warcraft III hero classes wander as hostile-to-everyone elites (a Crypt Lord killed
# bots and fought Stormwind guards next to a level 1). Off for wanderers; normal WoW classes only.
foreach ($c in 'Blademaster','ObsidianDestroyer','Archmage','Dreadlord','SpellBreaker','DarkRanger','Necromancer','SeaWitch','CryptLord') {
    $w["NpcBot.WanderingBots.Classes.$c.Enable"] = '0'
}
# "Era" preset (default): slower than vanilla - the sim's jobs/ambitions carry progression, not quest XP.
# -NoEra restores stock rates. The DB half (quest map areas, riding levels) is scripts\era-db.ps1.
$xp = if ($NoEra) { '1' } else { '0.5' }
$w['Rate.XP.Kill'] = $xp; $w['Rate.XP.Quest'] = $xp; $w['Rate.XP.Explore'] = $xp
# XP bands (Nov 17, the user's OK): vanilla's pace for 1-60 by band, replacing the flat rate above for players and the
# sim's adventurers (SoloWoW.XPBands holds the numbers; see solosim_xp.cpp)
$w['SoloWoW.XPBands.Enable'] = if ($NoEra) { '0' } else { '1' }
$w['DungeonFinder.OptionsMask'] = if ($NoEra) { '7' } else { '0' }   # 0 = no queue-and-teleport; walk there with hired bots
# Hiring is free: people come along for their own reasons (regard - see BotBubble::InviteByName / World::HireRefusal), not for gold.
$w['NpcBot.Cost.Hire'] = '0'
foreach ($k in $w.Keys) { Set-Conf $world $k $w[$k] }
if ($NoBubble) { Write-Host "Configured: $Bots wandering bots on $Continent (no bubble)" }
else           { Write-Host "Configured: bubble with $Bots wandering bots around the player" }
