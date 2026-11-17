-- SoloWoW: Murloc (race 15) server rows. Re-runnable.
-- Start: Deathknell, with the Undead (Brightwater Lake was ringed by level 5-7s - too much for a level 1, Nov 17).
DELETE FROM `playercreateinfo` WHERE `race` = 15;
INSERT INTO `playercreateinfo` (`race`, `class`, `map`, `zone`, `position_x`, `position_y`, `position_z`, `orientation`) VALUES
(15, 1, 0, 85, 1676.71, 1678.31, 121.67, 2.70526),
(15, 3, 0, 85, 1676.71, 1678.31, 121.67, 2.70526),
(15, 4, 0, 85, 1676.71, 1678.31, 121.67, 2.70526),
(15, 5, 0, 85, 1676.71, 1678.31, 121.67, 2.70526),
(15, 7, 0, 85, 1676.71, 1678.31, 121.67, 2.70526),
(15, 8, 0, 85, 1676.71, 1678.31, 121.67, 2.70526);

-- action bars: the Undead's, without the Undead racials (Cannibalize, Will of the Forsaken)
DELETE FROM `playercreateinfo_action` WHERE `race` = 15;
INSERT INTO `playercreateinfo_action` (`race`, `class`, `button`, `action`, `type`)
SELECT 15, `class`, `button`, `action`, `type` FROM `playercreateinfo_action`
WHERE `race` = 5 AND `class` IN (1, 3, 4, 5, 7, 8) AND NOT (`type` = 0 AND `action` IN (20577, 7744));

-- skills every Horde race shares (masks with both Orc and Undead) - languages, weapons
UPDATE `playercreateinfo_skills` SET `raceMask` = `raceMask` | 16384
WHERE `raceMask` <> 0 AND (`raceMask` & 2) AND (`raceMask` & 16);

-- racials (stock spells for now): Underwater Breathing, Nature Resistance, Quickness (dodge)
DELETE FROM `playercreateinfo_spell_custom` WHERE `racemask` = 16384;
INSERT INTO `playercreateinfo_spell_custom` (`racemask`, `classmask`, `Spell`, `Note`) VALUES
(16384, 0, 5227,  'Murloc - Underwater Breathing'),
(16384, 0, 20551, 'Murloc - Nature Resistance'),
(16384, 0, 20582, 'Murloc - Slippery (Quickness)');

-- small and quick
DELETE FROM `player_race_stats` WHERE `Race` = 15;
INSERT INTO `player_race_stats` (`Race`, `Strength`, `Agility`, `Stamina`, `Intellect`, `Spirit`) VALUES (15, -2, 3, 0, -1, 2);
