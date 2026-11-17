# Graphics: Project Reforged HD models, LuxVelum, DXVK (Nov 17)

Everything here is someone else's work, fetched from its own project at install time (always the latest release) and
never kept in our repo. Our part is the script that fetches and places it, and the step that makes our own client
patches agree with it.

    powershell -ExecutionPolicy Bypass -File scripts\graphics\install-graphics.ps1 [-Extras] [-Interface] [-SD]
    python scripts\graphics\merge-dbc.py

Run both again to update; only what changed is downloaded. Close WoW first.

## What goes where
| Part | From | Into the client |
|---|---|---|
| Wow.exe patches (preset "Reforged": 4 GB, custom GlueXML for our creation screen, DLL loaders, fixes, view distance, 8 lights...) | [St0ny's AIO WoW EXE Patcher](https://github.com/Raz0r1337/St0nys-AIO-WoW-EXE-Patcher), unattended | `Wow.exe` - patched from the original in `Backup\Wow.exe`; the exe it replaced is `Backup\Wow.exe.before-graphics` |
| DXVK (WoW build) | [Raz0r1337/dxvk](https://github.com/Raz0r1337/dxvk) releases, x86 | `d3d9.dll` |
| LuxVelum - fog, light shafts, haze, mist, modern water | [Raz0r1337/LuxVelum](https://github.com/Raz0r1337/Luxvelum) releases | `LuxVelum.dll`, `LuxVelum.ini` (Reforged's preset on a first install), `fogdata.bin`, `waterdata.bin` - loaded by the patched exe, no `version.dll` |
| Shaders for 8 point lights (the exe preset turns them on) | [WoW-lights-patch-shader](https://github.com/Raz0r1337/WoW-lights-patch-shader) | `Data\patch-5.MPQ\` (a folder) |
| HD fonts | [Lexara](https://github.com/Stormhand-dev/Lexara---HD-Font-Renderer-for-WoW-3.3.5) | `Lexara.dll`, `skia.dll` - loaded by the patched exe |
| HD models: characters and NPCs (A), buildings (B), creatures (C), doodads (D), environment (E), gear and weapons (G) | [Project Reforged's downloads page](https://projectreforged.github.io/wotlk/downloads/) | `Data\patch-A..G.MPQ` (~15.5 GB) |
| Optional (`-Extras`): maps and loading screens (M), darker nights (N), spell effects (P), upright orcs (U), sound and music (S) | same | `Data\patch-M/N/P/U/S.MPQ` (~3.3 GB) |
| Optional (`-Interface`): Reforged's interface (I) | same | `Data\patch-I.MPQ` - off by default: our creation screen (Worgen/Goblin) is a glue interface of its own |

B, D and E belong together (all or none) - the script always takes all three.

## Our patches and theirs
WoW reads patch MPQs in order, numbers then letters; the last one wins. Reforged's packs are `patch-A`..`patch-U`;
ours come later: `patch-W` (Worgen and Goblin, from mod-worgoblin) and `patch-Y` (made here). Where `patch-W` carries a
table Reforged's packs carry too (CharSections, hair and facial-hair geosets, helmet geosets, NPC looks, emote sounds...),
`merge-dbc.py` builds the table again: Reforged's version, plus every row `patch-W` added or changed against the stock
game (the new races). Where both changed the same row Reforged's stays and the script says so. The result is
`Data\patch-Y.MPQ`; the server gets the same tables (its originals in `server\data\dbc-before-graphics`), so what the
creation screen offers and what the server accepts agree.

## Undo
- `Backup\Wow.exe.before-graphics` back over `Wow.exe` (or `Backup\Wow.exe` for the original); delete `d3d9.dll`,
  `LuxVelum.*`, `Lexara.dll`, `skia.dll`, `fogdata.bin`, `waterdata.bin`, `Data\patch-5.MPQ\`, `Data\patch-A..U.MPQ`,
  `Data\patch-Y.MPQ`; copy `server\data\dbc-before-graphics\*` back into `server\data\dbc`.
- LuxVelum alone: `Enable=0` in `LuxVelum.ini`, or delete `LuxVelum.dll`.

## Notes
- The patched exe starts in a maximised window (two of the preset's patches); switch in the game's video options.
- With HD everything the 32-bit client needs the 4 GB patch (in the preset); a very long view distance can still run it
  out of memory.
- These mods are fine on our own offline server. Their authors warn that public servers may not allow them.
