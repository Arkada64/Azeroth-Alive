# SoloWoW design: a living single-player WoW

*Merged 2026-11-15 from the "Single-Player Edition" handoff, the "living-world 3.3.5a server" brief, and what we
learned building milestone 1 and the bot bubble. Living document - decisions here are settled unless a note says why.*

## The goal
Play WoW locally, everything bundled into one launcher, in a world that is alive whether or not you're playing.
Adventurers and civilians have lives - homes, jobs, goals, grudges - they travel for reasons, they can die, and the
state of the world is where your goals come from. Randomised gear and generated work make it a journey rather than a
race to 80. The stock quests and zones stay; the living layer is added on top.

**North star: you aren't "the hero" - you're someone living in the world.** Your work grows from rat-killer to someone
whose deeds change the world around you, because the world kept moving while you did it.

**And: you play WoW's content, but the sim always runs something new, so you feel small.** A living fantasy world -
war, a living economy, a world that escalates, your own settlement and home, a family, a legacy that outlives your
character. The full plan by track is in BACKLOG.md.

## Settled decisions
| Topic | Decision |
|---|---|
| Client | **Stock 3.3.5a client for now.** WoWee tested and parked until it matures (see ROADMAP); the integrated-server and mobile ideas wait for it |
| Core | AzerothCore + NPCBots, `trickerer/AzerothCore-wotlk-with-NPCBots`, branch `npcbots_3.3.5` |
| Role of this server | **A destination in its own right**, not a testbed for another engine |
| Language | **C++ inside the fork.** New code in its own files (`botbubble.cpp`, `sim*.cpp`), small marked hooks (`// SoloWoW`) in fork files so upstream merges stay easy. No Eluna: not in this fork, and it can't see `bot_ai` internals |
| Procedural generation | **People, quests and gear** - names, families, traits, goals, jobs, item rolls. Never terrain or buildings: events and rumours refer to Azeroth's real towns, roads and landmarks |
| Players | One. Offline, 127.0.0.1 only |
| Content | Full 1-80. **Planned (Nov 16): expansion gates** - levelling stops at 60 and 70 until the last raid of the expansion is beaten and its item used; Outland/Northrend closed until then (BACKLOG §8) |

## Design principle: peel WotLK back toward the original vision
By 3.3.5 WoW had moved toward retail: faster levelling, mounts at 20, Dungeon Finder teleports, quest-map blobs,
cheap travel. Each of these shrank the world. We peel them back so the player *exists in* the world - but friction
alone isn't immersion; vanilla felt big because there was friction **and** reasons to be somewhere. The sim supplies
the reasons; the levers below restore the friction. Ship them as a launcher preset ("Era"), not hard-coded.

| WotLK accelerant | Lever | Cost |
|---|---|---|
| Faster levelling (2.3/3.x XP cuts, bigger quest XP) | `Rate.XP.Kill/Quest/Explore` < 1; rested XP via `Rate.Rest.*` | config |
| Dungeon Finder: queue anywhere, teleport in | `DungeonFinder.OptionsMask` (drop the LFD bit) - walk to the dungeon, bots/hirelings form the group | config |
| Quest objective blobs on map/minimap (3.3) | empty `quest_poi` / `quest_poi_points` - you read the text and look for it | DB |
| Riding at 20/40 (3.2) | `trainer_spell` ReqLevel/MoneyCost for riding back to 40/60 and vanilla prices | DB |
| Cheap respecs, dual spec | trainer costs / disable dual spec | DB |
| Taxi network known everywhere | keep `AllFlightPaths = 0`, `InstantFlightPaths = 0` (already) | config |
| Durability, death, corpse runs | keep; `Death.SicknessLevel`, `Rate.RepairCost` tuned with the sim's money model | config |

