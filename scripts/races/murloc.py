# SoloWoW: Murloc as a playable race (v1), built from the game's own murloc and our own table rows.
#   python murloc.py            -> build\races\patch-X\ (client files) and build\races\server-dbc\ (server tables)
# Race 15 (the stock NPC "Skeleton" race slot, used by only ~25 NPC looks). Horde, Orcish, the creature murloc body
# (hands for weapons; armour is hidden server-side), five skin colours (the game's murloc skins), classes per our
# convention: Warrior, Hunter, Rogue, Priest, Shaman, Mage. Character creation via the worgoblin glue screens + a 13th
# race button. Never copies another project's files: everything comes from the stock client and the open module.
import os, shutil, struct, sys
from dbcfile import Dbc

RACE, BIT = 15, 1 << 14
CLASSES = [1, 3, 4, 5, 7, 8]
SKINS = ["Sahauginskin", "SahauginskinBlue", "SahauginskinGray", "SahauginskinOrange", "SahauginskinWhite"]
SRC = r"C:\SoloWoW\server\data\dbc"
GLUE = r"C:\SoloWoW\core\modules\mod-worgoblin\data\patch\Interface\GlueXML"
MODEL = r"C:\SoloWoW\reference\reforged-survey\extract_murloc\patch.MPQ\Creature\Murloc"
OUT = r"C:\SoloWoW\build\races\patch-X"
SDBC = r"C:\SoloWoW\build\races\server-dbc"
UNDEAD, ORC = 1 << 4, 1 << 1

shutil.rmtree(OUT, ignore_errors=True); shutil.rmtree(SDBC, ignore_errors=True)
os.makedirs(OUT + r"\DBFilesClient"); os.makedirs(SDBC)
def save(d, name):
    d.save(os.path.join(OUT, "DBFilesClient", name)); shutil.copy(os.path.join(OUT, "DBFilesClient", name), SDBC)

# ---- model: the creature murloc, its skin texture turned into a character skin (type 1) so skin colours apply
os.makedirs(OUT + r"\Character\Murloc\Male"); os.makedirs(OUT + r"\Character\Murloc\Female")
m2 = bytearray(open(MODEL + r"\Murloc.m2", "rb").read())
n, o = struct.unpack_from("<II", m2, 80)
for i in range(n):
    t = struct.unpack_from("<I", m2, o + i * 16)[0]
    if t == 11: struct.pack_into("<II", m2, o + i * 16, 1, 3)       # monster skin 1 -> character skin, wrap x/y
for sex in ("Male", "Female"):
    open(OUT + rf"\Character\Murloc\{sex}\Murloc{sex}.m2", "wb").write(m2)
    shutil.copy(MODEL + r"\Murloc00.skin", OUT + rf"\Character\Murloc\{sex}\Murloc{sex}00.skin")

# ---- CreatureModelData + CreatureDisplayInfo: the player bodies
md = Dbc(SRC + r"\CreatureModelData.dbc")
base = md.row(md.find(31))                                          # Creature\Murloc\Murloc.mdx
model_ids = {}
for sex in ("Male", "Female"):
    r = list(base); r[0] = md.max_id() + 1; r[2] = md.add_string(rf"Character\Murloc\{sex}\Murloc{sex}.mdx")
    md.add_row(r); model_ids[sex] = r[0]
save(md, "CreatureModelData.dbc")
cdi = Dbc(SRC + r"\CreatureDisplayInfo.dbc")
cbase = cdi.row(cdi.find(31))
display = {}
for sex in ("Male", "Female"):
    r = list(cbase); r[0] = cdi.max_id() + 1; r[1] = model_ids[sex]; r[3] = 0; r[4] = 0x3F800000
    r[6] = r[7] = r[8] = 0
    cdi.add_row(r); display[sex] = r[0]
save(cdi, "CreatureDisplayInfo.dbc")

# ---- ChrRaces: race 15 becomes Murloc, from the Undead row (Horde, Orcish, the Forsaken's reputation start)
cr = Dbc(SRC + r"\ChrRaces.dbc")
r = cr.row(cr.find(5)); r[0] = RACE
r[4], r[5] = display["Male"], display["Female"]
r[6] = cr.add_string("Ta")                       # item prefix: borrowed (armour isn't drawn on murlocs anyway)
r[11] = cr.add_string("Murloc")                  # client file string: glue screens, RACE_INFO_MURLOC
r[12] = 0                                        # no intro cinematic
name = cr.add_string("Murloc")
for f in range(14, 30): r[f] = name if f == 14 else 0
for f in (31, 48): r[f] = name
r[65] = r[66] = cr.add_string("NORMAL")
cr.set_row(cr.find(RACE), r)
save(cr, "ChrRaces.dbc")

# ---- CharBaseInfo: which classes (2-byte rows: race, class)
cb = Dbc(SRC + r"\CharBaseInfo.dbc")
cb.raw = [x for x in cb.raw if x[0] != RACE] + [bytearray([RACE, c]) for c in CLASSES]
save(cb, "CharBaseInfo.dbc")

# ---- CharSections: skins (the five murloc colours), and one plain face / facial hair / hair / underwear
cs = Dbc(SRC + r"\CharSections.dbc")
cs.raw = [x for x in cs.raw if struct.unpack_from("<I", x, 4)[0] != RACE]
nid = cs.max_id() + 1
for sex in (0, 1):
    for c, skin in enumerate(SKINS):
        cs.add_row([nid, RACE, sex, 0, cs.add_string(rf"Creature\Murloc\{skin}.blp"), 0, 0, 17, 0, c]); nid += 1
    for section in (1, 2, 3, 4):
        cs.add_row([nid, RACE, sex, section, 0, 0, 0, 17, 0, 0]); nid += 1
