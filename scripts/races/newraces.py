# SoloWoW: Naga, Tuskarr and Furbolg as playable races (Nov 17), built on top of murloc.py's output.
#   python murloc.py && python newraces.py   -> build\races\patch-X\ (client) + build\races\server-dbc\ (server tables)
#                                               + scripts\races\newraces.sql (server rows, generated from the same config)
# Race slots (all stock NPC slots, as the murloc's 15): Naga 13 (Character\Naga_, stock model), Tuskarr 17
# (Character\Tuskarr, stock model - male only: the female display already uses it), Furbolg 20 (the "Northrend Skeleton"
# slot; the creature furbolg, its skin baked in like the murloc's; its 37 NPC looks move to the Undead body).
# Classes per the user's convention (Reforged's lists, by lore). Kits, action bars and class skills come from template
# races that have each class; quests, items and reputations from the race they start beside. Armour is hidden on all
# three server-side (solosim_races.cpp); the creation preview shows weapons only. Never copies another project's files.
import os, shutil, struct
from dbcfile import Dbc

OUT = r"C:\SoloWoW\build\races\patch-X"
SDBC = r"C:\SoloWoW\build\races\server-dbc"
SRC = r"C:\SoloWoW\build\races\source-dbc"                    # murloc.py's clean copy of the server tables
FURBOLG = r"C:\SoloWoW\reference\reforged-survey\extract_furbolg"
SQL = r"C:\SoloWoW\scripts\races\newraces.sql"
HUMAN, ORC, DWARF, NELF, UNDEAD, TAUREN, GNOME, TROLL, GOBLIN, BELF, DRAENEI = (1 << i for i in range(11))

RACES = [
    dict(id=13, key="NAGA_", file="Naga_", name="Naga", horde=True, template=10, classes=[1, 3, 5, 7, 8],
         kit={1: 8, 3: 10, 5: 10, 7: 8, 8: 10},                       # warrior/shaman: troll kits; the rest: blood elf
         start=(530, 3431, 10349.6, -6357.29, 33.4026, 5.31605),      # Sunstrider Isle, Eversong Woods
         share=BELF | TROLL, quests=BELF | TROLL, rep=lambda m: bool(m & BELF) and bool(m & (ORC | UNDEAD)),
         racials=[(5227, "Naga - Underwater Breathing"), (20596, "Naga - Frost Resistance")], stats=(1, 1, 0, 1, -1),
         icon=r"Interface\\Icons\\Achievement_Boss_LadyVashj", scene="BloodElf",
         info="The naga were night elves once - the Highborne of old Zin-Azshari, drowned with their queen when the Well "
              "collapsed and remade in the deep. Some kept faith with Vashj and followed Kael'thas; their kin walk the "
              "shores of Eversong now, cold-eyed and patient, serving the Horde for reasons of their own.",
         abilities=["- Breathe underwater far longer than the land-born.", "- Increased resistance to Frost.", "- At home in the sea."]),
    dict(id=17, key="TUSKARR", file="Tuskarr", name="Tuskarr", horde=False, template=3, classes=[1, 3, 5, 7, 8],
         kit={1: 3, 3: 3, 5: 3, 7: 11, 8: 7},                         # warrior/hunter/priest: dwarf; shaman: draenei; mage: gnome
         start=(0, 1, -6240.32, 331.033, 382.758, 6.17716),           # Coldridge Valley, Dun Morogh
         share=DWARF | DRAENEI | GNOME, quests=DWARF | GNOME, rep=lambda m: bool(m & DWARF) and bool(m & HUMAN),
         racials=[(20596, "Tuskarr - Frost Resistance"), (20550, "Tuskarr - Endurance")], stats=(1, -1, 1, 0, 0),
         icon=r"Interface\\Icons\\Achievement_Reputation_Tuskarr", scene="Dwarf",
         info="The tuskarr of the northern shores live by the sea and the hunt, as their fathers did. A few have come "
              "south with the dwarves of Ironforge, who know a good fisherman and a better friend when they see one.",
         abilities=["- Increased resistance to Frost.", "- Hardier than most: more health.", "- Born to the cold."]),
    dict(id=20, key="FURBOLG", file="Furbolg", name="Furbolg", horde=False, template=11, classes=[1, 3, 7, 11],
         kit={1: 11, 3: 11, 7: 11, 11: 4},                            # draenei; druids: night elf
         start=(530, 3526, -3961.64, -13931.2, 100.615, 2.08364),     # Ammen Vale, Azuremyst Isle (by Stillpine lands)
         share=DRAENEI | NELF, quests=DRAENEI | NELF, rep=lambda m: bool(m & DRAENEI) and bool(m & HUMAN),
         racials=[(20551, "Furbolg - Nature Resistance"), (20550, "Furbolg - Endurance")], stats=(2, -1, 1, -1, 1),
         icon=r"Interface\\Icons\\Achievement_Reputation_TimbermawHold", scene="Draenei", creature=True,
         info="The furbolgs are an old people of the forests, wise in the ways of the earth. The Stillpine of Azuremyst "
              "took the draenei in when they fell from the sky; now their young walk beside them, and the Alliance has "
              "gained friends who do not forget.",
         abilities=["- Increased resistance to Nature.", "- Hardier than most: more health.", "- Stout of heart."]),
]

