# SoloWoW - Milestone 1 results (2026-11-15)

Spike goal: build the NPCBots fork on Windows, turn on wandering bots, measure start time and RAM.
Status: **done, except an in-client login check** (bots verified from the server side).

## What exists now
| Path | What |
|---|---|
| `source\` | trickerer/AzerothCore-wotlk-with-NPCBots, branch `npcbots_3.3.5`, commit `c89fa10` (2026-11-14) |
| `build\bin\Release\` | worldserver.exe (22.5 MB), authserver.exe, libmysql/libssl/libcrypto/legacy DLLs, `configs\` |
| `mysql\` | private MySQL 8.4 instance: 127.0.0.1:3310, 256 MB pool, no binlog, no perf schema, skip-name-resolve |
| `scripts\build.ps1` | AzerothManager's recipe: VS 2022, Boost 1.81, OpenSSL 3.6.5, RAM-capped /MP jobs, Release, no tools |
| `scripts\configure.ps1` | writes confs. `-Bots N -Continent EK|Kalimdor|Outland|Northrend|All` |
| `scripts\common.ps1` | Sql, SRP6 account (copied from AzerothManager), SOAP helper |
| `scripts\boot-measure.ps1` / `sample.ps1` | start hidden + time to world port, RAM sampling |
| `scripts\stop.ps1` | `.saveall` + `.server shutdown` via SOAP, kills auth; `-StopMysql` also stops MySQL |
| `logs\measure-*.txt` | raw measurements |

Map data is reused from `C:\SoloWoW\server\data` (DataDir). Local account `SOLO` / `solo`, GM 3 (also SOAP user, port 7879).
DBs: `acore_auth`, `acore_world`, `acore_characters` (no per-save split yet).

## Numbers (Release, 16 threads, 32 GB, warm boot)
| Run | Ready (world port open) | worldserver RAM (settled) | + mysqld | Total |
|---|---|---|---|---|
| First boot (DB import) | 654 s | - | - | one-time |
| 0 bots, stock conf | 23.1 s | 1.7 GB at 60 s, **still climbing** | 0.57 GB | 2.3 GB+ |
| 50 bots, stock conf, all 4 continents | 24.1 s | **3.95 GB** at ~8 min | 0.58 GB | **4.5 GB** |
| 0 bots, lean conf | 21.9 s | **0.91 GB flat** | 0.58 GB | 1.5 GB |
| 50 bots, lean, Eastern Kingdoms only | 24.1 s | 1.85 GB at 6 min | 0.59 GB | **2.45 GB** |
| 150 bots, lean, Eastern Kingdoms only | 25.6 s | 1.96 GB at 7 min | 0.60 GB | **2.6 GB** |

"Ready" = `World Initialized In 0 Minutes 21 Seconds` - the core itself takes ~21 s; client startup to the login screen runs in parallel.
Tick times stayed healthy in every run (mean 10-11 ms, p99 16-34 ms). 150 bots used ~0.2 of one core.

## Lessons (feed these into the launcher)
1. **Lean conf is the big lever.** `IsContinentTransport.Enabled=0`, `Wintergrasp.Enable=0`, `Warden.Enabled=0`
   turned a bots-off server from "1.7 GB and climbing, busy CPU" into "0.91 GB flat, idle". Transports are probably
   the main one (they drag grids along their routes) - but players will want boats/zeppelins, so the next step is
   testing them individually and finding out if transports can stay on with lower cost.
2. **Bots only on the player's continent.** All 4 continents: ~4 GB. One continent: ~1.9 GB. RAM tracks the
   continent's loaded grids, not the bot count (50 -> 150 bots cost only ~100 MB). So the launcher should set
   `Continents.Maps` to the save's current continent at boot (read the character's map from its characters DB).
3. **Maps and level spread must match**, or world crashes on boot: assert `botdatamgr.cpp:411`
   (`ASSERT(!level_nodes.empty())`) - no wander nodes for a requested bracket. Default `Levels` asks for 30% at
   60-79, which EK/Kalimdor don't have. configure.ps1 sets both together.
4. **No interactive prompts:** set env `AC_DISABLE_INTERACTIVE=1` for auth and world, or they block forever on
   "Database does not exist. Create it?" on a hidden console.
5. **DB setup is its own first-run step.** Starting auth and world together on empty DBs races: both populate
   `acore_auth`, auth loses and exits. Launcher: create/populate DBs first (or start world alone first time).
6. **Bot ceiling:** 368 bot templates in creature_template, so 368 is the hard max for the bot count.
7. Upstream bug: `.npcbot list spawned stats` has an inverted check and prints "No bots found!" when bots exist.
   Use `.npcbot list spawned` / `.npcbot list spawned zone <name>` instead.
8. The fork's custom Warcraft III classes (Blademaster, Crypt Lord, Sea Witch, Dreadlord, ...) wander by default.
   `NpcBot.WanderingBots.Classes.<Name>.Enable = 0` turns them off if we want a classic-only feel.
9. PowerShell gotchas on this PC: script execution is disabled (use `-ExecutionPolicy Bypass`), and
   `-h127.0.0.1` style args get mangled - use `--host=127.0.0.1`.

## Still to do for milestone 1
- Log in with the client (`D:\WOTLK Client`, realmlist already 127.0.0.1 from AzerothManager; account SOLO/solo)
  and eyeball bots in Elwynn (and Durotar: run `configure.ps1 -Continent Kalimdor`). Don't run AzerothManager's
  server at the same time - both use auth port 3724 and world port 8085.

## Suggested next steps (milestone 2)
- Test transports / Wintergrasp individually to see which ones actually cost RAM.
- Set the continent per save, XP/drop rates, death knight decision, WC3 classes on/off.
- Try `MapUpdate.Threads` > 1 and measure start-time cuts (startup profile from world.out timings).