save(cs, "CharSections.dbc")
hg = Dbc(SRC + r"\CharHairGeosets.dbc")
hid = hg.max_id() + 1
for sex in (0, 1): hg.add_row([hid + sex, RACE, sex, 0, 0, 0])
save(hg, "CharHairGeosets.dbc")
fh = Dbc(SRC + r"\CharacterFacialHairStyles.dbc")
for sex in (0, 1): fh.add_row([RACE, sex, 0, 0, 0, 0, 0, 0])
save(fh, "CharacterFacialHairStyles.dbc")

# ---- CharStartOutfit: the Undead class kits (byte 0 of field 1 is the race)
so = Dbc(SRC + r"\CharStartOutfit.dbc")
oid = so.max_id() + 1
for i in range(len(so.raw)):
    raw = so.raw[i]                                  # id, race/class/sex/outfit bytes, items[24], displays[24], slots[24]
    if raw[4] == 5 and raw[5] in CLASSES:
        new = bytearray(raw); struct.pack_into("<I", new, 0, oid); oid += 1; new[4] = RACE
        so.add_raw(new)
save(so, "CharStartOutfit.dbc")

# ---- everything the Horde shares (masks with both Orc and Undead): languages, skills, reputations
def widen(d, fields, name):
    k = 0
    for i in range(len(d.raw)):
        row = d.row(i); hit = False
        for f in fields:
            if row[f] not in (0, 0xFFFFFFFF) and row[f] & UNDEAD and row[f] & ORC and not row[f] & BIT:
                row[f] |= BIT; hit = True
        if hit: d.set_row(i, row); k += 1
    save(d, name); print(f"{name}: {k} rows now include murlocs")
widen(Dbc(SRC + r"\SkillRaceClassInfo.dbc"), [2], "SkillRaceClassInfo.dbc")
widen(Dbc(SRC + r"\SkillLineAbility.dbc"), [3], "SkillLineAbility.dbc")
widen(Dbc(SRC + r"\Faction.dbc"), [2, 3, 4, 5], "Faction.dbc")

# ---- character creation screen: a 13th race button, the Undead's background/lighting/icon for now, our text
os.makedirs(OUT + r"\Interface\GlueXML")
xml = open(GLUE + r"\CharacterCreate.xml", encoding="utf-8").read()
anchor = '<CheckButton name="CharacterCreateRaceButton6"'
button13 = ('<CheckButton name="CharacterCreateRaceButton13" inherits="CharacterCreateRaceButtonTemplate" id="13">\n'
            '\t\t\t\t\t\t<Anchors>\n\t\t\t\t\t\t\t<Anchor point="TOPLEFT" relativeTo="CharacterCreateRaceButton11" relativePoint="BOTTOMLEFT" x="0" y="-10"/>\n'
            '\t\t\t\t\t\t</Anchors>\n\t\t\t\t\t</CheckButton>\n\t\t\t\t\t')
assert anchor in xml
open(OUT + r"\Interface\GlueXML\CharacterCreate.xml", "w", encoding="utf-8").write(xml.replace(anchor, button13 + anchor, 1))
lua = open(GLUE + r"\CharacterCreate.lua", encoding="utf-8").read().replace("MAX_RACES = 12;", "MAX_RACES = 13;", 1)
lua += ('\n-- SoloWoW: murlocs borrow the Undead icon until they have their own\n'
        'RACE_ICON_TCOORDS["MURLOC_MALE"] = RACE_ICON_TCOORDS["SCOURGE_MALE"];\n'
        'RACE_ICON_TCOORDS["MURLOC_FEMALE"] = RACE_ICON_TCOORDS["SCOURGE_FEMALE"];\n')
open(OUT + r"\Interface\GlueXML\CharacterCreate.lua", "w", encoding="utf-8").write(lua)
gp = open(GLUE + r"\GlueParent.lua", encoding="utf-8").read()
gp = gp.replace("function SetBackgroundModel(model, name)\n", "function SetBackgroundModel(model, name)\n    if ( name == \"Murloc\" ) then name = \"Scourge\"; end   -- SoloWoW: no murloc scene yet\n", 1)
open(OUT + r"\Interface\GlueXML\GlueParent.lua", "w", encoding="utf-8").write(gp)
gs = open(GLUE + r"\GlueStrings.lua", encoding="utf-8").read()
gs += ('\n-- SoloWoW: Murloc\n'
       'RACE_INFO_MURLOC = "Mrrgl. The murlocs of Azeroth\'s shores have always been there - in every bay, every lake, every river mouth, long before the cities rose and long after they fall. Most of their kind still see a stranger as a meal. A few, sharper and stranger than the rest, have learned the tongues of the land-walkers and walked out of the water to see what the world is made of.";\n'
       'RACE_INFO_MURLOC_FEMALE = RACE_INFO_MURLOC;\n'
       'ABILITY_INFO_MURLOC1 = "- Breathe underwater for far longer than land-walkers.";\n'
       'ABILITY_INFO_MURLOC2 = "- Slippery: harder to hit in melee.";\n'
       'ABILITY_INFO_MURLOC3 = "- Increased resistance to Nature.";\n')
open(OUT + r"\Interface\GlueXML\GlueStrings.lua", "w", encoding="utf-8").write(gs)
print("murloc build ready:", OUT, "displays", display, "models", model_ids)