def popcount(x): return bin(x & 0x7FF).count("1")             # the eleven old races

def load(name, server=False):
    for d in ((SDBC,) if server else (os.path.join(OUT, "DBFilesClient"),)) + (SRC,):
        p = os.path.join(d, name)
        if os.path.exists(p): return Dbc(p)
    raise FileNotFoundError(name)

def save(d, name):
    d.save(os.path.join(OUT, "DBFilesClient", name)); shutil.copy(os.path.join(OUT, "DBFilesClient", name), SDBC)

# ---- the furbolg body: the creature model, its skin and gear textures baked in (type 0), under Character\Furbolg
fm2 = bytearray(open(FURBOLG + r"\patch.MPQ\Creature\furbolg\Furbolg.M2", "rb").read())
n, o = struct.unpack_from("<II", fm2, 80)
for i in range(n):
    t = struct.unpack_from("<I", fm2, o + i * 16)[0]
    name = {11: r"Creature\furbolg\FurbolgSkinBrown.blp", 12: r"Creature\furbolg\FurbolgStuffBrown.blp"}.get(t)
    if name:
        s = name.encode() + b"\0"; at = len(fm2); fm2 += s
        struct.pack_into("<IIII", fm2, o + i * 16, 0, 0, len(s), at)
for sex in ("Male", "Female"):
    os.makedirs(OUT + rf"\Character\Furbolg\{sex}", exist_ok=True)
    open(OUT + rf"\Character\Furbolg\{sex}\Furbolg{sex}.m2", "wb").write(fm2)
    shutil.copy(FURBOLG + r"\common-2.MPQ\Creature\furbolg\Furbolg00.skin", OUT + rf"\Character\Furbolg\{sex}\Furbolg{sex}00.skin")

# ---- the tuskarr: the game only has a male body; the client loads Character\<file>\<Sex>\<file><Sex>.m2 by the
#      character's sex, so the female is the male model (and its skins and external animations) under the female name
TUSK = r"C:\SoloWoW\reference\reforged-survey\extract_tuskarr\patch.MPQ\Character\TUSKARR\MALE"
os.makedirs(OUT + r"\Character\Tuskarr\Female", exist_ok=True)
for f in os.listdir(TUSK):
    if f.lower().startswith("tuskarrmale"):
        shutil.copy(os.path.join(TUSK, f), OUT + r"\Character\Tuskarr\Female\TuskarrFemale" + f[len("TuskarrMale"):])

