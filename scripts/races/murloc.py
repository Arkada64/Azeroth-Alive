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
SRC = r"C:\SoloWoW\build\races\source-dbc"   # the server's tables as they were before any murloc edit (see below)
GLUE = r"C:\SoloWoW\core\modules\mod-worgoblin\data\patch\Interface\GlueXML"
MODEL = r"C:\SoloWoW\reference\reforged-survey\extract_murloc\patch.MPQ\Creature\Murloc"
OUT = r"C:\SoloWoW\build\races\patch-X"
SDBC = r"C:\SoloWoW\build\races\server-dbc"
UNDEAD, ORC, TAUREN, TROLL = 1 << 4, 1 << 1, 1 << 5, 1 << 7

shutil.rmtree(OUT, ignore_errors=True); shutil.rmtree(SDBC, ignore_errors=True); shutil.rmtree(SRC, ignore_errors=True)
# start from clean tables every time: the server's, with the pre-murloc copies of anything we changed before
shutil.copytree(r"C:\SoloWoW\server\data\dbc", SRC)
if os.path.isdir(r"C:\SoloWoW\server\data\dbc-before-murloc"):
    for f in os.listdir(r"C:\SoloWoW\server\data\dbc-before-murloc"): shutil.copy(os.path.join(r"C:\SoloWoW\server\data\dbc-before-murloc", f), SRC)
os.makedirs(OUT + r"\DBFilesClient"); os.makedirs(SDBC)
def save(d, name):
    d.save(os.path.join(OUT, "DBFilesClient", name)); shutil.copy(os.path.join(OUT, "DBFilesClient", name), SDBC)

# ---- model: the creature murloc, its skin texture turned into a character skin (type 1) so skin colours apply
os.makedirs(OUT + r"\Character\Murloc\Male"); os.makedirs(OUT + r"\Character\Murloc\Female")
m2 = bytearray(open(MODEL + r"\Murloc.m2", "rb").read())
n, o = struct.unpack_from("<II", m2, 80)
# The character-skin route (texture type 1, composited from CharSections) left the skin blank in game - only the shiny
# "wet" layer showed (neon green, chrome legs), Nov 17. So v1 bakes the murloc skin into the model as a fixed texture
# (type 0 + file name, appended to the end of the file): it looks exactly like the game's murlocs; skin colours wait.
skin_name = (r"Creature\Murloc\%s.blp" % SKINS[0]).encode() + b"\0"
for i in range(n):
    t = struct.unpack_from("<I", m2, o + i * 16)[0]
    if t == 11:
        at = len(m2); m2 += skin_name
        struct.pack_into("<IIII", m2, o + i * 16, 0, 0, len(skin_name), at)
# Sheathed weapons hang from attachment points the creature model doesn't have (they vanished when sheathed, Nov 17):
# add the back and hip sheath points on the back bone the "chest blood back" point uses. The new attachment list and
# lookup go at the end of the file; the header is pointed at them.
SHEATHS = {26: (-0.26, -0.08, 0.95), 27: (-0.26, 0.08, 0.95), 28: (-0.28, 0.0, 0.85),
           30: (-0.26, 0.06, 0.95), 31: (-0.26, -0.06, 0.95), 32: (-0.05, 0.24, 0.60), 33: (-0.05, -0.24, 0.60)}
an, ao = struct.unpack_from("<II", m2, 0xF0)
atts = [bytearray(m2[ao + i * 40: ao + (i + 1) * 40]) for i in range(an)]
back = next(a for a in atts if struct.unpack_from("<I", a)[0] == 16)
have = {struct.unpack_from("<I", a)[0] for a in atts}
for aid, pos in SHEATHS.items():
    if aid in have: continue
    a = bytearray(back); struct.pack_into("<I", a, 0, aid); struct.pack_into("<3f", a, 8, *pos); atts.append(a)
ln, lo = struct.unpack_from("<II", m2, 0xF8)
lookup = [struct.unpack_from("<h", m2, lo + 2 * i)[0] for i in range(ln)]
lookup += [-1] * max(0, 35 - len(lookup))
for i, a in enumerate(atts): lookup[struct.unpack_from("<I", a)[0]] = i
while len(m2) % 16: m2 += b"\0"
at = len(m2); m2 += b"".join(atts); struct.pack_into("<II", m2, 0xF0, len(atts), at)
while len(m2) % 16: m2 += b"\0"
at = len(m2); m2 += struct.pack(f"<{len(lookup)}h", *lookup); struct.pack_into("<II", m2, 0xF8, len(lookup), at)
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
    r = list(cbase); r[0] = cdi.max_id() + 1; r[1] = model_ids[sex]; r[3] = 0; r[4] = struct.unpack("<I", struct.pack("<f", 0.7))[0]   # gnome-sized
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
# The client crashed (Nov 17, twice, in its texture loader) with only one face/hair/facial-hair entry: switching races
# carries the last race's choices over (face 3, hair colour 5...) and a missing entry is a null. So every section gets
# a full 10 x 10 grid, every entry a real texture (the murloc skin - nothing is painted from these while the skin is
# fixed in the model, they only have to exist).
GRID = 10
tex = [cs.add_string(rf"Creature\Murloc\{s}.blp") for s in SKINS]
for sex in (0, 1):
    for c in range(GRID):
        cs.add_row([nid, RACE, sex, 0, tex[c % len(tex)], 0, 0, 17, 0, c]); nid += 1
    for section in (1, 2, 3, 4):
        for v in range(GRID):
            for c in range(GRID):
                cs.add_row([nid, RACE, sex, section, tex[0], tex[0] if section == 1 else 0, 0, 17, v, c]); nid += 1
