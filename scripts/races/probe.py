from dbcfile import Dbc
D = r"C:\SoloWoW\server\data\dbc"
m = Dbc(D + r"\CreatureModelData.dbc")
mids = {}
for r in m.rows():
    p = m.s(r[2]).lower()
    if p.startswith("creature\\murloc\\") or "murloccostume" in p: mids[r[0]] = m.s(r[2])
print("modeldata:", mids)
c = Dbc(D + r"\CreatureDisplayInfo.dbc")
for r in c.rows():
    if r[1] in mids: print("cdi", r[0], "model", r[1], "sound", r[2], "extra", r[3], "scale", r[4], "tex", [c.s(x) for x in r[6:9]])
mrow = [r for r in m.rows() if r[0] in mids][:1]
print("modeldata row", mrow)