md = load("CreatureModelData.dbc"); cdi = load("CreatureDisplayInfo.dbc")
base_model = next(md.row(i) for i in range(len(md.raw)) if md.s(md.row(i)[2]).lower().replace("/", "\\") == r"creature\furbolg\furbolg.mdx")
base_disp = next(cdi.row(i) for i in range(len(cdi.raw)) if cdi.row(i)[1] == base_model[0])
furbolg_display = {}
for sex in ("Male", "Female"):
    r = list(base_model); r[0] = md.max_id() + 1; r[2] = md.add_string(rf"Character\Furbolg\{sex}\Furbolg{sex}.mdx"); md.add_row(r)
    d = list(base_disp); d[0] = cdi.max_id() + 1; d[1] = r[0]; d[3] = 0; d[4] = struct.unpack("<I", struct.pack("<f", 0.9))[0]
    d[6] = d[7] = d[8] = 0; cdi.add_row(d); furbolg_display[sex] = d[0]
save(md, "CreatureModelData.dbc"); save(cdi, "CreatureDisplayInfo.dbc")

# ---- ChrRaces: each slot becomes playable from its template race's row (side, language, reputation start), keeping
#      its own looks (the stock displays, or the furbolg's) and names
cr = load("ChrRaces.dbc")
for R in RACES:
    own = cr.row(cr.find(R["id"])); r = cr.row(cr.find(R["template"]))
    r[0] = R["id"]
    r[4], r[5] = (furbolg_display["Male"], furbolg_display["Female"]) if R.get("creature") else (own[4], own[5])
    r[6] = cr.add_string(R["file"][:2])                       # item prefix (armour isn't drawn on these bodies anyway)
    r[11] = cr.add_string(R["file"])                          # client file string: Character\<file>\..., glue keys
    r[12] = 0                                                 # no intro cinematic
    nm = cr.add_string(R["name"])
    for f in range(14, 30): r[f] = nm if f == 14 else 0
    for f in (31, 48): r[f] = nm
    r[65] = r[66] = cr.add_string("NORMAL")
    cr.set_row(cr.find(R["id"]), r)
save(cr, "ChrRaces.dbc")

cb = load("CharBaseInfo.dbc")
ids = {R["id"] for R in RACES}
cb.raw = [x for x in cb.raw if x[0] not in ids] + [bytearray([R["id"], c]) for R in RACES for c in R["classes"]]
save(cb, "CharBaseInfo.dbc")

# ---- customization: a full 10 x 10 grid in every section for both sexes (a missing entry crashes the client when
#      switching races - the murloc lesson), each entry a real texture: the race's own rows repeated, else the furbolg skin
GRID = 10
cs = load("CharSections.dbc")
existing = {}
for x in cs.raw:
    race, sex, sec = struct.unpack_from("<III", x, 4)
    if race in ids: existing.setdefault((race, sex, sec), []).append(list(struct.unpack_from(f"<{cs.fields}I", x)))
cs.raw = [x for x in cs.raw if struct.unpack_from("<I", x, 4)[0] not in ids]
nid = cs.max_id() + 1
fur_tex = cs.add_string(r"Creature\furbolg\FurbolgSkinBrown.blp")
for R in RACES:
    for sex in (0, 1):
        for sec in range(5):
            src = existing.get((R["id"], sex, sec)) or existing.get((R["id"], 0, sec)) or [[0, R["id"], sex, sec, fur_tex, fur_tex if sec == 1 else 0, 0, 17, 0, 0]]
            for v in (range(1) if sec == 0 else range(GRID)):
                for c in range(GRID):
                    t = list(src[(v * GRID + c) % len(src)])
                    t[0] = nid; t[1] = R["id"]; t[2] = sex; t[3] = sec; t[8] = v; t[9] = c; nid += 1
                    cs.add_row(t)
save(cs, "CharSections.dbc")
hg = load("CharHairGeosets.dbc")
have = {(r[1], r[2], r[3]) for r in hg.rows()}
hid = hg.max_id() + 1
for R in RACES:
    for sex in (0, 1):
        for v in range(GRID):
            if (R["id"], sex, v) not in have: hg.add_row([hid, R["id"], sex, v, 0, 0]); hid += 1
save(hg, "CharHairGeosets.dbc")
fh = load("CharacterFacialHairStyles.dbc")
have = {(r[0], r[1], r[2]) for r in fh.rows()}
for R in RACES:
    for sex in (0, 1):
        for v in range(GRID):
            if (R["id"], sex, v) not in have: fh.add_row([R["id"], sex, v, 0, 0, 0, 0, 0])
