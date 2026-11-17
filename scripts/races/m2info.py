# SoloWoW race tooling: what a WotLK (v264) M2 model holds - textures (type, file), animation ids, attachments, geosets.
#   python m2info.py <model.m2> [<model00.skin>]
import struct, sys

def arr(b, o): return struct.unpack_from("<II", b, o)   # (count, offset)

b = open(sys.argv[1], "rb").read()
assert b[:4] == b"MD20", "not an M2"
print("version", struct.unpack_from("<I", b, 4)[0])
n, o = arr(b, 8); print("name", b[o:o + n].rstrip(b"\0").decode())
n, o = arr(b, 28)
anims = sorted({struct.unpack_from("<H", b, o + i * 64)[0] for i in range(n)})
print("animations", n, "ids", anims)
n, o = arr(b, 80)
for i in range(n):
    t, fl, fn, fo = struct.unpack_from("<IIII", b, o + i * 16)
    print(f"texture {i}: type {t} flags {fl} file '{b[fo:fo + fn].rstrip(bytes(1)).decode() if fn else ''}'")
n, o = arr(b, 0xF0)   # attachments (WotLK header offset 0xF0)
print("attachments", n, [struct.unpack_from("<I", b, o + i * 40)[0] for i in range(n)])
if len(sys.argv) > 2:
    s = open(sys.argv[2], "rb").read()
    n, o = arr(s, 28)   # SKIN: magic, indices, triangles, props, submeshes(28)
    print("geosets (submesh ids)", sorted({struct.unpack_from("<H", s, o + i * 48)[0] for i in range(n)}))
