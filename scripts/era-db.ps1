# SoloWoW "Era" preset - world DB part. Reversible: originals are kept in solo_era_* backup tables.
#   era-db.ps1        apply   (quest map/minimap objective areas off, vanilla/TBC riding requirements, and the difficulty
#                     revert: vanilla/TBC creatures and quests, the XP curve, old quest XP; no dual spec, heirlooms, barbers)
#   era-db.ps1 -Off   restore the stock 3.3.5 values
# Needs SoloWoW's MySQL running. Restart worldserver afterwards (trainers/POIs are cached at boot).
param([switch]$Off)
. C:\SoloWoW\scripts\common.ps1
if (-not (Test-Port 3310)) {
    Start-Process 'C:\Program Files\MySQL\MySQL Server 8.4\bin\mysqld.exe' -ArgumentList '--defaults-file=C:\SoloWoW\db\my.ini' -WindowStyle Hidden
    $t = 0; while (-not (Test-Port 3310) -and $t -lt 60) { Start-Sleep 1; $t++ }
    if (-not (Test-Port 3310)) { Write-Host 'The database did not start - nothing was changed.'; exit 1 }
}

# Riding: spell -> (Era level, Era cost in copper). Stock 3.3.5: 20/4g, 40/50g, 60/250g, 70/5000g, 77/1000g.
#   Apprentice 40 / 20g and Journeyman (epic) 60 / 900g as in vanilla; Expert (flying) 70 / 800g as in TBC.
$riding = @{ 33388 = @(40, 200000); 33391 = @(60, 9000000); 34090 = @(70, 8000000) }

# ---- the difficulty revert (Nov 17 night) -----------------------------------------------------------------------
# Facts (what creatures and quests were like before Blizzard eased them) come from mod-individual-progression at a
# pinned commit, fetched here and never shipped (MIT; scripts\era\era-data.py takes only an allow-list of columns).
$EraPin   = 'e1795553d381e561ab5f37585566c22e23cdff73'
$EraDir   = 'C:\SoloWoW\build\era'
$EraRef   = "$EraDir\mod-individual-progression"
$EraFacts = "$EraDir\era-facts.sql"
$CtCols = 'faction','rank','minlevel','maxlevel','HealthModifier','DamageModifier','ArmorModifier','ManaModifier','speed_walk','speed_run','detection_range','BaseAttackTime','RangeAttackTime','mingold','maxgold','dmgschool'
$QtCols = 'QuestLevel','MinLevel','SuggestedGroupNum','QuestInfoID','RewardXPDifficulty','RequiredNpcOrGoCount1','RequiredNpcOrGoCount2','RequiredNpcOrGoCount3','RequiredNpcOrGoCount4','RequiredItemCount1','RequiredItemCount2','RequiredItemCount3','RequiredItemCount4','RequiredItemCount5','RequiredItemCount6','RewardMoney'
function Has-Table([string]$t) { [int](Sql "SELECT COUNT(*) FROM information_schema.TABLES WHERE TABLE_SCHEMA='acore_world' AND TABLE_NAME='$t'") -gt 0 }
function Bq([string[]]$c) { ($c | ForEach-Object { '`' + $_ + '`' }) -join ', ' }

if ($Off) {
    Sql @"
INSERT IGNORE INTO quest_poi SELECT * FROM solo_era_quest_poi;
INSERT IGNORE INTO quest_poi_points SELECT * FROM solo_era_quest_poi_points;
UPDATE trainer_spell t JOIN solo_era_trainer_spell b ON b.TrainerId = t.TrainerId AND b.SpellId = t.SpellId
   SET t.ReqLevel = b.ReqLevel, t.MoneyCost = b.MoneyCost;
DROP TABLE IF EXISTS solo_era_quest_poi, solo_era_quest_poi_points, solo_era_trainer_spell;
"@ acore_world
    if (Has-Table 'solo_era_bak_creature') {
        $set = ($CtCols | ForEach-Object { "ct.``$_`` = b.``$_``" }) -join ', '
        Sql "UPDATE creature_template ct JOIN solo_era_bak_creature b ON b.entry = ct.entry SET $set" acore_world
    }
    if (Has-Table 'solo_era_bak_quest') {
        $set = ($QtCols | ForEach-Object { "q.``$_`` = b.``$_``" }) -join ', '
        Sql "UPDATE quest_template q JOIN solo_era_bak_quest b ON b.ID = q.ID SET $set" acore_world
    }
    if (Has-Table 'solo_era_bak_northrend') { Sql "UPDATE creature_template ct JOIN solo_era_bak_northrend b ON b.entry = ct.entry SET ct.DamageModifier = b.DamageModifier, ct.ArmorModifier = b.ArmorModifier" acore_world }
    if (Has-Table 'solo_era_bak_xp')      { Sql "REPLACE INTO player_xp_for_level SELECT * FROM solo_era_bak_xp" acore_world }
    if (Has-Table 'solo_era_bak_gossip')  { Sql "INSERT IGNORE INTO gossip_menu_option SELECT * FROM solo_era_bak_gossip" acore_world }
    if (Has-Table 'solo_era_bak_vendor')  { Sql "INSERT IGNORE INTO npc_vendor SELECT * FROM solo_era_bak_vendor" acore_world }
    if (Has-Table 'solo_era_bak_barber')  { Sql "INSERT IGNORE INTO gameobject SELECT * FROM solo_era_bak_barber" acore_world }
    Sql "DROP TABLE IF EXISTS solo_era_bak_creature, solo_era_bak_quest, solo_era_bak_xp, solo_era_bak_gossip, solo_era_bak_vendor, solo_era_bak_barber, solo_era_bak_northrend, solo_era_quest_xp, solo_era_xp_curve, solo_era_fact" acore_world
    Write-Host 'Era DB changes reverted (stock 3.3.5 values restored).'
    exit 0
}

