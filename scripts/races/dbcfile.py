# SoloWoW race tooling: read, edit and write WDBC files (all-uint32 fields, or a raw byte layout for odd tables).
import struct

class Dbc:
    def __init__(self, path):
        b = open(path, "rb").read()
        magic, self.n, self.fields, self.recsize, ss = struct.unpack_from("<4s4I", b)
        assert magic == b"WDBC", path
        self.raw = [bytearray(b[20 + i * self.recsize: 20 + (i + 1) * self.recsize]) for i in range(self.n)]
        self.strings = bytearray(b[20 + self.n * self.recsize:])
        self.uint = self.recsize == self.fields * 4
        self._index = None

    # records as lists of uint32 (only for all-4-byte tables)
    def row(self, i): return list(struct.unpack_from(f"<{self.fields}I", self.raw[i]))
    def rows(self): return [self.row(i) for i in range(len(self.raw))]
    def set_row(self, i, vals): self.raw[i] = bytearray(struct.pack(f"<{self.fields}I", *[v & 0xFFFFFFFF for v in vals]))
    def add_row(self, vals): self.raw.append(bytearray(struct.pack(f"<{self.fields}I", *[v & 0xFFFFFFFF for v in vals]))); return len(self.raw) - 1
    def add_raw(self, data): assert len(data) == self.recsize; self.raw.append(bytearray(data)); return len(self.raw) - 1
    def find(self, id_):
        for i in range(len(self.raw)):
            if struct.unpack_from("<I", self.raw[i])[0] == id_: return i
        return -1
    def max_id(self): return max(struct.unpack_from("<I", r)[0] for r in self.raw)

    def s(self, off):
        e = self.strings.find(b"\0", off)
        return self.strings[off:e].decode("utf-8", "replace")
    def add_string(self, text):
        if not text: return 0
        enc = text.encode("utf-8") + b"\0"
        at = self.strings.find(b"\0" + enc)
        if at >= 0: return at + 1
        off = len(self.strings); self.strings += enc; return off

    def save(self, path):
        with open(path, "wb") as f:
            f.write(struct.pack("<4s4I", b"WDBC", len(self.raw), self.fields, self.recsize, len(self.strings)))
            for r in self.raw: f.write(r)
            f.write(self.strings)
