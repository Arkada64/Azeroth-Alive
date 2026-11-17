# SoloWoW race tooling: find a safe start spot in an area. Reads creature spawns (tsv from mysql) and the server's
# .map files (area grid), keeps the spawns standing in the area, and ranks friendly/critter spots by distance to the
# nearest hostile creature.   python findspot.py <spawns.tsv> <map> <areaId>
import struct, sys, os, math

MAPS = r"C:\SoloWoW\server\data\maps"
GRID = 533.3333
cache = {}

def area_at(mapid, x, y):
    gx, gy = int(32 - x / GRID), int(32 - y / GRID)
    key = (mapid, gx, gy)
    if key not in cache:
        path = os.path.join(MAPS, f"{mapid:03}{gx:02}{gy:02}.map")
        if not os.path.exists(path): cache[key] = None
        else:
            b = open(path, "rb").read()
            ofs = struct.unpack_from("<I", b, 12)[0]
            flags, garea = struct.unpack_from("<HH", b, ofs + 4)
            cache[key] = garea if flags & 1 else list(struct.unpack_from("<256H", b, ofs + 8))
    a = cache[key]
    if a is None or isinstance(a, int): return a
    fx = (32 - x / GRID - gx) * 16; fy = (32 - y / GRID - gy) * 16
    return a[min(15, int(fx)) * 16 + min(15, int(fy))]

# hostile to a Horde player: the faction template's enemy group has players (1) or the Horde (4)
ft = {}
b = open(r"C:\SoloWoW\server\data\dbc\FactionTemplate.dbc", "rb").read()
_, n, fc, rs, _ = struct.unpack_from("<4s4I", b)
for i in range(n):
    f = struct.unpack_from(f"<{fc}I", b, 20 + i * rs)
    ft[f[0]] = bool(f[5] & 5)
rows = []
for line in open(sys.argv[1], encoding="utf-8-sig"):
    p = line.rstrip("\n").split("\t")
    if p[0] == "guid": continue
    rows.append(dict(guid=int(p[0]), name=p[1], lvl=int(p[2]), hostile=ft.get(int(p[3]), True) and p[4] != "1", critter=p[4] == "1", x=float(p[5]), y=float(p[6]), z=float(p[7])))
mapid, area = int(sys.argv[2]), int(sys.argv[3])
inside = [r for r in rows if area_at(mapid, r["x"], r["y"]) == area]
hostile = [r for r in rows if r["hostile"]]
print(f"{len(inside)} spawns in area {area}; hostile kinds there:",
      sorted({(r['name'], r['lvl']) for r in inside if r['hostile']}))
cands = []
for r in inside:
    if r["hostile"]: continue
    d = min((math.hypot(r["x"] - h["x"], r["y"] - h["y"]) for h in hostile), default=999)
    cands.append((d, r))
for d, r in sorted(cands, key=lambda t: -t[0])[:8]:
    print(f"  {d:6.1f} yd from any hostile: {r['name']} (lvl {r['lvl']}) at {r['x']:.1f} {r['y']:.1f} {r['z']:.1f}")