save(fh, "CharacterFacialHairStyles.dbc")

# ---- starting kits: per class from the template race that has it; the client's preview keeps weapons only
WEAPON_SLOTS = {13, 14, 15, 17, 21, 22, 23, 25, 26}
so_srv = load("CharStartOutfit.dbc", server=True)
so_srv.raw = [x for x in so_srv.raw if x[4] not in ids]
oid = so_srv.max_id() + 1
added = []
for R in RACES:
    for cls, tmpl in R["kit"].items():
        for raw in list(so_srv.raw):
            if raw[4] == tmpl and raw[5] == cls:
                new = bytearray(raw); struct.pack_into("<I", new, 0, oid); oid += 1; new[4] = R["id"]
                so_srv.add_raw(new); added.append(new)
so_srv.save(os.path.join(SDBC, "CharStartOutfit.dbc"))
so_cli = load("CharStartOutfit.dbc")
so_cli.raw = [x for x in so_cli.raw if x[4] not in ids]
for raw in added:
    c = bytearray(raw)
    for k in range(24):
        if struct.unpack_from("<I", c, 200 + 4 * k)[0] not in WEAPON_SLOTS:
            for at in (8, 104, 200): struct.pack_into("<I", c, at + 4 * k, 0)
    so_cli.add_raw(c)
so_cli.save(os.path.join(OUT, "DBFilesClient", "CharStartOutfit.dbc"))

# ---- skills, abilities, reputations: rows the template races share with another race (class lines, languages,
#      weapons) - never a single race's racials
def widen(name, fields):
    d = load(name); k = 0
    for i in range(len(d.raw)):
        row = d.row(i); hit = False
        for f in fields:
            for R in RACES:
                bit = 1 << (R["id"] - 1); m = row[f]
                if m in (0, 0xFFFFFFFF) or m & bit: continue
                ok = R["rep"](m) if name == "Faction.dbc" else (bool(m & R["share"]) and popcount(m) >= 2)
                if ok: row[f] |= bit; hit = True
        if hit: d.set_row(i, row); k += 1
    save(d, name); print(f"{name}: {k} rows widened for naga, tuskarr, furbolg")
widen("SkillRaceClassInfo.dbc", [2]); widen("SkillLineAbility.dbc", [3]); widen("Faction.dbc", [2, 3, 4, 5])

# ---- the 37 NPC looks in the old "Northrend Skeleton" slot: the Undead body (they'd draw as furbolgs now)
cx = load("CreatureDisplayInfoExtra.dbc"); moved = 0
for i in range(len(cx.raw)):
    row = cx.row(i)
    if row[1] == 20: row[1] = 5; cx.set_row(i, row); moved += 1
save(cx, "CreatureDisplayInfoExtra.dbc"); print(f"CreatureDisplayInfoExtra.dbc: {moved} Northrend skeleton looks moved to the Undead body")

# ---- the creation screen: buttons 14-16, icons by race (not by button), scenes, texts
G = OUT + r"\Interface\GlueXML"
xml = open(G + r"\CharacterCreate.xml", encoding="utf-8").read()
anchor = '<CheckButton name="CharacterCreateRaceButton13"'
assert anchor in xml
extra = "".join(f'<CheckButton name="CharacterCreateRaceButton{n}" inherits="CharacterCreateRaceButtonTemplate" id="{n}">\n'
                f'\t\t\t\t\t\t<Anchors>\n\t\t\t\t\t\t\t<Anchor point="TOPLEFT" relativeTo="CharacterCreateRaceButton{n - 1}" relativePoint="BOTTOMLEFT" x="0" y="-10"/>\n'
                f'\t\t\t\t\t\t</Anchors>\n\t\t\t\t\t</CheckButton>\n\t\t\t\t\t' for n in (14, 15, 16))
