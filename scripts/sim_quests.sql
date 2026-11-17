-- SoloWoW: the quest pool for townsfolk requests (solosim_quests.cpp). 1000 plain rows, 95000-95999, filled with each
-- request's words, objectives and reward at runtime. Empty titles on purpose: a row with no title is refilled from
-- its request when a player logs in. Safe to run again.
INSERT IGNORE INTO `quest_template` (`ID`, `QuestType`, `QuestLevel`, `MinLevel`, `LogTitle`)
WITH RECURSIVE n AS (SELECT 95000 AS id UNION ALL SELECT id + 1 FROM n WHERE id < 95999)
SELECT id, 2, 1, 1, '' FROM n;

-- board bounties: 96000-96999
INSERT IGNORE INTO `quest_template` (`ID`, `QuestType`, `QuestLevel`, `MinLevel`, `LogTitle`)
WITH RECURSIVE n AS (SELECT 96000 AS id UNION ALL SELECT id + 1 FROM n WHERE id < 96999)
SELECT id, 2, 1, 1, '' FROM n;
