# SoloWoW: fit our own client patches to Project Reforged's HD models (Nov 17).
#   python scripts\graphics\merge-dbc.py [client folder]
# WoW reads its patches in order and the last one wins. Our patch-W (Worgen/Goblin, from mod-worgoblin) carries client
# tables that Reforged's packs carry too (CharSections, hair, facial hair, helmet geosets, NPC looks...); loaded after
# theirs, ours would hide theirs and the HD models would be drawn with standard-definition tables. So for every table
# both carry: take Reforged's (the newest of their packs), then add what ours added or changed against the stock game
# (new races' rows: Worgen, Goblin...). Where both changed the same row, Reforged's stays (it's the look) - those are
# listed. The merged tables go into Data\patch-Y.MPQ (loads after both) and, for the server's copy, server\data\dbc
# (the old ones kept once in server\data\dbc-before-graphics). Our patch-W is left as it is. Run again after updating
# Reforged's packs (install-graphics.ps1 does). Only reads the client's files - never ships any of them.
import ctypes, os, shutil, struct, sys
from ctypes import wintypes
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "races"))
from dbcfile import Dbc

ROOT = r"C:\SoloWoW"
CLIENT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "WOTLK Client")
DATA = os.path.join(CLIENT, "Data")
OURS = ["patch-W.MPQ"]                                  # our client patches, in load order
OUT_NAME = "patch-Y.MPQ"
STAGE = os.path.join(ROOT, "build", "graphics", "patch-Y")
SERVER_DBC = os.path.join(ROOT, "server", "data", "dbc")
STORM = os.path.join(ROOT, "reference", "StormLib", "build", "Release", "StormLib.dll")
# string fields per table (3.3.5a layouts); a table not listed has none. Key: field 0 (the id), except where noted.
STRINGS = {"CharSections.dbc": [4, 5, 6], "CreatureDisplayInfoExtra.dbc": [20], "CreatureDisplayInfo.dbc": [6, 7, 8, 9],
           "CreatureModelData.dbc": [2], "ItemDisplayInfo.dbc": [1, 2, 3, 4, 5, 6, 15, 16, 17, 18, 19, 20, 21, 22],
           "ChrRaces.dbc": None, "Spell.dbc": None, "Achievement.dbc": None, "Achievement_Criteria.dbc": None,
           "SoundEntries.dbc": None, "NameGen.dbc": [1], "SkillLine.dbc": None, "TalentTab.dbc": None, "SpellIcon.dbc": [1],
           "VocalUISounds.dbc": None, "CreatureSoundData.dbc": None, "SummonProperties.dbc": [], "Faction.dbc": None}
KEYS = {"CharacterFacialHairStyles.dbc": (0, 1, 2)}       # race, sex, variation - no id column
SERVER_USES = {"CharSections.dbc", "CharStartOutfit.dbc", "ChrRaces.dbc", "CreatureDisplayInfo.dbc", "CreatureDisplayInfoExtra.dbc",
               "CreatureModelData.dbc", "Faction.dbc", "SkillLineAbility.dbc", "SkillRaceClassInfo.dbc", "Item.dbc", "Spell.dbc",
               "Achievement.dbc", "Achievement_Criteria.dbc", "SkillLine.dbc", "TalentTab.dbc", "SummonProperties.dbc", "EmotesTextSound.dbc",
               "CharHairGeosets.dbc", "CharacterFacialHairStyles.dbc", "BarberShopStyle.dbc", "ItemDisplayInfo.dbc", "SoundEntries.dbc"}

storm = ctypes.WinDLL(STORM)
H = ctypes.c_void_p

def open_mpq(path):
    h = H()
    if storm.SFileOpenArchive(ctypes.c_wchar_p(path), 0, 0x100, ctypes.byref(h)): return h
    if storm.SFileOpenArchive(ctypes.c_char_p(path.encode("mbcs")), 0, 0x100, ctypes.byref(h)): return h   # an ANSI StormLib build
    return None

def read(h, name):
    f = H()
    if not storm.SFileHasFile(h, name.encode()): return None
    if not storm.SFileOpenFileEx(h, name.encode(), 0, ctypes.byref(f)): return None
    hi = wintypes.DWORD(0)
    size = storm.SFileGetFileSize(f, ctypes.byref(hi))
    buf = ctypes.create_string_buffer(max(size, 1)); got = wintypes.DWORD(0)
    storm.SFileReadFile(f, buf, size, ctypes.byref(got), None)
    storm.SFileCloseFile(f)
    return buf.raw[:got.value] if got.value else None       # (SFileHasFile says yes too readily: an empty read is no file)