i = xml.index(anchor); j = xml.index("</CheckButton>", i) + len("</CheckButton>")
xml = xml[:j] + "\n\t\t\t\t\t" + extra + xml[j:]
open(G + r"\CharacterCreate.xml", "w", encoding="utf-8").write(xml)
lua = open(G + r"\CharacterCreate.lua", encoding="utf-8").read().replace("MAX_RACES = 13;", "MAX_RACES = 16;", 1)
icons = ['\tMURLOC = "Interface\\\\Icons\\\\INV_Misc_Head_Murloc_01",'] + [f'\t{R["key"]} = "{R["icon"]}",' for R in RACES]
lua += ("\n-- SoloWoW (Nov 17): the new races have no slot in the race-icon sheets - each button and the portrait show the game's\n"
        "-- own icon for that people, found by the race's file string (the buttons move as races are added)\n"
        + "".join(f'RACE_ICON_TCOORDS["{R["key"]}_MALE"] = RACE_ICON_TCOORDS["{"BLOODELF" if R["horde"] else "DRAENEI"}_MALE"];\n'
                  f'RACE_ICON_TCOORDS["{R["key"]}_FEMALE"] = RACE_ICON_TCOORDS["{"BLOODELF" if R["horde"] else "DRAENEI"}_FEMALE"];\n' for R in RACES)
        + "local SOLO_ICONS = {\n" + "\n".join(icons) + "\n};\n"
        'local SOLO_SHEET = "Interface\\\\Glues\\\\CharacterCreate\\\\UI-CharacterCreate-Races";\n'
        'local SOLO_ROUND2 = "Interface\\\\Glues\\\\CharacterCreate\\\\UI-CharacterCreate-RacesRound";\n'
        'local function SoloRaceIcons(...)\n'
        '\tlocal index = 1;\n'
        '\tfor i = 1, select("#", ...), 3 do\n'
        '\t\tlocal fs = strupper(select(i + 1, ...) or "");\n'
        '\t\tlocal icon = SOLO_ICONS[fs];\n'
        '\t\tlocal n = _G["CharacterCreateRaceButton"..index.."NormalTexture"]; local p = _G["CharacterCreateRaceButton"..index.."PushedTexture"];\n'
        '\t\tif ( n and p ) then\n'
        '\t\t\tif ( icon ) then n:SetTexture(icon); p:SetTexture(icon); n:SetTexCoord(0.08, 0.92, 0.08, 0.92); p:SetTexCoord(0.08, 0.92, 0.08, 0.92);\n'
        '\t\t\telse n:SetTexture(SOLO_SHEET); p:SetTexture(SOLO_SHEET); end\n'
        '\t\tend\n'
        '\t\tindex = index + 1;\n'
        '\tend\n'
        'end\n'
        'local function SoloRacePortrait()\n'
        '\tlocal _, fs = GetNameForRace();\n'
        '\tlocal icon = fs and SOLO_ICONS[strupper(fs)];\n'
        '\tif ( icon ) then CharacterCreateRaceIcon:SetTexture(icon); CharacterCreateRaceIcon:SetTexCoord(0.08, 0.92, 0.08, 0.92);\n'
        '\telse CharacterCreateRaceIcon:SetTexture(SOLO_ROUND2); end\n'
        'end\n'
        'local SoloEnumerate2 = CharacterCreateEnumerateRaces;\n'
        'function CharacterCreateEnumerateRaces(...) SoloEnumerate2(...); SoloRaceIcons(...); end\n'
        'local SoloSetRace2 = SetCharacterRace;\n'
        'function SetCharacterRace(id) SoloSetRace2(id); SoloRacePortrait(); end\n'
        'local SoloSetGender2 = SetCharacterGender;\n'
        'function SetCharacterGender(sex) SoloSetGender2(sex); SoloRacePortrait(); end\n')
open(G + r"\CharacterCreate.lua", "w", encoding="utf-8").write(lua)
gp = open(G + r"\GlueParent.lua", encoding="utf-8").read()
scenes = "".join(f'    if ( name == "{R["file"]}" ) then name = "{R["scene"]}"; end   -- SoloWoW: no {R["name"].lower()} scene yet\n' for R in RACES)
gp = gp.replace("function SetBackgroundModel(model, name)\n", "function SetBackgroundModel(model, name)\n" + scenes, 1)
open(G + r"\GlueParent.lua", "w", encoding="utf-8").write(gp)
gs = open(G + r"\GlueStrings.lua", encoding="utf-8").read()
for R in RACES:
    gs += f'\n-- SoloWoW: {R["name"]}\nRACE_INFO_{R["key"]} = "{R["info"]}";\nRACE_INFO_{R["key"]}_FEMALE = RACE_INFO_{R["key"]};\n'
    gs += "".join(f'ABILITY_INFO_{R["key"]}{k + 1} = "{a}";\n' for k, a in enumerate(R["abilities"]))