# Back up once (first apply only), then apply. Safe to run again.
Sql @"
CREATE TABLE IF NOT EXISTS solo_era_quest_poi LIKE quest_poi;
CREATE TABLE IF NOT EXISTS solo_era_quest_poi_points LIKE quest_poi_points;
CREATE TABLE IF NOT EXISTS solo_era_trainer_spell LIKE trainer_spell;
INSERT IGNORE INTO solo_era_quest_poi SELECT * FROM quest_poi;
INSERT IGNORE INTO solo_era_quest_poi_points SELECT * FROM quest_poi_points;
INSERT IGNORE INTO solo_era_trainer_spell SELECT * FROM trainer_spell WHERE SpellId IN ($(($riding.Keys) -join ','));
DELETE FROM quest_poi_points;
DELETE FROM quest_poi;
"@ acore_world
foreach ($s in $riding.Keys) {
    Sql "UPDATE trainer_spell SET ReqLevel = $($riding[$s][0]), MoneyCost = $($riding[$s][1]) WHERE SpellId = $s" acore_world
}

# the facts: fetch the pinned reference and extract, once
if (-not (Test-Path $EraFacts)) {
    New-Item -ItemType Directory -Force $EraDir | Out-Null
    if (-not (Test-Path "$EraRef\.git")) {
        git init -q $EraRef
        git -C $EraRef remote add origin https://github.com/ZhengPeiRu21/mod-individual-progression.git
    }
    git -C $EraRef fetch -q --depth 1 origin $EraPin
    git -C $EraRef checkout -q FETCH_HEAD
    python "$PSScriptRoot\era\era-data.py" $EraRef $EraFacts
    if (-not (Test-Path $EraFacts)) { Write-Host 'Could not fetch the era facts (no internet?) - the difficulty revert was skipped.' }
}
if (Test-Path $EraFacts) {
    $env:MYSQL_PWD = 'acore'
    cmd /c "`"$Mysql`" --host=127.0.0.1 --port=3310 --user=acore acore_world < `"$EraFacts`""
    # back up what we change, once (the first apply) - so -Off gives back stock 3.3.5
    if (-not (Has-Table 'solo_era_bak_creature')) {
        Sql "CREATE TABLE solo_era_bak_creature AS SELECT entry, $(Bq $CtCols) FROM creature_template WHERE entry IN (SELECT DISTINCT k FROM solo_era_fact WHERE tbl = 'creature_template')" acore_world
        Sql "ALTER TABLE solo_era_bak_creature ADD PRIMARY KEY (entry)" acore_world
    }
    if (-not (Has-Table 'solo_era_bak_quest')) {
        Sql "CREATE TABLE solo_era_bak_quest AS SELECT ID, $(Bq $QtCols) FROM quest_template WHERE ID IN (SELECT DISTINCT k FROM solo_era_fact WHERE tbl = 'quest_template')" acore_world
        Sql "ALTER TABLE solo_era_bak_quest ADD PRIMARY KEY (ID)" acore_world
    }
    if (-not (Has-Table 'solo_era_bak_xp')) { Sql "CREATE TABLE solo_era_bak_xp LIKE player_xp_for_level; INSERT INTO solo_era_bak_xp SELECT * FROM player_xp_for_level" acore_world }
    foreach ($c in $CtCols) {
        Sql "UPDATE creature_template ct JOIN solo_era_fact f ON f.tbl = 'creature_template' AND f.k = ct.entry AND f.col = '$c' SET ct.``$c`` = f.val" acore_world
    }
    foreach ($c in $QtCols) {
        Sql "UPDATE quest_template q JOIN solo_era_fact f ON f.tbl = 'quest_template' AND f.k = q.ID AND f.col = '$c' SET q.``$c`` = f.val" acore_world
    }
    Sql "REPLACE INTO player_xp_for_level SELECT lvl, xp FROM solo_era_xp_curve" acore_world
}

