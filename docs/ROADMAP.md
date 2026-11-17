# SoloWoW roadmap (living notes)

> The overall design (vision, settled decisions, architecture, NPCBots hook points) is in **DESIGN.md**.
> Scope has grown from "host WoW locally, optimised" to "play WoW locally, all bundled, in a world alive without players".

## Folder layout (moved to C:\SoloWoW on 2026-11-15)
```
C:\SoloWoW\
  SoloWoW.exe        the launcher (play, settings)
  SoloWoW Manager.bat  server manager: start/stop/restart database, login and world; logs; clean shutdown
  docs\              DESIGN.md, ROADMAP.md, MILESTONE1.md
  core\              AzerothCore + NPCBots source (our edits: botbubble.*, botdatamgr.*)
  build\             CMake build of core\ (not relocatable - rebuild if moved)
  server\            installed server: worldserver/authserver + DLLs, configs\, data\ (maps/vmaps/mmaps/dbc)
  db\                MySQL 8.4 instance: my.ini (127.0.0.1:3310) + data\
  scripts\           build, configure, era-db, play, stop, measurement scripts
  logs\
```
## Decision 2026-11-15 (late): stock client for now, WoWee parked
WoWee worked (login, world, bubble bots, `.npcbot spawn`, hire, party frames) but its rendering isn't there yet - no
proper night skybox, odd animations. Removed (client data + source); re-download/re-extract when it matures
(~1 h: release zip + `extract_assets.ps1`, then `antialiasing=0` in `%APPDATA%\wowee\settings.cfg` for the
NVIDIA MSAA device-lost bug #157). The biggest WoWee-dependent ideas (integrated listen server, mobile, deep client
edits) wait for that. Also worth watching: *Benilla-WotLK* (currently on 1.12 layers).
**Priority now: a working sim.** The sim lives in worldserver, so it carries over to any client later.