**Pace decision (2026-11-15): slower than vanilla.** 0.5x XP; the sim's jobs and ambitions are meant to carry
progression, not quests. Implemented as the default "Era" preset: `configure.ps1` (XP 0.5, `DungeonFinder.OptionsMask = 0`;
`-NoEra` for stock) + `era-db.ps1` (quest POIs moved to `solo_era_quest_poi*`, riding 40/20g, 60/900g, flying 70/800g;
`-Off` restores). Until the sim exists, 0.5x is just a slower grind - use `-NoEra` for testing sessions.

## Adventurer drives (accepted 2026-11-15)
Each drive generates plans out of the verbs below.
1. **Growth** - hunt where the level fits; move on to the next region when outlevelled.
2. **Upkeep** - money, repairs, food; sell loot in town; broke -> take any job.
3. **Rest** - sleep at an inn or camp; injured -> retreat to town.
4. **Ties** - a home town, a few friends/rivals; sometimes travel in pairs.
5. **Ambition** - one long goal each ("make Stormwind's barracks", "avenge a partner killed in Duskwood",
   "get rich on Westfall bounties").
6. **Death** - permanent; the town notices (rumour), and someone may take up their goal.

## Architecture
**Two levels of detail.** Every person is a *row*; a *body* only exists near the player.
- **Off-screen ("abstract")**: the sim ticks rows on its own clock - travel between sites, work, rest, fight
  abstractly (odds by level/danger), die, get replaced. No creatures, no grids, no pathing. Thousands of rows are cheap.
- **On-screen ("embodied")**: inside the bubble a row becomes a creature (an NPCBot for adventurers, a light
  creature for civilians/wildlife) whose AI only *executes the row's current plan step*. Leaving the bubble writes
  state back and removes the body. This is `botbubble.cpp` today, minus identity.

**Sim clock, separate from game time.** WoW game time is real time, so the sim has its own clock. In play it runs at
real time (or a chosen rate); headless it steps as fast as it can: `.sim run 30d` prints a chronicle in seconds.

**Determinism.** The sim has its own seeded RNG (per save) and never iterates unordered containers when deciding.
Bodies may be non-deterministic (combat, pathing) - only the rows' history must be reproducible.

**Sites as a graph.** Two layers:
1. *Road graph*: the fork's 5,098 wander nodes with links and level ranges (`creature_template_npcbot_wander_nodes`),
   already used by bots and by the bubble's zone graph. Pathfinding over it exists: `WanderNode::GetShortestPathLinks`.
2. *Places*: authored sites on real geography (home, inn, forge, field, mine, camp, gate), each attached to its nearest
   road node. Authored in-game with a GM command at your position (`.sim site add <type> <name>`). **This authoring is
   the real cost of each new area**, not code.

**Plan verbs** (the contract between brain and body):
`GOTO(site) WAIT(s) WORK(site,dur) SLEEP(site,until) TALK(agent) HAUL(item,from,to) FIGHT(target) FLEE(site) USE(object) IDLE`.

**Event log** - append-only, per save. The single output: debugger, chronicle, and the source of rumours/gossip.

**Wildlife ledger** per group (population, hunger, boldness, territory, fear of the player); failed hunts raise
boldness; cleared sites get reoccupied by the highest-pressure unclaimed faction.

**Persistence** - all sim state lives in the save's own characters DB (one characters DB per save): agents, sites
ownership, plans, quests in progress, generated items, event log.

