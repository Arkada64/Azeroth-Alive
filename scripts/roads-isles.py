# SoloWoW: road nodes where the NPCBots wander graph has none (Nov 18) - the blood elves' and draenei's isles and Dalaran.
#
# The sim's towns, hunting grounds and travel all hang off the wander-node graph (creature_template_npcbot_wander_nodes),
# and Eversong, Ghostlands, Silvermoon, Azuremyst, Bloodmyst, the Exodar and Dalaran have no nodes, so nobody lives there.
# This builds them from the world itself, offline, from what the server already has:
#   - where the zones are: the server's own terrain files (data\maps\*.map carry the area of every 33-yard square, and
#     the ground's height);
#   - where people walk: guards' patrol paths (waypoint_data - they keep to the roads) first, then friendly NPCs, then
#     the creatures of the wilds to fill the gaps;
#   - one node per 110-yard square (45 in the cities), linked to its nearest neighbours where the ground between is
#     walkable (no cliff, no wall of rock: sampled every 6 yards), every island's nodes joined into one graph.
# The wanderers path between nodes with the server's own navigation mesh, as they do on the other nodes. Ids from
# 900000 up; re-running replaces them. Needs SoloWoW's MySQL up; restart the worldserver afterwards.
#   python roads-isles.py [--dry]
import math, os, struct, subprocess, sys
from collections import defaultdict

ROOT = r"C:\SoloWoW"
MAPS = os.path.join(ROOT, "server", "data", "maps")
DBC = os.path.join(ROOT, "server", "data", "dbc")
MYSQL = r"C:\Program Files\MySQL\MySQL Server 8.4\bin\mysql.exe"
BASE = 900000
# zone -> (map, min level, max level, spawn side flag: 2 Alliance, 4 Horde, 0 both, city)
ZONES = {
    3430: (530, 1, 10, 4, False),   # Eversong Woods
    3433: (530, 10, 21, 4, False),  # Ghostlands
    3487: (530, 1, 80, 4, True),    # Silvermoon City
    3524: (530, 1, 10, 2, False),   # Azuremyst Isle
    3525: (530, 10, 21, 2, False),  # Bloodmyst Isle
    3557: (530, 1, 80, 2, True),    # The Exodar
    4395: (571, 70, 80, 0, True),   # Dalaran
}
GRID = 533.33333

def sql(q, db="acore_world"):
    env = dict(os.environ, MYSQL_PWD="acore")
    out = subprocess.run([MYSQL, "--host=127.0.0.1", "--port=3310", "--user=acore", "-B", "-N", db, "-e", q],
                         capture_output=True, text=True, env=env, encoding="utf-8", errors="replace")
    if out.returncode: sys.exit("mysql: " + out.stderr)
    return [l.split("\t") for l in out.stdout.splitlines() if l]

def dbc(name):
    b = open(os.path.join(DBC, name), "rb").read()
    _, n, f, size, ss = struct.unpack_from("<4s4I", b, 0)
    strings = b[20 + n * size:]
    def s(off): return strings[off:strings.index(b"\0", off)].decode("utf-8", "replace")
    return [struct.unpack_from(f"<{f}I", b, 20 + i * size) for i in range(n)], s

rows, string = dbc("AreaTable.dbc")
AREA = {r[0]: (r[1], r[2], string(r[11])) for r in rows}     # id -> (map, parent, name)
def zone_of(area):
    z = area
    while z in AREA and AREA[z][1]: z = AREA[z][1]
    return z

class Tile:
    def __init__(self, path):
        self.area = None; self.garea = 0; self.h = None; self.gh = 0.0
        if not os.path.exists(path): return
        b = open(path, "rb").read()
        hdr = struct.unpack_from("<11I", b, 0)
        ao, ho = hdr[3], hdr[5]
        if ao:
            _, fl, ga = struct.unpack_from("<IHH", b, ao)
            self.garea = ga
            if not fl & 1: self.area = struct.unpack_from("<256H", b, ao + 8)
        if ho:
            _, fl, gh, gmax = struct.unpack_from("<IIff", b, ho)
            self.gh = gh
            if not fl & 1:
                if fl & 2: v = struct.unpack_from("<16641H", b, ho + 16); m = (gmax - gh) / 65535
                elif fl & 4: v = struct.unpack_from("<16641B", b, ho + 16); m = (gmax - gh) / 255
                else: v = struct.unpack_from("<16641f", b, ho + 16); m = None
                self.h = [gh + x * m for x in v] if m is not None else list(v)

_tiles = {}
def tile(mp, x, y):
    gx, gy = int(32 - x / GRID), int(32 - y / GRID)
    k = (mp, gx, gy)
    if k not in _tiles: _tiles[k] = Tile(os.path.join(MAPS, f"{mp:03}{gx:02}{gy:02}.map"))
    return _tiles[k]