## Sim v1 - first working build (2026-11-15)
Code: `core\src\server\game\SoloSim\` (solosim.h, solosim.cpp, solosim_commands.cpp; hooked in `botconfig.cpp`
`AddNpcBotScripts`). Commands (GM chat / console / SOAP): `.sim run [days]`, `.sim status`, `.sim reset`.
Writes `logs\chronicle.txt` (notable events, by day) and `logs\sim-full.log` (everything); state in `sim_*` tables.
- 30 days x 100 people = ~1 s. Deterministic (same seed -> identical history, verified).
- Bootstraps 43 hubs + 205 hunting grounds (200-yd grid cells, holiday spawns excluded) from the world DB.
- Real WoW xp (BaseGain, GetXPForLevel, grey level, Era 0.5x rate); 10 kills/hour; ~1->10 in 10 days, 2-5 days/level later.
- Daily newcomers (60%/day, cap 130) keep low levels populated; ~1 death and ~1 defeat per day in 100 people.
- Chronicle ~35 lines/day: deaths, defeats, revenge oaths and fulfilment, rivalries, purposeful journeys, every 5th level.
**v1 approximations to revisit**: travel = straight line x1.3 (not the wander-node path); beast loot = level-based
junk value (not loot tables); news of a death reaches friends instantly (should travel); hub names are area names
(several "Stormwind City" hubs); no parties yet; level-ups every 5th still the noisiest chronicle line.

## Plan for the next sessions (agreed 2026-11-15)
Three tracks, in this priority:
1. **Move to WoWee** (pulled forward from "mobile, last"). First job: check how complete WoWee's Windows build is
   against *our* server (login, char create, movement, combat, NPCBots gossip/hire, quests). The sim lives in
   worldserver, so it doesn't depend on which client is used - nothing in track 2 waits on this.
   Goal over time: less reliance on a separate server - WoWee is the only client where server logic can move into
   the client process (integrated "listen server").
   *Status checked 2026-11-15 (README):* pre-release but broad - Vulkan renderer (not OpenGL as the handoff said),
   Windows/Linux/macOS x64+ARM64, Android arm64 (13+, Pixel 9a tested), MIT. Login, char create, movement, combat,
   spells, talents, quests, vendors, trainers, gossip, parties, pets, taxis, mail, AH listed as working. Known issues:
   getting stuck at terrain/WMO transitions (`/unstuck`), Android drops the session when backgrounded, no addon API.
   Reads an **extracted** Data/ tree (manifest.json), not MPQs: full ~18 GB, built-in phone "data profiles"
   (login profile ~787 MB) - this replaces our planned client-trim step, and changes the Worldgen-patch-MPQ idea.
   Untested with NPCBots-specific gossip/hire and bot party frames - first thing to check.
   *Tested 2026-11-15 (v3.1.55, RTX 3060 Ti):* login, character select (after the MSAA workaround), world, bubble
   bots, `.npcbot spawn`, bot gossip + Hire, party frames and following all work. Visuals are rough (odd animations,
   some things look bad) - acceptable for now; WoWee is where the integrated server and launcher will live, so
   deep edits to its source (`C:\SoloWoW\wowee`) are expected.
2. **Start the sim**: agent record first (fields + how the six drives become plans), then registry tables, sim clock,
   `.sim run 30d` chronicle for ~60 adventurers around Elwynn/Westfall/Redridge/Duskwood. See DESIGN.md.
3. **Launcher, slowly in between**: Rust, logic crate separate from UI (reused on Android). Owns the settings we now
   set by script: saves, bot count, bubble on/off, Era preset (configure.ps1 + era-db.ps1), client path, graphics
   preset, and PC vs mobile profiles. Replaced Play.bat/Stop.bat (removed Nov 17).

Current state to pick up from: server builds/boots (`scripts\build.ps1`, the launcher or SoloWoW Manager), bubble working (40/40 in
Elwynn, recycles on zone change), WC3 hero classes off for wanderers, Era preset on by default.

Milestones 1-7 are in the handoff doc. Ideas and decisions added since, in rough order:

## Now
- **Bot bubble** (built, in testing): `botbubble.cpp` keeps `NpcBot.Bubble.Count` wanderers in the player's
  zone + linked zones, recycles left-behind ones out of sight. Has `Pin/Unpin` for later quest use.

## Next: memory
- Grid unloading: this AzerothCore never unloads open-world grids in a session (only `Map::UnloadAll`).
  Re-add unloading of empty grids (no players/bots, idle N minutes). Biggest long-session win.
- Lean MySQL: world data is read once at boot -> buffer pool ~64 MB. Later: SQLite/embedded DB.
- Load spawn data only for the player's continent.
- Fold authserver into worldserver (single local account).
- Target: ~1 GB idle, < 2 GB while playing.

## Then: living world
- **Civilians**: lightweight creatures (not NpcBots) with a schedule (home / work / tavern) tied to server time.
  Travel the wander-node road network between towns; go in and out of buildings; idle at landmarks.
  Random names via runtime template copies; more looks later via a Worldgen-built local patch MPQ.
- **Dynamic enemies & wildlife**: runtime spawns (packs, camps) in places they don't normally appear, inside the bubble.
  - **Roaming hero elites**: bring back the fork's WC3 hero classes (Crypt Lord, Dreadlord, Blademaster, ...) as rare
    hostile world threats - they already wander as hostile-to-all elites (`wbot_faction_for_ex_class`), so they just
    need controlled spawning: rare, level-appropriate, kept out of towns, maybe announced, and tied to bounty quests.
    Currently disabled for wanderers in configure.ps1.
- **Random quests**: reserved quest-ID pool, generated from what's in the bubble (kill / fetch / escort / clear camp).
  Launcher clears the client's WDB cache on start (quest text is cached by ID).
- **Persistence**: per-save table of generated quests (ID, text, objectives, rewards, pinned entities).
  Rebuilt at boot before login; pinned NPCs/camps saved with identity + position and respawned on return.
  ID freed only on complete/abandon. Short memory (~30 min) for unpinned bots/civilians you met.

## With WoWee (milestone 7)
- **Single-program game**: compile the server logic into the WoWee client as an integrated "listen server"
  (like Minecraft single-player) instead of separate mysqld/auth/world processes. Only possible with an
  open-source client; the stock 3.3.5 client is a thin client with no game logic.

## Embodiment v1 (Nov 16, 2026)
- Sim runs **live** with the server (SoloSim.Live.*): real time, hour of day = wall clock, catches up to 72 h of downtime on start, saves every 10 min.
- Journeys follow the wander-node roads (Dijkstra over 5,098 nodes); travel pace 5.5 yd/s, mounted x1.6 from 40. Mid-journey position is exact, so you can cross people on the road.
- Bubble Source=sim: embodies the sim people in your zone(s), nearest first, with their name/race/class/level/gender; steers them (bot_ai GetNextTravelNode hook) toward their sim goal, or strolls them around their site.
- .sim people (nearest 12 in game / per-zone counts on console), .sim hours N (test skip).
- Known gaps: sleepers are embodied strolling round the inn; in-game fights/deaths of bodies don't feed back to the sim; hunt/gather bodies stay near the road node closest to their site, not in the mob camp.

## Next for the sim (agreed)
- **Dangerous bounties**: rares, elites and packs (grouped spawns) as bounty targets; reward and danger scale; the board says "bring friends", which drives FormParty. Some go unclaimed and expire.
- Feedback from bodies: a body killed in game = a sim defeat; the player helping someone = a memory/relationship.

## Bodies v2 + dangerous bounties (Nov 16, 2026, late)
- Bodies act out the sim (botbubble.cpp SimLife, modeled on mod-playerbots NewRpg; source in reference\mod-playerbots, read-only):
  own lane on roads; inns mapped once (chairs/bench slots + indoor floor on every storey, upstairs included), one person per spot,
  same bed every night; errands walk up to vendor / own class trainer and talk; hunting = roam the camp (navmesh-checked points).
- NPCBots habits off for sim bodies: no out-of-combat buffing of others, no casting while sitting/asleep, not stood up by the AI,
  hunter/warlock pets dismissed at the inn (re-summon only when standing).
- A body killed in game = a sim defeat, the chronicle names the real killer.
- Bounties: thinning / pack (25-40, x2.5, bring 2) / elites (bring 3) / named rares (31 found in the region; bring 2-3; slain = gone 5-10 days).
  Dangerous jobs recruit strangers in town (they become acquaintances); no help + not brave = handed back; brave = goes alone.
  A defeated taker drops the bounty back on the board. .sim bounties lists the boards. Columns added in place (no world reset).
- LESSON: test runs on the live save need the DB up before backing up sim tables (a failed backup let a 30-day test advance the real world, Day 33 -> 63).
- Noted for the civilians batch: wandering townsfolk sometimes carry a quest, marked with a purple "!" (see DESIGN.md).
- Noted: civilians go home (despawn at a non-enterable building's door, come out of it in the morning); per-person chronotypes and
  time-of-day activity preferences (adventurers too) so the world is never all asleep at once.

## You in the world (Nov 16, 2026, morning)
- Personal rhythms: larks (up 4-6), ordinary (6-8), owls (up 10-13), 6-9 h of sleep; inn time comes before their own bedtime. Derived from id, no saved field.
- Bounties pay by level only (lvl^2*30+500): the loot is the rest of the pay.
- Player on the boards: .bounty / .bounty take <n> / .bounty turnin / .bounty drop. Kills count at the bounty's camp (name match within 350 yd)
  or the rare itself; first to turn in gets paid (a beaten sim taker holds it against you). Saved in sim_bounty.player/pprogress.
- Hiring: NPCBots hire on a sim body marks the person Hired in the sim; dismissed / body gone -> home. Hirelings who fall count as sim defeats.
  Hired generated bots don't survive a restart: on start every Hired person goes home.
- Memory of the player (sim_memory stype 4): helped in a fight (+), hired (+), beaten to a bounty (-). .sim people shows "(remembers you kindly)" etc.
- Next: civilians (homes, purple-! quests), wider world, market stock in vendors; hire refusals by regard.

## Townsfolk v1 (Nov 16, 2026)
- 34 civilians (sim role 1; 6 per inn town, 10 in Stormwind): jobs farmhand / laborer / market hand / loafer, own chronotype.
  Day: up and about -> ~8 h of work -> errands/market/idling -> evening at the inn (sociable ones) -> home to sleep.
- Bodies are plain creatures, not NPCBots (solosim_civilians.cpp): runtime templates 9,000,000+agent (ObjectMgr::GetCreatureTemplate hook),
  own name + job title, peasant/lumberjack looks and tools. Own AI: walk (navmesh) to fields / timber / vendors / random NPCs /
  inn seats (shared with adventurers), work emotes, talk emotes; at bedtime walk to their own door (outdoor spot beside a
  building wall, same every day) and go in (despawn); come out of the same door. Killed in game -> sim defeat.
- NpcBot.Bubble.Townsfolk = 30 (own cap, nearest first). Purple-! quests for townsfolk: next.

## Markets in the shops (Nov 16, 2026)
- 37 general-goods / trade-supplies vendors near town centres sell what the sim's gatherers brought in (ore, herbs, leather, fish
  by the town's level band); runtime vendor items, synced every 2 min; up to 20 shown; a purchase draws down the town's stock,
  none left = "Sold out - nobody has brought any in lately."; towns use up 30% of their goods a day. .bounty shows "In the shops: ...".
- Hiring: sim adventurers (wanderer bots) now open the NPCBots menu for non-GMs and show the hire option.
- Townsfolk looks: Hillsbrad/Stormwind/Worried Citizens, Hillsbrad Farmer, Dock Worker, Woodworker (no more WC3 peasants).
- Fixed: sim_event rows failed to save since the townsfolk batch (format string); chronicle files were unaffected.

## Bounty boards + letters (Nov 16, 2026)
- A "Bounty Board" (runtime GO template 9,100,000, Sentinel Hill Wanted Board look, no quests) is put up in each inn town when the
  player comes within 250 yd (open spot near the centre, same each time). Gossip via AllGameObjectScript: notices (take), yours with
  progress, shops, check/turn in, drop. .bounty still works.
- Mail from the town's innkeeper: payout with the gold when a player's count is complete (no walk back); "Too late" when a sim
  adventurer turns it in first; "A price on <rare>" to players the sim knows.
- Next: wider world (more zones, a few hundred people, higher camps -> elite bounties, more towns/shops).

## The wider world (Nov 16, 2026)
- Region: + Dun Morogh, Ironforge, Loch Modan, Wetlands, Hillsbrad, Arathi, Stranglethorn, Badlands, Swamp of Sorrows (14 zones).
  Older saves grow on load (ExpandRegion): 37 new towns/outposts, 469 hunting grounds, 64 adventurers (dwarves/gnomes in the
  dwarven lands), 40 townsfolk (dwarf looks there), 98 named rares, 87 shops. Horde guards/towns are not hunting grounds.
- Boards: every 12 h each inn town fills up to 3 bounties (no two on the same camp).
- Backup of the pre-expansion world: acore_characters.bak_sim_* (restore = copy back with the server stopped).
- Watch: Duskwood holds ~50 adventurers; Arathi/Badlands/Swamp have hunting grounds but no inn towns of their own yet.

## Townsfolk you can talk to + their requests (Nov 16, 2026)
- Townsfolk have gossip: "What's the news?" (they say a real recent notable event near their town, from the chronicle),
  their request, hand in, goodbye. Marks via a PlayerQuest.cpp hook: blue ! = request available, grey ? = yours, blue ? = ready.
  (The stock client has no purple; blue - normally "repeatable" - is the distinct native colour.)
- Requests (sim_errand, saved): fetch N gathered goods of the town's tier (reason by job), or cull N of a nearby camp; ~1 in 5
  townsfolk at a time, refreshed every 12 h, 2-3 days to do; pay lvl^2*12+150, +regard, chronicle line.
- More requests: "carry word home" (find a named adventurer from the town wherever the sim has them; within 8 yd they answer and
  really head home) and "never came back" (only for real deaths in the last 10 days; go to where they fell, within 70 yd you find
  what's left, bring the news back). sim_errand.target added in place.
- Next options: launcher; Kalimdor (needs the sim to handle a second map); more request kinds.

## Launcher v1 (Nov 16, 2026)
- C:\SoloWoW\launcher: Rust workspace. core (no UI, no deps: status by ports + tasklist, start/stop via play.ps1/stop.ps1,
  launch Wow.exe after clearing Cache\WDB, SOAP .sim status with a tiny HTTP client, chronicle tail, settings in
  worldserver.conf + C:\SoloWoW\launcher.cfg) and app (eframe/egui 0.29). C:\SoloWoW\SoloWoW.exe = the release build.
- Play / Start the world only / Stop; status lights; "The world" panel: sim status + live chronicle; settings: adventurers and
  townsfolk around you, stop the world when WoW closes (default on).
- Build: cd launcher; cargo build --release (idle priority, half the cores).

## Kalimdor (v4)
- The sim spans two maps (0 Eastern Kingdoms, 1 Kalimdor). Sites, rares and shops are built per map; every 'nearby' check is same-map only.
- New zones: Teldrassil, Darnassus, Darkshore, Ashenvale, Stonetalon, Dustwallow, the Barrens, Desolace. Night elves are about 75% of newcomers in the night elf lands; their townsfolk use Night Elf Commoner / Darnassus Reveler looks.
- Crossing continents: road to a port town (Stormwind, Menethil | Auberdine, Theramore), about 25 min at sea, road on. Nobody has a body while at sea.
- Hunting never crosses the sea; moving home can (PickNewHome looks at every inn town).
- First boot on Day 63: +132 towns and camps, +1,360 hunting grounds, 58 new townsfolk. Backup before it: bak2_sim_* in acore_characters.

## Requests: stolen things, walk me there
- ERRAND_STOLEN (5): a humanoid camp within 1200 yd took a keepsake. One of the camp has it: after 2-6 kills there (hidden count) the player is told they found it. Pay x1.2.
- ERRAND_ESCORT (6): walk a townsperson to the nearest other inn town (300-2200 yd). The body follows the player (running), waits where it is if the player goes 70+ yd away, stops at the destination and is paid there, then goes in. If it dies on the road the request ends and a letter comes from home (nothing owed). Pay x1.5.
- Any civilian death now ends their open request. Townsfolk no longer run back to their door after a fight (EnterEvadeMode override).
- No schema change: stolen uses site/count/progress, escort uses site as the destination.

## Townsfolk levels
- CivilianLevel(): from the home town's regionLevel (cities at least 20), scaled by job: labourer 0.75-1.1, farmhand 0.6-0.95, market 0.5-0.9, loafer 0.35-0.85; 6% are retired adventurers (+8-20). Capped at 1-60. Deterministic, re-applied on every load. First result: levels 4 to 41.
- The user said it's always fine to restart the world while they're playing.

## Hire refusals
- World::HireRefusal(agent, guid, playerLevel): '' = yes. Trust (regard sum >= 60) always comes. Otherwise: hates you (< -30) / grudge (< 0) -> no; in a hunting party -> usually no (sociable ones sometimes leave it); 6+ levels above you and regard < 30 -> no; cautious strangers (caution > 70, never met) -> often no. The answer holds for the game day.
- Hooked into NPCBots' hire menu as reason 10: the option still shows, and choosing it makes them say their answer instead of hiring. Regard grows from rescues (HelpedBy), errands and past hires, so a 'no' can turn into a 'yes'.

## Townsfolk grow
- CivilianLevel() is now a floor; the saved level is kept. CivilianGrow(): +1 level, capped at the starting level + 10 (max 60).
- Daily: labourer 10%, farmhand 7%, market 4%, loafer 2%. A fight their body wins in game: 35%. A safe escort: +1. A chronicle line at every 5th level.
- The body template picks up the new level the next time they step out of their door.
- Not done: ageing (with real-time 1:1, years would never pass; would need its own pace).

## Ageing
- A year passes every game week (SIM_YEAR). sim_agent.age0 = age when 'born' was set; Age() = age0 + years since. AssignAges() gives anyone without one an age: townsfolk 16-74 (skewed young), adventurers 17-47.
- TownsfolkGenerations() (daily): 60+ may give up their work (3%/day, become idlers); 66+ may die in their sleep ((age-65) x 0.4%/day); a town below its count (6, cities 10) has a 25%/day chance that one of its young (16-18) comes of age, taking a family name from a townsperson 36+ of the same race. All of it goes in the chronicle and the town's rumors.
- Adventurers only have an age so far (no retirement).

## Retired adventurers
- RetireAdventurers() (daily, at most 3/day): an Active adventurer not in a party or a fight, human/dwarf/night elf, home an inn town. Chance (age-44) x 0.8%/day from 45; +3%/day with 5+ defeats from 30. Becomes ROLE_CIVILIAN at home: job from nature (greedy -> market, brave -> labourer, else farmhand/idler), a townsfolk look of their own sex, keeps name, level (max 60), relations and memories. Their bounty goes back on the board; messages meant for them lapse. Title 'Retired Adventurer' (SimBody.veteran = civilian with kills).
- Fix: townsfolk bodies now pick a model matching their sex (templates like Darnassus Reveler hold both).

## Start fresh (launcher)
- scripts\reset-sim.ps1: stops the world, renames sim_* -> prev_sim_* (dropping any older prev_), archives chronicle.txt / sim-full.log as *-before-<stamp>. -Undo drops the current sim_*, renames prev_sim_* back and restores the newest chronicle-before-*. Characters are never touched. One level of undo.
- Launcher Settings: 'Start fresh...' and 'Bring back the previous world...' (shown while a chronicle-before-* exists), each with a confirm window. Window icon drawn in code.
- New-world path now matches a grown world: Elwynn region bootstrapped + GeneratePopulation(100), then ExpandRegion() for every other zone, then 14 days of history (chronicle header 'Before you arrived'). Tested: Day 15, 408 people, 13 dead; then undone back to Day 63 (counts matched the bak4_ safety copy).
- .sim reset refuses while the live world runs (its bodies belong to the old people); it also clears requests, mail, rumors and rares now.

## Finding things out in game (instead of the chronicle file)
- Files: sim-full.log only with SoloSim.FullLog = 1 (default off). chronicle.txt rotates past 2 MB to chronicle-older.txt (one older volume).
- sim_event keeps notable events only, for 90 game days (routine rows deleted on load; the pre-change copy is bak4_sim_event). Nothing reads it except RebuildTownTalk.
- Town talk survives restarts: RebuildTownTalk() refills _rumors (8 per town) and _notices (15 per town, timestamped) from the last 14 days of notable events.
- Bounty board: 'Read the notices' (recent happenings near the town) and 'Look for word of someone...' (type a name).
- Townsfolk: 'Have you seen someone?' (type a name -> AskAfter: alive = what they're doing; dead = when/where/what killed them; several matches listed) and 'Any advice for the road?' (Advice: the deadliest place within 2000 yd over the last 30 days; old hands also name a rare they know of).
- ERRAND_OLDFOE (7): retired adventurers ask you to kill a rare (within 2500 yd, within 6 levels of them) that nearly killed them. Pay rareLevel^2 x 20 + 300. Credit on killing that entry anywhere.
- GM/console: .sim who <name>, .sim advice <agent id>.
