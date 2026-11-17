# SoloWoW: pack a folder into a client patch MPQ with StormLib (reference\StormLib, built from source).
#   python make-mpq.py <folder> <out.MPQ>
# Every file under <folder> goes in under its relative path (backslashes), zlib-compressed, with a listfile.
import ctypes, os, sys
from ctypes import wintypes

storm = ctypes.WinDLL(r"C:\SoloWoW\reference\StormLib\build\Release\StormLib.dll")
MPQ_CREATE_LISTFILE, MPQ_CREATE_ATTRIBUTES, MPQ_CREATE_ARCHIVE_V2 = 0x00100000, 0x00200000, 0x01000000
MPQ_FILE_COMPRESS, MPQ_FILE_REPLACEEXISTING = 0x00000200, 0x80000000
ZLIB, NEXT_SAME = 0x02, 0xFFFFFFFF

src, out = sys.argv[1], sys.argv[2]
files = []
for root, _, names in os.walk(src):
    for n in names:
        full = os.path.join(root, n)
        files.append((full, os.path.relpath(full, src).replace("/", "\\")))
if os.path.exists(out): os.remove(out)
h = ctypes.c_void_p()
count = max(64, 1 << (len(files) * 2).bit_length())
if not storm.SFileCreateArchive(out.encode("mbcs"), MPQ_CREATE_LISTFILE | MPQ_CREATE_ATTRIBUTES | MPQ_CREATE_ARCHIVE_V2, count, ctypes.byref(h)):
    sys.exit(f"cannot create {out}: error {ctypes.windll.kernel32.GetLastError()}")
bad = 0
for full, name in files:
    if not storm.SFileAddFileEx(h, full.encode("mbcs"), name.encode("mbcs"), MPQ_FILE_COMPRESS | MPQ_FILE_REPLACEEXISTING, ZLIB, NEXT_SAME):
        bad += 1; print("failed:", name)
storm.SFileCloseArchive(h)
print(f"{out}: {len(files) - bad} files packed, {bad} failed, {os.path.getsize(out) // (1024 * 1024)} MB")