save(cs, "CharSections.dbc")
hg = Dbc(SRC + r"\CharHairGeosets.dbc")
hid = hg.max_id() + 1
for sex in (0, 1):
    for v in range(GRID): hg.add_row([hid, RACE, sex, v, 0, 0]); hid += 1
save(hg, "CharHairGeosets.dbc")
fh = Dbc(SRC + r"\CharacterFacialHairStyles.dbc")
for sex in (0, 1):
    for v in range(GRID): fh.add_row([RACE, sex, v, 0, 0, 0, 0, 0])
save(fh, "CharacterFacialHairStyles.dbc")

# ---- CharStartOutfit: the Troll class kits (byte 0 of field 1 is the race) - trolls have all six murloc classes;
#      the Undead (the first template) have no shamans or hunters, so those murlocs started with nothing (Nov 17)
so = Dbc(SRC + r"\CharStartOutfit.dbc")
oid = so.max_id() + 1
for i in range(len(so.raw)):
    raw = so.raw[i]                                  # id, race/class/sex/outfit bytes, items[24], displays[24], slots[24]
    if raw[4] == 8 and raw[5] in CLASSES:
        new = bytearray(raw); struct.pack_into("<I", new, 0, oid); oid += 1; new[4] = RACE
        so.add_raw(new)
so.save(os.path.join(SDBC, "CharStartOutfit.dbc"))      # the server keeps the full kits (it hands out the items)
# the client's copy only previews the kit on the creation screen: its clothes would be painted over the murloc's
# skin, so only the weapons are kept in the preview (offsets: displays at byte 104, inventory types at byte 200)
WEAPON_SLOTS = {13, 14, 15, 17, 21, 22, 23, 25, 26}
for raw in so.raw:
    if raw[4] == RACE:
        for k in range(24):
            if struct.unpack_from("<I", raw, 200 + 4 * k)[0] not in WEAPON_SLOTS:
                # the whole slot goes, not just its look: an item with no display crashes the client (Nov 17)
                for at in (8, 104, 200): struct.pack_into("<I", raw, at + 4 * k, 0)
so.save(os.path.join(OUT, "DBFilesClient", "CharStartOutfit.dbc"))

# ---- animations the creature murloc lacks: a jump fell back to standing (an upright hop). Jumping now falls back to
#      the swimming paddle, the same as falling already does - only models with no jump of their own (Nov 17)
ad = Dbc(SRC + r"\AnimationData.dbc")
for i in range(len(ad.raw)):
    row = ad.row(i)
    if row[0] == 38 and row[5] == 0: row[5] = 41; ad.set_row(i, row)
ad.save(os.path.join(OUT, "DBFilesClient", "AnimationData.dbc"))

# ---- NPC looks that used race 15 when it was the unused "Skeleton" slot (25 of them): they would draw as murlocs now;
#      the Undead body carries their baked skeleton textures (Nov 17)
cx = Dbc(SRC + r"\CreatureDisplayInfoExtra.dbc"); moved = 0
for i in range(len(cx.raw)):
    row = cx.row(i)
    if row[1] == RACE: row[1] = 5; cx.set_row(i, row); moved += 1
save(cx, "CreatureDisplayInfoExtra.dbc"); print(f"CreatureDisplayInfoExtra.dbc: {moved} skeleton looks moved to the Undead body")

# ---- what the Horde shares: skills and class abilities the Trolls have together with at least one other Horde race
#      (so shaman and hunter lines come along, troll-only racials don't); reputations as the Forsaken's (Orc + Undead)
def widen(d, fields, name, shared):
    k = 0
    for i in range(len(d.raw)):
        row = d.row(i); hit = False
        for f in fields:
            if row[f] not in (0, 0xFFFFFFFF) and shared(row[f]) and not row[f] & BIT:
                row[f] |= BIT; hit = True
        if hit: d.set_row(i, row); k += 1
    save(d, name); print(f"{name}: {k} rows now include murlocs")
