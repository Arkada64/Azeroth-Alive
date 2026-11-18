# SoloWoW: room in the client for generated items (Nov 18) - "truly generated" loot.
#
# A 3.3.5 client shows an item's icon and its model on a character only if the item's id is in its Item.dbc. Generated
# items get ids from a reserved range, 950000 up: this script adds those rows - three for every look the game's weapons
# and armour have (class, subclass, slot, model) - to the client (its own patch, patch-X.MPQ, built from the client's
# own Item.dbc, so nothing is shipped) and to the server's copy. The server then fills the ids it hands out with names
# and stats (SoloSim/solosim_genitems.cpp). Run again after anything else changes Item.dbc (patch-W, a client update).
#   python gen-items.py [client folder]
import importlib.util, os, shutil, struct, sys

ROOT = r"C:\SoloWoW"
spec = importlib.util.spec_from_file_location("mergedbc", os.path.join(ROOT, "scripts", "graphics", "merge-dbc.py"))
m = importlib.util.module_from_spec(spec)
argv = sys.argv; sys.argv = [argv[0]] + argv[1:2]
spec.loader.exec_module(m)
sys.argv = argv

GEN_BASE, PER_LOOK, MAX_ROWS = 950000, 3, 90000
OUT = os.path.join(m.DATA, "patch-X.MPQ")
STAGE = os.path.join(ROOT, "build", "graphics", "patch-X")
SERVER_ITEM = os.path.join(m.SERVER_DBC, "Item.dbc")
EQUIP = set(range(1, 29)) - {4, 18, 19, 24, 27}          # not shirts, bags, tabards, ammo, quivers

def main():
    best = None
    for p in m.load_order():
        if os.path.basename(p).lower() == "patch-x.mpq": continue
        h = m.open_mpq(p)
        if not h: continue
        b = m.read(h, "DBFilesClient\\Item.dbc")
        if b: best = (p, b)
        m.storm.SFileCloseArchive(h)
    if not best: sys.exit("no Item.dbc in the client?")
    src, blob = best
    magic, n, fields, size, strsize = struct.unpack_from("<4s4I", blob, 0)
    assert magic == b"WDBC" and fields == 8 and size == 32, "unexpected Item.dbc layout"
    rows = [struct.unpack_from("<8i", blob, 20 + i * size) for i in range(n)]
    rows = [r for r in rows if r[0] < GEN_BASE]                # a fresh pool each time
    looks = sorted({(r[1], r[2], r[6], r[5], r[3], r[4], r[7]) for r in rows if r[1] in (2, 4) and r[5] > 0 and r[6] in EQUIP})
    pool = []
    for (cls, sub, inv, disp, sound, mat, sheath) in looks:
        for k in range(PER_LOOK):
            if len(pool) >= MAX_ROWS: break
            pool.append((GEN_BASE + len(pool), cls, sub, sound, mat, disp, inv, sheath))
    allrows = rows + pool
    out = bytearray(struct.pack("<4s4I", b"WDBC", len(allrows), 8, 32, strsize))
    for r in allrows: out += struct.pack("<8i", *r)
    out += blob[20 + n * size:]                                # the (empty) string block as it was
    # the client: its own patch
    shutil.rmtree(STAGE, ignore_errors=True)
    os.makedirs(os.path.join(STAGE, "DBFilesClient"))
    open(os.path.join(STAGE, "DBFilesClient", "Item.dbc"), "wb").write(out)
    os.system(f'python "{os.path.join(ROOT, "scripts", "make-mpq.py")}" "{STAGE}" "{OUT}"')
    # the server: the same table (the original kept once)
    before = m.SERVER_DBC + "-before-genitems"
    os.makedirs(before, exist_ok=True)
    if os.path.exists(SERVER_ITEM) and not os.path.exists(os.path.join(before, "Item.dbc")): shutil.copy(SERVER_ITEM, before)
    open(SERVER_ITEM, "wb").write(out)
    print(f"Item.dbc from {os.path.basename(src)}: {len(rows)} items, {len(looks)} looks -> {len(pool)} generated-item ids "
          f"({GEN_BASE}-{GEN_BASE + len(pool) - 1}) in {OUT} and the server's copy")

if __name__ == "__main__":
    main()