def load_order():
    """The client's MPQs in the order it reads them (later wins)."""
    names = {}
    for root, _, files in os.walk(DATA):
        for f in files:
            if f.lower().endswith(".mpq") and os.path.isfile(os.path.join(root, f)):
                names[os.path.relpath(os.path.join(root, f), DATA)] = os.path.join(root, f)
    def rank(rel):
        b = os.path.basename(rel).lower(); loc = os.path.dirname(rel) != ""
        base = {"common.mpq": 0, "common-2.mpq": 1, "expansion.mpq": 2, "lichking.mpq": 3}
        if b in base: return (0, base[b], 0)
        if b.startswith("locale-") or b.startswith("expansion-locale") or b.startswith("lichking-locale") or b.startswith("speech-") or b.startswith("expansion-speech") or b.startswith("lichking-speech"):
            return (1, 0, 0)
        stem = b[:-4]
        if stem == "patch" or stem.startswith("patch-"):
            suffix = stem.split("-")[-1] if "-" in stem else ""
            if loc: suffix = stem.split("-")[-1] if stem.count("-") >= 2 else ""
            order = 0 if suffix in ("", "enus") else (int(suffix) if suffix.isdigit() else 100 + ord(suffix[0].upper()))
            return (2, order, 1 if loc else 0)
        return (1, 1, 0)
    return [names[k] for k in sorted(names, key=rank)]

def is_ours(p): return os.path.basename(p) in OURS or os.path.basename(p) == OUT_NAME
def is_reforged(p):
    b = os.path.basename(p)
    return os.path.dirname(os.path.relpath(p, DATA)) == "" and len(b) == 11 and b.lower().startswith("patch-") and b[6].isalpha() and not is_ours(p)

def tmpdbc(blob, tag):
    p = os.path.join(STAGE, "..", "tmp-" + tag + ".dbc"); open(p, "wb").write(blob); return Dbc(p)

def decode(d, row, strs):
    return tuple(d.s(v) if i in strs else v for i, v in enumerate(row))

def merge(name, stock_b, ref_b, our_b, log):
    st, rf, ou = tmpdbc(stock_b, "s"), tmpdbc(ref_b, "r"), tmpdbc(our_b, "o")
    strs = STRINGS.get(name, [])
    if strs is None: raise RuntimeError("needs a string map")
    if not (st.fields == rf.fields == ou.fields): raise RuntimeError(f"field counts differ: stock {st.fields}, reforged {rf.fields}, ours {ou.fields}")
    key = KEYS.get(name, (0,))
    kf = lambda r: tuple(r[i] for i in key)
    stock = {kf(r): decode(st, r, strs) for r in st.rows()}
    refrows = {kf(r): i for i, r in enumerate(rf.rows())}
    added = changed = kept = 0
    for r in ou.rows():
        k = kf(r); mine = decode(ou, r, strs)
        if stock.get(k) == mine: continue                       # ours didn't touch it
        if k in refrows:
            theirs = decode(rf, rf.row(refrows[k]), strs)
            if theirs == mine: continue
            if k in stock and theirs != stock[k]:               # both changed it: their look stays
                kept += 1; log.append(f"   {name} {k}: both changed - Reforged's kept"); continue
        out = list(r)
        for i in strs: out[i] = rf.add_string(ou.s(r[i])) if r[i] else 0
        if k in refrows: rf.set_row(refrows[k], out); changed += 1
        else: rf.add_row(out); refrows[k] = len(rf.raw) - 1; added += 1
    os.makedirs(os.path.join(STAGE, "DBFilesClient"), exist_ok=True)
    rf.save(os.path.join(STAGE, "DBFilesClient", name))
    return added, changed, kept

# Reforged models that don't show up right in our client: the stock model (from the player's own client) goes back in
# patch-Y, which loads after Reforged's patches. Each entry is a model; its .skin files come with it. (Nov 17 night:
# the HD signposts - the arrows show, the posts don't, so the signs hang in mid-air or in a tree.)
REVERT_MODELS = [
    r"World\GENERIC\HUMAN\PASSIVE DOODADS\SignPosts\HumanSignPost03.m2",
    r"World\GENERIC\HUMAN\PASSIVE DOODADS\SignPosts\HumanSignPostPointer03.m2",
    r"World\GENERIC\HUMAN\PASSIVE DOODADS\SignPosts\WoodSignPostNice01.m2",
    r"World\GENERIC\HUMAN\PASSIVE DOODADS\SignPosts\WoodSignPointerNice01.m2",
    r"World\GENERIC\HUMAN\PASSIVE DOODADS\SignPosts\WoodSignPostWorn01.m2",
    r"World\GENERIC\HUMAN\PASSIVE DOODADS\SignPosts\WoodSignPointerWorn01.m2",
    r"World\GENERIC\ORC\PASSIVE DOODADS\SignPosts\OrcSignPost03.m2",
    r"World\GENERIC\ORC\PASSIVE DOODADS\SignPosts\OrcSignPostPointer03.m2",
]