with_trolls = lambda m: bool(m & TROLL) and bool(m & (ORC | UNDEAD | TAUREN))
widen(Dbc(SRC + r"\SkillRaceClassInfo.dbc"), [2], "SkillRaceClassInfo.dbc", with_trolls)
widen(Dbc(SRC + r"\SkillLineAbility.dbc"), [3], "SkillLineAbility.dbc", with_trolls)
widen(Dbc(SRC + r"\Faction.dbc"), [2, 3, 4, 5], "Faction.dbc", lambda m: bool(m & UNDEAD) and bool(m & ORC))

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
lua += ('\n-- SoloWoW: murlocs have no slot in the race-icon sheets, so their button and portrait use the game\'s own murloc\n'
        '-- head icon (Nov 17; was the Undead icon). The coords entries stay so the stock code finds something.\n'
        'RACE_ICON_TCOORDS["MURLOC_MALE"] = RACE_ICON_TCOORDS["SCOURGE_MALE"];\n'
        'RACE_ICON_TCOORDS["MURLOC_FEMALE"] = RACE_ICON_TCOORDS["SCOURGE_FEMALE"];\n'
        'local SOLO_MURLOC_ICON = "Interface\\\\Icons\\\\INV_Misc_Head_Murloc_01";\n'
        'local SOLO_ROUND = "Interface\\\\Glues\\\\CharacterCreate\\\\UI-CharacterCreate-RacesRound";\n'
        'local function SoloMurlocButton()\n'
        '\tlocal n = _G["CharacterCreateRaceButton13NormalTexture"]; local p = _G["CharacterCreateRaceButton13PushedTexture"];\n'
        '\tif ( n ) then n:SetTexture(SOLO_MURLOC_ICON); n:SetTexCoord(0.08, 0.92, 0.08, 0.92); end\n'
        '\tif ( p ) then p:SetTexture(SOLO_MURLOC_ICON); p:SetTexCoord(0.08, 0.92, 0.08, 0.92); end\n'
        'end\n'
        'local function SoloMurlocPortrait()\n'
        '\tlocal _, fileString = GetNameForRace();\n'
        '\tif ( fileString and strupper(fileString) == "MURLOC" ) then\n'
        '\t\tCharacterCreateRaceIcon:SetTexture(SOLO_MURLOC_ICON); CharacterCreateRaceIcon:SetTexCoord(0.08, 0.92, 0.08, 0.92);\n'
        '\telse\n'
        '\t\tCharacterCreateRaceIcon:SetTexture(SOLO_ROUND);\n'
        '\tend\n'
        'end\n'
        'local SoloSetRace = SetCharacterRace;\n'
        'function SetCharacterRace(id) SoloSetRace(id); SoloMurlocPortrait(); end\n'
        'local SoloSetGender = SetCharacterGender;\n'
        'function SetCharacterGender(sex) SoloSetGender(sex); SoloMurlocPortrait(); end\n'
        '\n-- SoloWoW: race buttons laid out by faction to fit their banner, however many races each side has: each\n'
        '-- side\'s buttons share the space between the banner title and the gender buttons, shrinking when needed.\n'
        'local SOLO_RACE_TOP, SOLO_RACE_SPACE, SOLO_RACE_GAP = -50, 310, 10;\n'
        'function SoloArrangeRaceButtons()\n'
        '\tlocal sides = { Alliance = {}, Horde = {} };\n'
        '\tfor i = 1, CharacterCreate.numRaces do\n'
        '\t\tlocal _, faction = GetFactionForRace(i);\n'
        '\t\ttinsert(sides[faction == "Horde" and "Horde" or "Alliance"], i);\n'
        '\tend\n'
        '\tfor side, list in pairs(sides) do\n'
        '\t\tlocal n = #list;\n'
        '\t\tif ( n > 0 ) then\n'
        '\t\t\tlocal h = CharacterCreateRaceButton1:GetHeight();\n'
        '\t\t\tlocal step = min(h + SOLO_RACE_GAP, SOLO_RACE_SPACE / n);\n'
        '\t\t\tlocal scale = min(1, (step - 4) / h);\n'
        '\t\t\tlocal x = side == "Horde" and 50 or -50;\n'
        '\t\t\tfor k, index in ipairs(list) do\n'
        '\t\t\t\tlocal b = _G["CharacterCreateRaceButton"..index];\n'
        '\t\t\t\tb:SetScale(scale);\n'
        '\t\t\t\tb:ClearAllPoints();\n'
        '\t\t\t\tb:SetPoint("TOP", b:GetParent(), "TOP", x / scale, (SOLO_RACE_TOP - (k - 1) * step) / scale);\n'
        '\t\t\tend\n'
        '\t\tend\n'
        '\tend\n'
        'end\n'
        'local SoloEnumerateRaces = CharacterCreateEnumerateRaces;\n'
        'function CharacterCreateEnumerateRaces(...)\n'
        '\tSoloEnumerateRaces(...);\n'
        '\tSoloArrangeRaceButtons();\n'
        '\tSoloMurlocButton();\n'
        'end\n')
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