open(G + r"\GlueStrings.lua", "w", encoding="utf-8").write(gs)

# ---- the server's rows, generated from the same config
q = ["-- SoloWoW: Naga (13), Tuskarr (17), Furbolg (20) - generated by newraces.py; safe to run again.\n"]
for R in RACES:
    rid, bit = R["id"], 1 << (R["id"] - 1)
    m, z, x, y, zz, oo = R["start"]
    q.append(f"\n-- {R['name']}\nDELETE FROM `playercreateinfo` WHERE `race` = {rid};\nINSERT INTO `playercreateinfo` (`race`, `class`, `map`, `zone`, `position_x`, `position_y`, `position_z`, `orientation`) VALUES\n"
             + ",\n".join(f"({rid}, {c}, {m}, {z}, {x}, {y}, {zz}, {oo})" for c in R["classes"]) + ";\n")
    q.append(f"DELETE FROM `playercreateinfo_action` WHERE `race` = {rid};\n")
    for c, t in R["kit"].items():
        q.append(f"INSERT INTO `playercreateinfo_action` (`race`, `class`, `button`, `action`, `type`) SELECT {rid}, `class`, `button`, `action`, `type` "
                 f"FROM `playercreateinfo_action` WHERE `race` = {t} AND `class` = {c} AND NOT (`type` = 0 AND `action` IN "
                 f"(26297, 20554, 26296, 50621, 20555, 20557, 20558, 58943, 28730, 25046, 50613, 28880, 59542, 59543, 59544, 59545, 59547, 59548, 20589, 20591, 20592, 20593, 20594, 20549, 20572, 33697, 33702));\n")
    q.append(f"UPDATE `playercreateinfo_skills` SET `raceMask` = `raceMask` | {bit} WHERE `raceMask` <> 0 AND (`raceMask` & {R['share']}) AND BIT_COUNT(`raceMask` & 2047) >= 2;\n")
    for table, col in (("quest_template", "AllowableRaces"), ("item_template", "AllowableRace")):
        q.append(f"UPDATE `{table}` SET `{col}` = `{col}` | {bit} WHERE `{col}` NOT IN (0, -1) AND (`{col}` & {R['quests']}) AND NOT (`{col}` & {bit});\n")
    q.append(f"UPDATE `conditions` SET `ConditionValue1` = `ConditionValue1` | {bit} WHERE `ConditionTypeOrReference` = 16 AND (`ConditionValue1` & {R['quests']}) AND NOT (`ConditionValue1` & {bit});\n")
    q.append(f"UPDATE `spell_area` SET `racemask` = `racemask` | {bit} WHERE `racemask` <> 0 AND (`racemask` & {R['quests']}) AND NOT (`racemask` & {bit});\n")
    q.append(f"DELETE FROM `playercreateinfo_spell_custom` WHERE `racemask` = {bit};\nINSERT INTO `playercreateinfo_spell_custom` (`racemask`, `classmask`, `Spell`, `Note`) VALUES\n"
             + ",\n".join(f"({bit}, 0, {s}, '{note}')" for s, note in R["racials"]) + ";\n")
    s = R["stats"]
    q.append(f"DELETE FROM `player_race_stats` WHERE `Race` = {rid};\nINSERT INTO `player_race_stats` (`Race`, `Strength`, `Agility`, `Stamina`, `Intellect`, `Spirit`) VALUES ({rid}, {s[0]}, {s[1]}, {s[2]}, {s[3]}, {s[4]});\n")
open(SQL, "w", encoding="utf-8").write("".join(q))
print("new races ready:", ", ".join(f"{R['name']} ({R['id']})" for R in RACES), "- sql:", SQL)