def revert_models(handles):
    """Put the stock copy of each listed model (and its skins) into the stage, where Reforged replaced it."""
    n = 0
    for m in REVERT_MODELS:
        files = [m] + [m[:-3] + f"{i:02d}.skin" for i in range(4)]
        if not any(is_reforged(p) and read(h, m) is not None for p, h in handles): continue
        for f in files:
            stock = None
            for p, h in handles:
                if is_reforged(p) or is_ours(p): continue
                b = read(h, f)
                if b is not None: stock = b
            if stock is None: continue
            out = os.path.join(STAGE, f); os.makedirs(os.path.dirname(out), exist_ok=True)
            with open(out, "wb") as fh: fh.write(stock)
            n += 1
    print(f"  {n} stock model files put back over Reforged's ({len(REVERT_MODELS)} models listed)")
    return n

def main():
    order = load_order()
    handles = [(p, open_mpq(p)) for p in order]
    handles = [(p, h) for p, h in handles if h]
    ours = [p for p, _ in handles if os.path.basename(p) in OURS]
    if not ours: print("no patch of ours in the client - nothing to fit"); return
    shutil.rmtree(STAGE, ignore_errors=True)
    os.makedirs(STAGE, exist_ok=True)
    # every table one of ours carries
    tables = set()
    for p, h in handles:
        if os.path.basename(p) not in OURS: continue
        for name in STRINGS.keys() | set(KEYS) | {"HelmetGeosetVisData.dbc", "EmotesTextSound.dbc", "CharHairGeosets.dbc", "CharHairTextures.dbc",
                                                    "CharStartOutfit.dbc", "AnimationData.dbc", "BarberShopStyle.dbc", "Item.dbc", "SkillLineAbility.dbc",
                                                    "SkillRaceClassInfo.dbc", "CharBaseInfo.dbc", "CharacterFacialHairStyles.dbc"}:
            if read(h, "DBFilesClient\\" + name) is not None: tables.add(name)
    log, done = [], []
    for name in sorted(tables):
        path = "DBFilesClient\\" + name
        stock = ref = mine = None; refsrc = None
        for p, h in handles:
            b = read(h, path)
            if b is None: continue
            if is_ours(p): mine = b if os.path.basename(p) in OURS else mine
            elif is_reforged(p): ref, refsrc = b, os.path.basename(p)
            else: stock = b
        if ref is None: continue                                  # Reforged doesn't carry it: ours stands as it is
        if stock is None: print(f"  {name}: no stock copy - skipped"); continue
        try:
            a, c, k = merge(name, stock, ref, mine, log)
            done.append(name)
            print(f"  {name}: Reforged's ({refsrc}) + ours: {a} rows added, {c} changed, {k} left to Reforged")
        except Exception as ex:
            print(f"  {name}: NOT merged ({ex}) - ours will hide theirs for this table")
    for t in log[:40]: print(t)
    reverted = revert_models(handles)
    for _, h in handles: storm.SFileCloseArchive(h)
    for f in os.listdir(os.path.join(STAGE, "..")):
        if f.startswith("tmp-"): os.remove(os.path.join(STAGE, "..", f))
    dest = os.path.join(DATA, OUT_NAME)
    if not done and not reverted:
        if os.path.exists(dest): os.remove(dest)
        print("nothing to merge - no patch-Y"); return
    if os.path.exists(dest): os.remove(dest)
    os.system(f'python "{os.path.join(ROOT, "scripts", "make-mpq.py")}" "{STAGE}" "{dest}"')
    # the server's copies of the tables it reads, so creation and looks agree with the client
    before = SERVER_DBC + "-before-graphics"
    os.makedirs(before, exist_ok=True)
    for name in done:
        if name not in SERVER_USES: continue
        src = os.path.join(SERVER_DBC, name)
        if os.path.exists(src) and not os.path.exists(os.path.join(before, name)): shutil.copy(src, before)
        shutil.copy(os.path.join(STAGE, "DBFilesClient", name), src)
    print(f"{dest}: {len(done)} tables fitted to Reforged's models; server tables updated (originals in {before})")

if __name__ == "__main__":
    main()