# Northrend to the same rules (Nov 17 night): the revert gave the old world's ordinary outdoor creatures x1.22 damage and
# x1.06 armour on average (measured: vanilla facts against 3.3.5); Northrend's ordinary outdoor creatures get the same.
# Its elites are already there (8.5% of outdoor spawns, against 7.5% on the reverted old continents); its missing rares
# (28, against ~336 in the old world) are the sim's work, not data.
$NrDmg = 1.22; $NrArmor = 1.06
if (-not (Has-Table 'solo_era_bak_northrend')) {
    Sql "CREATE TABLE solo_era_bak_northrend AS SELECT ct.entry, ct.DamageModifier, ct.ArmorModifier FROM creature_template ct WHERE ct.``rank`` = 0 AND ct.npcflag = 0 AND ct.type <> 8 AND ct.entry IN (SELECT id FROM creature WHERE map = 571) AND ct.entry NOT IN (SELECT id FROM creature WHERE map <> 571) AND ct.entry NOT IN (SELECT k FROM solo_era_fact WHERE tbl = 'creature_template')" acore_world
    Sql "ALTER TABLE solo_era_bak_northrend ADD PRIMARY KEY (entry)" acore_world
}
Sql "UPDATE creature_template ct JOIN solo_era_bak_northrend b ON b.entry = ct.entry SET ct.DamageModifier = b.DamageModifier * $NrDmg, ct.ArmorModifier = b.ArmorModifier * $NrArmor" acore_world

# comforts that go (the user, Nov 17 22:30 - only the later flight paths stay): dual spec, heirlooms, the barbershop
if (-not (Has-Table 'solo_era_bak_gossip')) {
    Sql "CREATE TABLE solo_era_bak_gossip LIKE gossip_menu_option; INSERT INTO solo_era_bak_gossip SELECT * FROM gossip_menu_option WHERE OptionText LIKE '%Dual Talent%'" acore_world
    Sql "CREATE TABLE solo_era_bak_vendor LIKE npc_vendor; INSERT INTO solo_era_bak_vendor SELECT v.* FROM npc_vendor v JOIN item_template i ON i.entry = v.item WHERE i.Quality = 7" acore_world
    Sql "CREATE TABLE solo_era_bak_barber LIKE gameobject; INSERT INTO solo_era_bak_barber SELECT g.* FROM gameobject g JOIN gameobject_template t ON t.entry = g.id WHERE t.type = 32" acore_world
}
Sql "DELETE o FROM gossip_menu_option o JOIN solo_era_bak_gossip b ON b.MenuID = o.MenuID AND b.OptionID = o.OptionID" acore_world
Sql "DELETE v FROM npc_vendor v JOIN solo_era_bak_vendor b ON b.entry = v.entry AND b.item = v.item AND b.ExtendedCost = v.ExtendedCost" acore_world
Sql "DELETE g FROM gameobject g JOIN solo_era_bak_barber b ON b.guid = g.guid" acore_world

Write-Host 'Era DB applied:'
Sql "SELECT 'era creatures', COUNT(DISTINCT k) FROM solo_era_fact WHERE tbl = 'creature_template' UNION ALL SELECT 'era quests', COUNT(DISTINCT k) FROM solo_era_fact WHERE tbl = 'quest_template' UNION ALL SELECT 'XP to 60 (vanilla 4.08M)', SUM(Experience) FROM player_xp_for_level WHERE Level < 60 UNION ALL SELECT 'quests with old XP', COUNT(*) FROM solo_era_quest_xp UNION ALL SELECT 'Northrend creatures x1.22 dmg', COUNT(*) FROM solo_era_bak_northrend" acore_world
Sql "SELECT 'quest_poi rows', COUNT(*) FROM quest_poi UNION ALL SELECT 'backed up', COUNT(*) FROM solo_era_quest_poi" acore_world
Sql "SELECT SpellId, ReqLevel, MoneyCost/10000 AS gold FROM trainer_spell WHERE SpellId IN (33388,33391,34090) GROUP BY SpellId, ReqLevel, MoneyCost" acore_world