def area_at(mp, x, y):
    t = tile(mp, x, y)
    if t.area is None: return t.garea
    fx, fy = 16 * (32 - x / GRID), 16 * (32 - y / GRID)
    return t.area[(int(fx) & 15) * 16 + (int(fy) & 15)]

def place_of(mp, x, y, z):
    # Dalaran floats over Crystalsong: the terrain's area grid has Crystalsong there, so by height
    if mp == 571 and 5500 < x < 6100 and 200 < y < 1100 and z > 550: return 4395, 4395
    a = area_at(mp, x, y)
    return a, zone_of(a)

def height_at(mp, x, y):
    t = tile(mp, x, y)
    if t.h is None: return t.gh if t.gh else None
    fx, fy = 128 * (32 - x / GRID), 128 * (32 - y / GRID)
    xi, yi = int(fx), int(fy); dx, dy = fx - xi, fy - yi
    xi &= 127; yi &= 127
    h = t.h
    a, b2, c, d = h[xi * 129 + yi], h[(xi + 1) * 129 + yi], h[xi * 129 + yi + 1], h[(xi + 1) * 129 + yi + 1]
    return a * (1 - dx) * (1 - dy) + b2 * dx * (1 - dy) + c * (1 - dx) * dy + d * dx * dy

def main():
    dry = "--dry" in sys.argv
    maps = sorted({v[0] for v in ZONES.values()})
    WATCH = {k for k, v in AREA.items() if v[2] in ("Valaar's Berth", "Sunfury Spire", "The Violet Stand", "Azure Watch", "The Bulwark")}
    watch = defaultdict(list)
    pts = []        # (map, x, y, z, zone, area, weight, kind) kind: 3 road, 2 friendly, 1 wild
    print("reading spawns...")
    flyers = {int(r[0]) for r in sql("SELECT CreatureId FROM creature_template_movement WHERE Flight > 0 AND Ground = 0")}
    friendly_flags = 65536 | 128 | 4096 | 8192 | 16 | 2 | 1   # inn, vendor, repair, flight, trainer, quest, gossip
    for r in sql(f"SELECT c.guid, c.id, c.map, c.position_x, c.position_y, c.position_z, t.npcflag, t.faction, t.type "
                 f"FROM creature c JOIN creature_template t ON t.entry = c.id WHERE c.map IN ({','.join(map(str, maps))})"):
        guid, entry, mp = int(r[0]), int(r[1]), int(r[2]); x, y, z = float(r[3]), float(r[4]), float(r[5])
        if entry in flyers or int(r[8]) in (8, 11, 12, 13): continue
        a, zn = place_of(mp, x, y, z)
        if a in WATCH: watch[a].append((x, y))
        if zn not in ZONES or ZONES[zn][0] != mp: continue
        kind = 2 if int(r[6]) & friendly_flags else 1
        pts.append((mp, x, y, z, zn, a, kind))
    for r in sql("SELECT c.map, w.position_x, w.position_y, w.position_z FROM waypoint_data w JOIN creature_addon ca ON ca.path_id = w.id "
                 f"JOIN creature c ON c.guid = ca.guid WHERE c.map IN ({','.join(map(str, maps))})"):
        mp = int(r[0]); x, y, z = float(r[1]), float(r[2]), float(r[3])
        a, zn = place_of(mp, x, y, z)
        if zn in ZONES and ZONES[zn][0] == mp: pts.append((mp, x, y, z, zn, a, 3))
    print(f"{len(pts)} points in the seven zones")

    def on_ground(p):
        h = height_at(p[0], p[1], p[2])
        return h is not None and abs(h - p[3]) < 4.0
    # cells: roads and friendly folk first, the wilds only where nothing else is near
    nodes = []
    for pass_kind in (2, 1):
        cells = defaultdict(list)
        for p in pts:
            if (p[6] >= 2) != (pass_kind == 2): continue
            city = ZONES[p[4]][4]
            if not city and not on_ground(p): continue                # caves, bridges, roofs: not a road
            size = 45.0 if city else 110.0
            cells[(p[0], int(p[1] // size), int(p[2] // size))].append(p)
        for key in sorted(cells):
            c = cells[key]
            wsum = sum(p[6] for p in c)
            cx = sum(p[1] * p[6] for p in c) / wsum; cy = sum(p[2] * p[6] for p in c) / wsum
            best = min(c, key=lambda p: (p[1] - cx) ** 2 + (p[2] - cy) ** 2 - 400 * p[6])
            if pass_kind == 1:
                if len(c) < 3: continue
                if any(n[0] == best[0] and math.hypot(n[1] - best[1], n[2] - best[2]) < 220 for n in nodes): continue
            nodes.append(best)
    print(f"{len(nodes)} nodes")

    def walkable(a, b):
        if a[0] != b[0]: return False
        if ZONES[a[4]][4] or ZONES[b[4]][4]: return True              # the cities: the navigation mesh knows the floors
        d = math.hypot(b[1] - a[1], b[2] - a[2]); steps = max(2, int(d / 6))
        prev = None
        for i in range(steps + 1):
            t = i / steps; x = a[1] + (b[1] - a[1]) * t; y = a[2] + (b[2] - a[2]) * t
            h = height_at(a[0], x, y)
            if h is None: return False
            line = a[3] + (b[3] - a[3]) * t
            if h - line > 18 or line - h > 30: return False             # a ridge in the way, or a drop
            if prev is not None and abs(h - prev) > 6.5: return False   # steeper than ~47 degrees
            prev = h
        return True

    links = defaultdict(set)
    for i, a in enumerate(nodes):
        reach = 70 if ZONES[a[4]][4] else 240
        near = sorted((math.hypot(b[1] - a[1], b[2] - a[2]), j) for j, b in enumerate(nodes) if j != i and b[0] == a[0])
        dirs = []
        for d, j in near:
            if d > reach or len(dirs) >= 4: break
            b = nodes[j]
            ang = math.atan2(b[2] - a[2], b[1] - a[1])
            if any(abs((ang - o + math.pi) % (2 * math.pi) - math.pi) < math.radians(35) for o in dirs): continue
            if not walkable(a, b): continue
            dirs.append(ang); links[i].add(j); links[j].add(i)
    # one graph per island: join the pieces by their closest pair
    while True:
        comp = {}
        for s in range(len(nodes)):
            if s in comp: continue
            stack = [s]; comp[s] = s
            while stack:
                u = stack.pop()
                for v in links[u]:
                    if v not in comp: comp[v] = s; stack.append(v)
        roots = sorted(set(comp.values()))
        best = None
        for r in roots:
            members = [i for i in comp if comp[i] == r]
            for i in members:
                for j, b in enumerate(nodes):
                    if comp[j] == r or b[0] != nodes[i][0]: continue
                    d = math.hypot(b[1] - nodes[i][1], b[2] - nodes[i][2])
                    if d < 600 and (best is None or d < best[0]): best = (d, i, j)
            if best: break
        if not best: break
        links[best[1]].add(best[2]); links[best[2]].add(best[1])
    pieces = len(set(comp.values()))
    print(f"{sum(len(v) for v in links.values()) // 2} links, {pieces} separate graphs (Quel'thalas, the draenei isles, Dalaran: 3 expected)")

    out = [f"DELETE FROM creature_template_npcbot_wander_nodes WHERE id >= {BASE};"]
    for i, n in enumerate(nodes):
        mp, x, y, z, zn, a, kind = n
        _, lo, hi, side, city = ZONES[zn]
        spawn = kind == 2                                              # friendly folk about: wanderers may start here
        flags = (1 | side) if spawn else 0
        name = AREA.get(a, (0, 0, "the wilds"))[2].replace("'", "''") or AREA[zn][2].replace("'", "''")
        lk = " ".join(f"{BASE + j}:0" for j in sorted(links[i])) + " "
        out.append(f"INSERT INTO creature_template_npcbot_wander_nodes (id, name, mapid, zoneid, areaid, minlevel, maxlevel, flags, x, y, z, o, links) "
                   f"VALUES ({BASE + i}, '{name}', {mp}, {zn}, {a}, {lo}, {hi}, {flags}, {x:.2f}, {y:.2f}, {z:.2f}, 0, '{lk}');")
    sqlfile = os.path.join(ROOT, "build", "roads-isles.sql")
    os.makedirs(os.path.dirname(sqlfile), exist_ok=True)
    open(sqlfile, "w", encoding="utf-8").write("\n".join(out) + "\n")
    by = defaultdict(int)
    for n in nodes: by[n[4]] += 1
    print("by zone:", {AREA[z][2]: c for z, c in by.items()})
    # where the crossings stand (for the sim's routes)
    for a, ps in watch.items():
        print(f"  {AREA[a][2]} ({a}): {sum(p[0] for p in ps)/len(ps):.0f}, {sum(p[1] for p in ps)/len(ps):.0f} ({len(ps)} spawns)")
    if dry: print("dry run:", sqlfile); return
    env = dict(os.environ, MYSQL_PWD="acore")
    with open(sqlfile, "rb") as f:
        r = subprocess.run([MYSQL, "--host=127.0.0.1", "--port=3310", "--user=acore", "acore_world"], stdin=f, capture_output=True, env=env)
    if r.returncode: sys.exit("mysql: " + r.stderr.decode())
    print(f"{len(nodes)} road nodes written (ids {BASE}-{BASE + len(nodes) - 1}); restart the worldserver")

if __name__ == "__main__":
    main()
