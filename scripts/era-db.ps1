# SoloWoW "Era" preset - world DB part. Reversible: originals are kept in solo_era_* backup tables.
#   era-db.ps1        apply   (quest map/minimap objective areas off, vanilla/TBC riding requirements)
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

if ($Off) {
    Sql @"
INSERT IGNORE INTO quest_poi SELECT * FROM solo_era_quest_poi;
INSERT IGNORE INTO quest_poi_points SELECT * FROM solo_era_quest_poi_points;
UPDATE trainer_spell t JOIN solo_era_trainer_spell b ON b.TrainerId = t.TrainerId AND b.SpellId = t.SpellId
   SET t.ReqLevel = b.ReqLevel, t.MoneyCost = b.MoneyCost;
DROP TABLE IF EXISTS solo_era_quest_poi, solo_era_quest_poi_points, solo_era_trainer_spell;
"@ acore_world
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
Write-Host 'Era DB applied:'
Sql "SELECT 'quest_poi rows', COUNT(*) FROM quest_poi UNION ALL SELECT 'backed up', COUNT(*) FROM solo_era_quest_poi" acore_world
Sql "SELECT SpellId, ReqLevel, MoneyCost/10000 AS gold FROM trainer_spell WHERE SpellId IN (33388,33391,34090) GROUP BY SpellId, ReqLevel, MoneyCost" acore_world
