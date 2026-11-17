# The one-click installer

Goal: run `Install.bat`, point it at your own 3.3.5a client, and get everything: database, server, launcher, a patched
client. Nothing from Blizzard is ever downloaded or shipped by us - game data is extracted from the player's client.

## Steps
1. **Where** - install folder (default `C:\AzerothAlive`) and the player's client folder.
2. **Check the client** - `Wow.exe` must be 3.3.5a build 12340 (file version 3.3.5.12340). *(working)*
3. **Runtime** - Visual C++ 2015-2022 redistributable (x64); MySQL 8.4 as a private instance in the install folder
   (its own data dir, port 3310, never the system's MySQL).
4. **Server** - download the release zip (worldserver, authserver, DLLs, extractors) from GitHub Releases.
5. **Game data** - extract DBC, maps, vmaps and mmaps from the player's client with the bundled extractors (one-time;
   mmaps take a long while - show progress, allow resume).
6. **Databases** - create the auth/characters/world databases and the local account; the worldserver's own updater
   fills them from AzerothCore's SQL on first start.
7. **Configure** - generate worldserver.conf / authserver.conf from the .dist files with our settings (scripts\configure.ps1),
   paths relative to the install folder; Era preset on by default.
8. **Launcher** - `AzerothAlive.exe` + desktop/start-menu shortcuts.
9. **Patch the client** - with a backup of everything touched *(working)*:
   - realmlist set to 127.0.0.1;
   - Large Address Aware flag on Wow.exe (4 GB instead of 2) - backup `Wow.exe.aa-backup`;
   - later: our patch MPQ (races, looks, Plainsrunning...), DXVK/ReShade as options.
10. **Uninstall** - restores the client from the backups and removes the install folder.

## Developer path
Build from source instead of downloading: clone the core at the pinned commit, apply the patches in `core-patch/` in order
(see core-patch/README.md), then `scripts\build.ps1`.

## To do before it works end to end
- Scripts and launcher still assume `C:\SoloWoW`: make the root configurable (one setting the installer writes).
- Release packaging (a script that zips the server binaries and extractors for GitHub Releases).
- Bundling/fetching the MySQL 8.4 zip and the VC++ redistributable.
## World-DB scripts the installer must apply (in order, after the core's own updates)
- `scripts/era-db.ps1` (Era preset: quest map areas, riding levels)
- `scripts/races/murloc.sql` (after `scripts/races/murloc.py` has built the tables and patch-X.MPQ)
- `scripts/sim_quests.sql` (the 95000-95999 quest pool for townsfolk requests in the quest log)
- mod-worgoblin's own SQL is applied by the core updater (fetch the module into core/modules first)