## Brain and body (decided 2026-11-15)
- **Brain = ours, from scratch**: identity, drives, plans, sim, memory, persistence, chronicle, hiring.
- **Body = NPCBots, reused**: combat, casting, healing, following, moving between points (~3 MB of tuned C++ we
  don't rewrite). Replace individual body pieces only if one gets in the way.
- **Playerbots = reference, not dependency** (source already in `D:\AzerothManager\source\modules\mod-playerbots`):
  study its travel manager (flight paths, boats, zeppelins, portals; travel node data) and its decision logic
  (quests, professions, spending). Being a full Player is what makes playerbots expensive; we port the logic only.
- **License note**: NPCBots/AzerothCore are GPL-family, WoWee is MIT. Combining is allowed; the combined program
  falls under the GPL terms (matters only if ever handed to others).

## Adventurers, built from NPCBots
**What a wanderer does today** (read from `bot_ai.cpp`): a weighted random walk.
- `bot_ai::Evade()` (~l.18611) is the movement driver: walk to `_travel_node_cur`; on arrival call
  `OnWanderNodeReached()`, pick `GetNextTravelNode()`, wait `urand(node wait time)` or "mill about".
- `GetNextWanderNode()` (~l.19145): random weighted link that fits the bot's level and faction, avoiding the node it
  just came from. No destination, no memory.
- `OnWanderNodeReached()` (~l.20200): battleground-only; in the open world it does nothing.
- They eat/drink when low (`Feasting()`), fight what they meet, level from kills (`XPGain`).

**Hook points for a life:**
| Hook | Becomes |
|---|---|
| `GetNextTravelNode()` | if the agent has a plan step `GOTO(site)`, follow `GetShortestPathLinks` toward it instead of a random link |
| `OnWanderNodeReached()` | report arrival to the brain; start `WORK/WAIT/SLEEP/USE` for the plan step |
| `evadeDelayTimer` | plan-step duration instead of the node's random wait |
| death / despawn | report to the brain -> event log; row becomes dead (permanent) |
| bubble spawn | spawn *this agent's* body (name, looks, gear, level from the row) instead of a random spare bot |

**Identity & hiring** (verified gaps):
- Wanderers are runtime copies of bot templates held only in memory (`_botsExtraCreatureTemplates`); recycled = gone.
- They can't be hired: `bot_ai::OnGossipHello` (l.7717) returns early for wanderers; the hire option needs `!IsWanderer()`.
- Fix both with the registry: an adventurer row stores template source, name, looks, gear, level, spec; the body is
  generated from it; hiring flips the row to "owned" and persists it as a real owned bot.

**How NPCBots stores a bot** (read 2026-11-15):
- **One bot = one `creature_template` entry.** `characters_npcbot` (characters DB) is keyed by `entry` (PRIMARY KEY):
  owner (character lowguid, 0 = free), roles bitmask, spec, faction, 19 `equip*` columns (item_instance guids),
  `spells_disabled`. Side tables: `characters_npcbot_group_member`, `_stats`, `_transmog`.
  World DB: `creature_template_npcbot_extras` (race/class), `_appearance` (gender, skin, face, hair...), plus one
  `creature` spawn row per static bot (`.npcbot spawn #entry` refuses if the entry already has data or a spawn).
- So **static persistent bots are capped by templates** (368 shipped; `.npcbot createnew` can add more, but those live
  in the *shared* world DB, not the per-save DB).
- **Generated bots** (wanderers, dungeon bots) are runtime copies of a template under a fresh id (`next_bot_id`,
  persisted in `worldstates` as `BOT_GIVER_ENTRY`), held only in memory (`_botsExtraCreatureTemplates`, `_botsData`,
  `_botsExtras`, `_botsAppearanceData`, equipment), cleaned up on despawn (`CleanExtraBotData`).
- **Registry consequence**: agents are not templates. An agent row stores its *source* template (looks/class),
  generated name, level, spec, gear; its body is a generated copy made on embodiment. **Hiring** = persist that
  generated copy per save (our own table, loaded into the in-memory maps at boot *before* `LoadNpcBots`, so the hired
  entry's `characters_npcbot` row resolves). This keeps everything per save and lifts the 368 cap.

**Off-limits for wandering**: the fork's 9 WC3 hero classes are hostile-to-all (`wbot_faction_for_ex_class`).
Disabled for wanderers; planned return as rare roaming elites (see ROADMAP).

## First build: "adventurer lives", headless
Prove the history is worth reading before any client work.
- Area: Elwynn, Westfall, Redridge, Duskwood (the bubble's zone graph around Goldshire).
- ~60 adventurers + the sites they need (inns, vendors, trainers, hunting grounds by level, the roads between).
- 30 sim days via `.sim run 30d`, chronicle printed to a file. Read it. If it's boring, fix the model, not the content.
- Then embody: the bubble spawns registry agents instead of random bots, and they act out their plans in front of you.
- Civilians and wildlife reuse the same registry, clock, sites, verbs and log.

## Sim v1 spec (DRAFT 2026-11-15 - for review before code)
Scope: adventurers only, off-screen only, around Elwynn / Westfall / Redridge / Duskwood (+ Stormwind as a hub).
Output: `.sim run 30d` -> `logs\chronicle-<seed>.txt`. Embodiment (bubble spawns registry agents) comes after.

### Time and determinism
- **Sim clock** in sim-minutes, stored in `sim_state`. In play it advances with real time (1 sim-min = 1 real min,
  so day/night matches the game clock); headless it jumps from event to event.
- **Event-driven, not per-tick**: each agent has a `wake_at`. A priority queue ordered by `(wake_at, agent_id)` wakes
  agents only when their current plan step ends. 10,000 agents cost ~10,000 decisions per sim-hour at most, not
  10,000 x 60 updates.
- **RNG**: one seed per save; every roll uses a sub-seed `hash(seed, agent_id, wake_at, purpose)`, so any run is
  reproducible and changing one agent never reshuffles another.

### Tables (in the save's characters DB, all prefixed `sim_`)
| Table | Holds |
|---|---|
| `sim_state` | seed, clock, next ids, version |
| `sim_site` | id, type (inn, vendor, trainer, flight, hunt, camp, home, landmark), name, map, x/y/z, zone (from nearest wander node), road node, level range (hunt/camp), faction, tags |
| `sim_agent` | see below |
| `sim_relation` | agent a, agent b, kind (friend, rival, partner, family), strength -100..100 |
| `sim_memory` | agent, subject (agent / player guid / site / faction), event type, valence -100..100, intensity 0..100, at, event id - max ~16 per agent |
| `sim_event` | append-only log: id, at, type, map, zone, site, actor, target, a few typed numbers, text key - feeds chronicle, rumours, memory |

### The agent record (`sim_agent`)
| Group | Fields |
|---|---|
| Identity | id, given name, family name, gender, race, class, spec, faction/team, source bot template (looks/class), appearance (skin, face, hair, colour, features) |
| Progress | level, xp, gear tier (abstract 0-100 vs level; real items picked only when embodied or hired), money (copper) |
| Body state | wounds 0-100, fatigue 0-100, supplies 0-100 (food/water/ammo/reagents), durability 0-100, bag fullness 0-100 (unsold loot) |
| Personality | bravery, greed, sociability, caution, wanderlust - 0..100, seeded at birth, never change |
| Life | home site, birth time, status (active, recovering until X, dead, hired by player), defeats count, death time/cause/killer |
| Ambition | type, target (site/agent/level/money), progress, started at |
| Plan | current verb, its target, started at, `wake_at`; remaining steps of the plan (small serialized list); party id |
Bodies are runtime-only: when embodied, the bot entry and creature guid live in memory, never in the row.

### How drives become plans (utility scoring)
When an agent wakes, each drive scores 0-100 from its state x personality; highest wins (seeded tie-break), and
it emits a short plan of verbs:
| Drive | Urgency from | Plan it emits |
|---|---|---|
| Rest | fatigue (+ night), wounds | GOTO(inn or camp) SLEEP(until morning / healed) |
| Upkeep | low supplies, low durability, full bags, low money (x greed) | GOTO(town) USE(vendor: sell, repair, restock) |
| Growth | always some; more if xp to next level is close (x bravery) | GOTO(hunt site fitting level) FIGHT(site, 1-3 h) |
| Ties | sociability x time since seeing friend/home | GOTO(friend's site or home) TALK(friend) WAIT |
| Ambition | ambition progress x wanderlust | its own step (travel to a new region, hunt a rival, save money...) |
| Move on | outlevelled the area (grey-level formula) | GOTO(next region's hub) - changes home if it stays long |

### Abstract fights (off-screen FIGHT at a hunt site)
- Per hour: kills from a rate by class/role; **xp per kill from WoW's own formula** (level difference); loot value
  from the site's creatures (`mingold/maxgold`, world-drop value by level); supplies and durability drain.
- **Danger** per hour from (site level - agent level), party size, wounds, caution: adds wounds; at a threshold it is a
  **defeat** -> survival roll (10% final, +15% per earlier defeat, cap 85%); survivors become `recovering` at home.
- Tuning target for the first chronicle: a level-1 adventurer reaches ~10 in 3-5 sim days, ~20 in 2-3 weeks.

### Population v1 (seeded)
~60 adventurers: Alliance races weighted to the area (human, dwarf, gnome, night elf, draenei), normal WoW classes only
(no WC3 heroes), levels 1-25 skewed low, homes in Goldshire / Stormwind / Sentinel Hill / Lakeshire / Darkshire,
2-4 relations each, one ambition each from a small template list.

### Sites v1 - bootstrapped, then hand-tagged
Generated at first run from the world DB: **hubs** = clusters of service NPCs by `npcflag` (innkeeper 65536, vendor
128, repair 4096, flight master 8192, trainer 16); **hunt sites** = clusters of hostile spawns, level range from
their templates; zone from the nearest wander node (spawn rows' `zoneId` is almost always 0: 39 of 30,191 on map 0).
Travel time between sites = path length over the wander-node graph / walking speed. Hand tags come later.

### Chronicle
Each `sim_event` renders through a text template, grouped by day, e.g.
`Day 3, 14:20 - Garrick Ashworth (human warrior, 8) left Goldshire for the Jasperlode Mine.`
`Day 3, 17:05 - Garrick was beaten by kobolds at the Jasperlode Mine and dragged himself back to Goldshire (2nd defeat).`

### Decided 2026-11-15
1. Population v1: **100** adventurers.
2. **Real WoW formulas** for xp (per-kill gain, xp per level, grey level).
3. **Mixed starting levels** (lively from day 1).

## Survival and memory (decided 2026-11-15; part of the first sim build)
**Defeat is not always death** (Shadow of War style). On an open-world defeat, the chance it is final starts low and
rises with every defeat: ~10% first, +15% per later defeat, cap ~85%. Survivors are "carried off"/flee, recover at home
for a few days, and return with a scar (memory) and maybe a grudge. Same roll on- and off-screen.
**Never counts**: battlegrounds (bots there are temporary participants), duels, sparring.

**Memory**: each agent keeps ~16 strongest entries `(who, what, valence, intensity, when)`, derived from the event log,
fading over time (strong ones slowly). Examples: "Myrtha hired me", "Myrtha left me to die in Duskwood", "Garrick
saved me from gnolls". Memory drives hire willingness/price, greetings, travel companions, and ambitions
("hunt down the orc who nearly killed me").

## War: soldiers, armies, RP-PvP (decided 2026-11-15; after adventurers work)
- **Soldiers** are agents like adventurers: patrol, hold positions, raid enemy settlements, defend.
- **Scale**: battles resolve mostly abstractly (strength, odds, losses); the bubble embodies a slice near the player
  (~15-25 bots). NPCBots' battleground AI (capture/hold/defend objectives) is reusable body behaviour.
- **What can change hands**: contested places and outposts only - camps, roads, outposts, contested hubs (Southshore vs
  Tarren Mill, Ashenvale, Stonetalon, Arathi keeps, Hillsbrad farms, Thousand Needles...). Plenty of them, so the war
  is something to worry about. **Never** anywhere important: protected towns can be raided, besieged, patrolled near,
  never captured (their quest givers/vendors must not die or vanish).
- **Soldier levels match their front**, like WoW's guards. Out-of-level danger comes only from rare roaming elites.
- **Realm type RP-PvP standard**: `GameType = 8` in worldserver.conf and `realmlist.icon = 8`. Never FFA (16): the bot
  generator turns every wanderer hostile-to-all on FFA realms. Sanctuaries stay sanctuaries.

## Multiplayer modes (option, not foundation)
Single-player stays the design centre. Modes, all carrying the same game connection to the same server:
1. **Solo** - integrated server on localhost (default).
2. **Host** - same server opened to friends: open server (port forward, real passwords, launcher warning), LAN
   (also covers phone -> PC), optional **Steam tunnel via Spacewar (app 480)** for invites/NAT relay - private use
   only, nothing depends on it, Valve may change it; overlay networks (Tailscale/ZeroTier) documented as the
   private-by-default internet option.
3. **Join** - client only, to a friend's world or a dedicated box (likely the main mobile mode: phone plays, PC hosts).
**Rule from day one: no single-player assumptions in sim code.** Bubble = union of all players' bubbles (today's
`FindLocalPlayer` picks the first player - fix when sim lands); memory keyed by subject; war, reputation, hiring per player.

## Loot (decided 2026-11-15)
Parity by construction - copy WoW's systems and numbers, never invent a stat formula.
- **Wildlife**: materials only - beast junk, meat, skinning, quest trophies. Rare "stomach find" from the zone's
  world-drop pool at world-drop odds. Beasts carry no gear or money, as in WoW.
- **Humanoids, camps, hero elites**: WoW's own rolled loot - random-suffix items ("Bandit Cinch of the Monkey"),
  stats from the client's ItemRandomProperties/ItemRandomSuffix data, no new IDs. Rates measured from the existing
  world-drop reference tables per level band; copper from `mingold/maxgold` by level.
- **Generated quest rewards: option A** - existing items only (random-suffix or static items of right level/slot).
  XP via `RewardXPDifficulty` + quest level (so Era's 0.5x applies), money by level.
  Option B (new named uniques with reserved IDs, median stat budget of same slot/quality/ilvl, never more) is
  parked for agent-story quests later.

## Generated quests
**Stakes ladder.** The generator never offers a quest that is grey for the player (WoW's grey-level formula).
When an area has nothing left for you, locals say so and a rumour points onward ("the Redridge watch is short on swords").
| Band | Work | Consequence |
|---|---|---|
| 1-10 | chores: vermin, wolves at the farm, deliveries, lost tools | a farm's crop, a vendor's stock |
| 10-30 | local threats: bandit camps, missing people, escorts between towns | road safety, who lives, prices |
| 30-50 | regional: outpost raids, war fronts, bounties on roaming hero elites | which outposts hold, zone pressure |
| 50-80 | world-shaping: retaking sites, sieges, war objectives, your nemesis | who controls an area, who lives |

**Zone kits**: built from the world DB (beasts, humanoid factions, named places, levels per zone) plus small hand
tags ("farmland", "mine", "Defias territory", "war front"). Templates x kit. Quests can span neighbour zones
(kill, deliver to someone, craft, fetch from a rare enemy in a cave, escort...).

**Procedural Quest System handoff** (`Handoff: Procedural Quest System`, Worldgen project) slots in here:
- Its **portable quest graph** (symbolic keys, never engine IDs; adapters resolve them) becomes the shared format.
  Our dynamic generator (world events from the sim: bounties, missing persons, shortages, war) and its authored-feeling
  **zone story arcs** (canon threat, local villain, motive, twist, 6 beats, side quests) both emit that graph.
- Its rules are adopted: never contradict canon (canon lock); every seed must be good (validate + repair in the
  generator); reuse stock items for rewards; generated content only in our own ID ranges; clear the WDB cache.
- **What the living world provides it** (its "living-world adapter"): the zone graph with place tags (our sites +
  wander-node graph), an NPC/creature spawn API (the bubble/registry), text and reward hooks, and which generator
  runs per zone. Quest givers can be our sim agents, so a chain's "trusted traitor" can be someone you actually know.
- **Persistence**: generated quests are rows in the save until completed/abandoned; pinned givers/targets/camps are
  saved and respawned; IDs freed only on completion.
- Its lore kit lives in `D:\Worldgen\kits\lore\` (motives, twists, beats reused as-is).

## Era preset additions (vanilla direction)
Implemented so far: XP 0.5x, no Dungeon Finder, no quest POIs, vanilla/TBC riding (see pace decision). Planned:
- **"Vanilla diff" tool**: compare this world DB with a vanilla DB (VMaNGOS) by matching creature/quest IDs and revert
  only changes toward easier - faction (neutral back to hostile in starter zones), rank (normal back to elite),
  health/damage modifiers, quest type and group size. Classic-era zones only (Outland data is already post-2.3);
  same IDs only. Backup tables, reversible. Group content becomes "hire a party" content.
- **Professions**: uncapped (`MaxPrimaryTradeSkill`), each new one costs more than the last (~1x, 2x, 4x, 8x; small
  trainer-cost hook), slower skill-ups (`SkillChance.Yellow/Green` lower; gathering gain settings).

## Living-world NPC systems
- **Shops with stock**: `npc_vendor.maxcount` + restock (native, client shows quantity); player and agents deplete the
  same stock. Prices can be dynamic (price is sent in the vendor list).
- **Gatherer jobs**: fishermen, hunters, miners, herbalists haul goods to vendors - off-screen arithmetic adds stock;
  on-screen bodies fish (channel visual), skin, mine at real spots. You see others fishing.
- **Social life**: sitting at tavern tables, eating/drinking visuals, two agents TALKing with /say lines and emote
  animations (/talk /laugh /point /cheer). Lines are templates filled from memory and the event log - that is how
  rumours reach the player. No AI-generated text.
- **Scale**: 10,000+ simulated people (rows, cheap ticks) is a fair target; bodies at once ~100-300 (bubble), by level
  of detail - civilians cheap (walk/sit/emote/talk), combat bots at full AI only when fighting or near the player.
  (10,000 embodied NPCBots is not realistic: each runs full class AI; we measured 150 at ~0.2 core.)
- **Navigation that doesn't look like ants**: each trip targets a random point within the node's proximity radius
  (`WanderNode::GetProximity()`), per-agent seeded road-side offset, mixed pace (walk/run/mount, varied speeds and
  pauses), small detours to sites, groups walk side by side. First two go in the first embodied build.

## Known engine limits
- Open-world grids never unload in this AzerothCore (only `Map::UnloadAll`) - visited areas stay in memory.
- 368 bot templates total; generated agents reuse them as appearance/class sources (names come from the sim).
- Client caches quests/items/creature names by ID (WDB) - generated content uses reserved ID ranges; launcher clears cache.

## Not yet examined
- Custom races (`mod-worgoblin-high-elf`): stability, client patch, and compatibility with the NPCBots fork.
- Client modernisation (WotLKExtensions etc.).

## Wandering townsfolk with quests (noted Nov 16, 2026)
- Some of the townsfolk/civilians who wander and idle around can have a quest of their own: not all the time and not many,
  something for players to keep an eye out for (procedural, from their sim life: a lost item, a relative at a camp, a debt).
- They should stand out from static quest givers with a **purple exclamation mark** over their head.
- Client note: the stock 3.3.5a client only draws yellow/blue/grey quest marks from quest status, so purple needs our own
  marker: a custom spell visual/aura or a small object above the head (to test), or a proper marker once WoWee is the client.

## Civilians' homes (noted Nov 16, 2026)
- Civilians don't sleep in taverns (that's for travellers/adventurers). Each civilian gets a "home": one of the town's
  non-enterable buildings. At home time they walk to its door and despawn there; next morning they come out of the same door.
- Homes: door spots found per town (building fronts near the town's roads/paths), each assigned to a person or household and kept.

## Day/night rhythm per person (noted Nov 16, 2026)
- Not everyone sleeps at the same time. Each person (civilian AND adventurer) gets a chronotype: early riser, ordinary, night owl,
  plus how much sleep they need. The world is never fully asleep: night watch, drinkers at the inn, night hunters.
- Activity preferences shift by time of day per person: e.g. one hunts at dawn and trades at noon, another socializes in the
  evening and hunts at night. The sim's drive scores read the person's own clock, not one shared IsNight().
