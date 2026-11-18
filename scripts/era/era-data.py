# SoloWoW Era - vanilla/TBC facts for the difficulty revert (Nov 17 night).
#
# The facts (what a creature or quest was like before Blizzard eased it) come from mod-individual-progression
# (MIT, https://github.com/ZhengPeiRu21/mod-individual-progression), fetched by the installer at a pinned commit -
# never shipped with SoloWoW. We take only the difficulty facts on an allow-list below: no scripts, spawns, texts,
# loot or anything that needs that module's own code. Our own rules (the player's power in old content, armour,
# daze, regen, quest XP by difficulty and distance, the sim's pace, Wrath to the same formula) live in our code.
#
#   python era-data.py <reference checkout> <out.sql>
# writes REPLACE-free UPDATEs into solo_era_* staging tables; era-db.ps1 applies them with backups (reversible).
import os, re, sys

PIN = "e1795553d381e561ab5f37585566c22e23cdff73"

# table -> (key column, columns we take)
ALLOW = {
    "creature_template": ("entry", {"faction", "rank", "minlevel", "maxlevel", "HealthModifier", "DamageModifier",
                                    "ArmorModifier", "ManaModifier", "speed_walk", "speed_run", "detection_range",
                                    "BaseAttackTime", "RangeAttackTime", "mingold", "maxgold", "dmgschool"}),
    "quest_template": ("ID", {"QuestLevel", "MinLevel", "SuggestedGroupNum", "QuestInfoID", "RewardXPDifficulty",
                              "RequiredNpcOrGoCount1", "RequiredNpcOrGoCount2", "RequiredNpcOrGoCount3", "RequiredNpcOrGoCount4",
                              "RequiredItemCount1", "RequiredItemCount2", "RequiredItemCount3", "RequiredItemCount4",
                              "RequiredItemCount5", "RequiredItemCount6", "RewardMoney"}),
}
# files that are about other things (progression gates, raids that module rebuilds, PvP, its own systems)
SKIP = re.compile(r"^(00_cleanup|adjustment_auras|aq_|av|cs_|ipp_|naxx40|pvp_|player_|restore_|si|spell_|warlock_|wotlk_|"
                  r"item_|tbc_item|vanilla_item|tbc_vendors|vanilla_vendors|tbc_reference_loot|tbc_loot|class_trainers|professions|"
                  r"mounts_and_riding|quest_missing|quest_the_masq|boss_lord_kazzak|dungeon_attunements)")

ASSIGN = re.compile(r"`?([A-Za-z_0-9]+)`?\s*=\s*('(?:[^'\\]|\\.)*'|-?[0-9.]+)")
UPD = re.compile(r"UPDATE\s+`?([a-z_]+)`?\s+SET\s+(.*?)\s+WHERE\s+(.*?);", re.I | re.S)

def keys(where, keycol):
    w = where.strip()
    m = re.fullmatch(rf"`?{keycol}`?\s*=\s*'?(\d+)'?", w, re.I)
    if m: return [int(m.group(1))]
    m = re.fullmatch(rf"`?{keycol}`?\s+IN\s*\(([\d,\s']+)\)", w, re.I)
    if m: return [int(x) for x in re.findall(r"\d+", m.group(1))]
    return None                                     # a condition we don't take (by name, by other columns)

def main():
    ref, out = sys.argv[1], sys.argv[2]
    base = os.path.join(ref, "data", "sql", "world", "base")
    facts = {t: {} for t in ALLOW}                  # table -> key -> {col: value}
    skipped = 0
    for fn in sorted(os.listdir(base)):             # the module's own order: a later file wins
        if not fn.endswith(".sql") or SKIP.match(fn): continue
        text = open(os.path.join(base, fn), encoding="utf-8", errors="replace").read()
        text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
        text = re.sub(r"(?m)^\s*--.*$", "", text)
        for m in UPD.finditer(text):
            table = m.group(1).lower()
            if table not in ALLOW: continue
            keycol, cols = ALLOW[table]
            ks = keys(m.group(3), keycol)
            if ks is None: skipped += 1; continue
            sets = {c: v for c, v in ASSIGN.findall(m.group(2)) if c in cols}
            if not sets: continue
            for k in ks: facts[table].setdefault(k, {}).update(sets)
    # the vanilla XP curve and the per-quest vanilla XP
    xp = open(os.path.join(base, "vanilla_xp_tables.sql"), encoding="utf-8").read()
    levels = [(int(a), int(b)) for a, b in re.findall(r"\(\s*(\d+)\s*,\s*(\d+)\s*\)", xp)]
    qx = open(os.path.join(base, "quest_xp_table.sql"), encoding="utf-8").read()
    questxp = [(int(a), int(b)) for a, b in re.findall(r"\(\s*(\d+)\s*,\s*(\d+)\s*\)", qx)]

    with open(out, "w", encoding="utf-8", newline="\n") as w:
        w.write(f"-- SoloWoW Era facts, from mod-individual-progression {PIN} (MIT). Generated; not shipped.\n")
        w.write("DROP TABLE IF EXISTS solo_era_fact;\n")
        w.write("CREATE TABLE solo_era_fact (tbl VARCHAR(32) NOT NULL, k INT UNSIGNED NOT NULL, col VARCHAR(40) NOT NULL, val VARCHAR(64) NOT NULL, PRIMARY KEY (tbl, k, col)) ENGINE=InnoDB;\n")
        rows = [(t, k, c, v) for t, d in facts.items() for k, cs in d.items() for c, v in cs.items()]
        for i in range(0, len(rows), 500):
            part = ",".join("('{}',{},'{}','{}')".format(t, k, c, v.strip("'").replace("'", "''")) for t, k, c, v in rows[i:i + 500])
            w.write(f"INSERT INTO solo_era_fact VALUES {part};\n")
        w.write("DROP TABLE IF EXISTS solo_era_xp_curve;\nCREATE TABLE solo_era_xp_curve (lvl INT UNSIGNED NOT NULL PRIMARY KEY, xp INT UNSIGNED NOT NULL) ENGINE=InnoDB;\n")
        w.write("INSERT INTO solo_era_xp_curve VALUES " + ",".join(f"({a},{b})" for a, b in levels) + ";\n")
        w.write("DROP TABLE IF EXISTS solo_era_quest_xp;\nCREATE TABLE solo_era_quest_xp (quest INT UNSIGNED NOT NULL PRIMARY KEY, xp INT UNSIGNED NOT NULL) ENGINE=InnoDB;\n")
        for i in range(0, len(questxp), 1000):
            w.write("INSERT INTO solo_era_quest_xp VALUES " + ",".join(f"({a},{b})" for a, b in questxp[i:i + 1000]) + ";\n")
    print(f"{out}: " + ", ".join(f"{t} {len(d)} rows" for t, d in facts.items())
          + f", XP curve {len(levels)} levels, quest XP {len(questxp)} quests ({skipped} updates by other conditions left out)")

if __name__ == "__main__":
    main()
