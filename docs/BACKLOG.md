# Azeroth Alive (SoloWoW) - backlog

**North star:** you aren't the hero, you're someone living in the world - you play WoW's content, but the sim always
runs something new, so you feel small. A living fantasy world.

Tracks, roughly in the order we work them. Each step should be playable on its own. Done items move to ROADMAP.md.

## 0. Repo and install (Nov 16)
- GitHub: github.com/Arkada64/Azeroth-Alive (public). Never commit Blizzard files, client, databases, binaries, logs.
- One-click installer: installs our server + launcher and patches a client the player provides (realmlist, LAA,
  later our patch MPQ). Server binaries ship as release downloads, not in git. Core changes live as a patch over a
  pinned trickerer/AzerothCore-wotlk-with-NPCBots commit.

## 1. Correctness (now)
- **Playtest feedback, Nov 17 (04:35) - the user's list.** Done the same morning, not yet seen in game: errands pay
  lvl^2 x 3 + 40 copper (was x12 + 150), escorts x distance (800 yd = 1x, 0.5-3x) x danger (camps/threats on the way,
  up to 3x), old foes lvl^2 x 6 + 100; requests posted at 3.5% a tick (was 20%) and trimmed to one in seven townsfolk;
  rumours and board notices only from news worth passing on (deaths, defeats, clashes, rares, bounties, comings and
  goings, ambitions done - not socialising); an escorted townsperson settles where you took them (their new home, the
  body stays); townsfolk hold still while you talk (45 s, renewed by each choice, "Good day." releases); the talk
  animation now comes with words in /say when you're within 25 yd (templated small talk by job, or the town's news, at
  most once a minute each); murlocs take the Trolls' starting kits, action bars and shared class skills (the Undead
  have no shamans or hunters, so those got nothing) - server side applied, client patch-X rebuilt when WoW is closed;
  an existing murloc shaman needs re-creating. The list:
  - Fixes (quick): townsfolk work pays far too much (11 silver to walk someone Stormwind -> Goldshire) - pay by level,
    danger and distance, money shouldn't come easily; far fewer townsfolk with requests (more had one than not);
    rumours: drop the "X shared stories with Y" kind - only what a listener would want to know; an escorted townsperson
    vanished after paying - they should stay and live in that area; townsfolk walk away while you talk to them - stop
    them; townsfolk play the talk animation but nothing shows in /say; bounty boards float (a wall board standing in
    the open) - a better model, or the same one placed against a wall where it makes sense.
  - Generated quests/requests: a quest marker (map/minimap POI) where to go - vague is fine, an idea is the point.
  - Townsfolk lives: walking onto railings and tables, several sitting in the same spot, random paths - robotic. Real
    places to go and things to do (stalls, benches, wells, doors, each other), one person per seat.
  - Embodied count: 40 bodies at a time feels empty (even at night); live WoW areas had hundreds - optimise toward
    hundreds (lighter bodies far away, sleepers indoors, townsfolk cheap).
  - New world fills at once: a fresh world (the user will restart theirs at the end) should be fully populated from
    the first start, not dripped in.
  - Danger recognition (adventurers and townsfolk): leave areas, avoid roads or sneak through when a place is judged
    too dangerous (camps, threats, war, recent deaths) - builds on immigration's push and the route costs.
  - Crafters: people working the professions - goods, armour, weapons, potions - who gather or buy every ingredient,
    need the right place (forge and anvil, alchemy lab...) and sell what they make (shops, markets, the player).
  - Cooks: buy ingredients or hunt for them, cook, sell to taverns and shops (the inns' food comes from someone).
  - Adventurers: interact with the world around them and talk with each other (in /say, near you).
  - Camping (from WoW Forever) brought to 3.3.5: a camp you set up to rest with your party, with benefits; sim people
    camp too, alone or together, between areas.
  - Dialogue: "might be time to consider a text generator" for natural talk about world events, life and rumours,
    with you and between NPCs. NOTE: the standing rule so far is templated NPC text only, no AI-generated text - the
    user is reopening that; options when we get there: a bigger procedural grammar (combinatorial templates fed by the
    sim's facts - still no AI) or a small local language model (cost: RAM/CPU on this PC, consistency, and it
    changes the rule). The user decides.
  - Graphics: Lumenveil (github.com/lumenveil/Lumenveil, GPL v2, no game assets; fog, light shafts, lamp lights,
    water, per-zone looks, longer view distance; copies files next to Wow.exe, hooks Direct3D in memory) into the
    installer (fetched, not vendored). Check its docs/INSTALL.md against our DXVK + ReShade setup and our MPQs first.
    "Could be used with Project Reforged models": their HD packs are their files - we don't ship or copy them.
  - Murloc (then every new race the same way): no starter quests in the start area; sheath/unsheath doesn't work and
    weapons on the back sit wrong; items picked up instantly (no loot animation); characters start without spells -
    a murloc shaman had no class spells and no auto-attack on 1 (the race's playercreateinfo_spell/action rows).
- **Audit Nov 17 (01:55) - fix these:**
  - Not in the sim yet (no towns, adventurers, camps, herds or thinning there - an empty world): Eastern Kingdoms -
    Alterac Mountains, the Hinterlands, Searing Gorge, Burning Steppes, Blasted Lands, Deadwind Pass, Western and
    Eastern Plaguelands; Kalimdor - Thousand Needles, Feralas, Tanaris, Un'Goro Crater, Silithus, Felwood,
    Winterspring, Azshara, Moonglade. That's most of the 35-60 world, so the sim's people stall in the 40s (highest
    45, only 14 of 468 are 40+): add these zones (ExpandRegion handles a grown world) - and with them the neutral
    goblin towns (Gadgetzan, Everlook) and the Horde/Alliance outposts there.
  - Horde capitals are small: Orgrimmar's and the Undercity's sites were filed under Durotar and Tirisfal (their
    city zone ids 1637/1497 never show), so they get town-sized folk (48 for all of Durotar, 48 for Tirisfal) instead
    of a capital's 100-220 like Stormwind/Ironforge/Darnassus; Thunder Bluff (1638) has no townsfolk at all.
    Also: the indoor-door search for the Undercity, and the Horde capitals' districts.
  - Bad hunting grounds: training dummies count as level 60-80 grounds in Stormwind, Ironforge, Darnassus and Durotar;
    Lordaeron Citizen (the Undercity's ghostly ambient folk) is a 15-70 ground in Tirisfal; some grounds have
    merged level ranges (Flatland Cougar 7-80 in Mulgore, Slimeshell Makrura 12-57 in the Barrens). Exclude
    dummies/ambient/friendly creatures and keep a ground's level range to its own creature.
- Deaths by place: raids never touch the sim; dungeons rarely final (15% of the open-world chance); open world as is.
  SoloSim.Permadeath = 0 turns permanent death off for testing. (done Nov 16)
- Catch-up: one week cap (SoloSim.Live.CatchUpHours 168), quieter: deaths x0.3, people with regard >= 40 for a player
  never die, majors (threats, camps) a third as often and at most 2 per absence, the player's bounties and requests
  wait. (done Nov 16)
- A player who kills a threat/camp without taking its bounty is not paid (decide: pay anyway?).

## 2. Townsfolk
- **Done Nov 16 (night):** town sizes (Stormwind 220, Ironforge 160, Darnassus 100 - shared between a city's inns;
  other towns 5/10/18/30 by the services across their districts) -> 746 townsfolk; districts = a town's own service
  clusters within 450 yd (Stormwind has 14), weighted by activity - townsfolk live and work in theirs, evenings at the
  town's inn; race mix (30% gnomes among dwarves, Stormwind ~85% human with dwarves/gnomes/night elves, a few humans in
  Darnassus); looks drawn from every humanoid NPC model of their race and sex with its own face/hair/clothes, from
  ordinary-people templates (whitelist + blacklist), 94-106% height; doors found indoors too (Ironforge); 70 townsfolk
  bodies around you in capitals. Pool sizes are logged ("SoloWoW townsfolk looks") the first time someone spawns.
  Immigration (daily, at most 5 a day): 0.2%/day base, pushed up to 3%/day by danger (recent deaths from the town,
  camps within 1500 yd, threats within 2000 yd); destination = a town on the same continent with room, weighted by room,
  their own people, safety and distance; they travel the road (no body on the way); veterans (kills) and people with
  open requests stay. Not yet seen happening: towns were just filled to size, so moves start as room opens up.
  Still open below: props by job, draenei, cross-continent moves, refugees in bulk after raids.- Races: gnome townsfolk (Tinker Town, Kharanos, Stormwind - Gnomeregan refugees); draenei (needs map 530).
- Varied looks: every suitable humanoid NPC display in the client by race/gender (citizens, villagers, refugees,
  festival-goers...), no guards/soldiers/named characters. Props by job (pitchfork, hammer, broom, mug, lantern...),
  slight scale variation, idle habits. Later: a client patch with generated appearance combinations.
- Town sizes: outpost/camp (few civilians, later soldiers) / village 8-15 / town 20-40 / city ~100 / capital 200-300,
  measured from the game's data. Districts (subzones): Stormwind wards, Ironforge Commons/Tinker Town, Orgrimmar valleys,
  Undercity quarters - live, work, drink in their own; bodies near the player in capitals 60-80 (nearest first).
- Indoor cities (Ironforge, Undercity): doors found inside city buildings (the outdoor wall test finds nothing there).
- Immigration: room per town (the sizes above), push (danger, poverty, lost family), pull (room, prosperity,
  safety, kin, own people - not absolute), they travel the roads; refugees after camps/threats/raids; mixed towns.

## 3. Factions and the war
- **Stage 1 started Nov 17 (00:35):** towns have a side (sim_site.side: Alliance / Horde / neutral, from their service
  NPCs' factions; vendors who serve anyone don't count). The Horde homelands joined the region (Durotar, Orgrimmar,
  Mulgore, Thunder Bluff, Tirisfal, the Undercity, Silverpine), and an older world gains the Horde towns of zones it
  already had (the Crossroads, Camp Taurajo, Tarren Mill, Stonard...): 83 Horde towns, 180 Horde adventurers (orcs,
  tauren, Forsaken, trolls by homeland), 274 Horde/goblin townsfolk (Orc/Troll/Tauren/Forsaken/Goblin Commoner bodies,
  their own name sets). Agents only live, trade, rest, move or settle in their own side's towns or neutral ones;
  newcomers are born as their start town's people. Backup of the pre-Horde world: db/sim-before-horde.sql. Not yet:
  zeppelins (Horde can't cross the sea), clashes, enemy bodies hostile near you, mixed parties in neutral towns.
  **Sea crossings and clashes, Nov 17 (01:00), not yet seen running:** crossings are the real 3.3.5 routes - Alliance
  boats Menethil-Theramore and Menethil-Auberdine, Horde zeppelins Grom'gol-Orgrimmar and Tirisfal-Orgrimmar, the
  goblins' Booty Bay-Ratchet boat for anyone - each dock served by the nearest welcoming inn town (before: any inn in a
  "port zone", for everyone, and no zeppelins). Clashes: adventurers of the two sides hunting the same ground (or one
  within 350 yd) at the same time (15%, at most once a day each, within 12 levels) either fight - level, kit, party, luck; the loser is beaten
  (a defeat like any other, so it can be final), the winner bruised, a kill on the record - or the weaker, if cautious
  and clearly outmatched, keeps their distance. Both remember the other (grudges, -90 for the beaten); the nearest town
  hears of it. Revenge: the beaten (bold ones, by bravery) swear to pay back the one who beat them - they hunt that ground
  again, a meeting is then certain and no one walks away; won = revenge had, the foe gone = grudge let go, 80 kills there
  without finding them = given up. Test on a copy (48 h): at 35% there were 45 clashes (80% of all defeats, 2 deaths) -
  lowered to 15%. Crossings not yet seen in the log (arrivals aren't kept).
  **Roads and you, Nov 17 (01:10):** clashes on the roads too - someone of the other side travelling a road within 300 yd
  of yours (6% by day, 12% at night; the grudge-bound always stop); test copy at 12%: 22 road clashes in 48 h, halved.
  Enemy bodies near you were already hostile (NPCBots wanderers carry their race's faction and attack any hostile in
  reach, players included; NpcBot.PvP = 1) and fight each other for real, their deaths reaching the sim. New: a sim
  adventurer killed by you (or your pet or companion) remembers it (-90: no hiring, cold words), and when you're of the
  other side the news names you as the Horde/Alliance; /invite never reaches the other side's people.
  Still open: World-PvP gear drops (looting a piece off a sim body - needs in-game testing of NPCBot corpse loot), sim
  people seeking out a player who killed them, the Horde side's war news.
- Stage 1, a two-sided world: both factions' towns (side from guard/service hostility), Horde zones and races (orc,
  troll, tauren, undead, blood elf; goblins neutral), Horde townsfolk looks; danger-aware routes (enemy towns and
  contested roads cost more; the brave or vengeful take them); clashes resolved like threats, feeding grudges,
  revenge, advice, missing-person requests; enemy bodies near you are hostile and fight for real. A fresh start fits.
- Stage 2, the war layer: bases (towns, keeps, forward camps that are set up, held, lost), patrols between bases,
  zone control (Ashenvale, Barrens, Hillsbrad, Arathi, STV...), raids on outposts (refugees), war bounties.
- Stage 3, you in the war: standing with each side, recruitment by what you've done for them, your deeds in the news.
- Third map (530): Outland + draenei + blood elf lands; boats/portals.
- Tauren Plainsrunning (as in WoW Forever, player-verified): +2% speed per stack, a stack per 5 s of running by
  distance, max 30 (+60%), -1 stack per second standing still, -6 stacks per hit, works everywhere incl. instances,
  doesn't stack with other speed effects (stronger wins), no mounted bonus, no level requirement; numbers as settings.
  Client buff (reuse the beta Plainsrunning spell if it's still in 3.3.5's data). Tauren bots/townsfolk get it too.
  **Done Nov 16 (late) for the player, not yet seen in game:** spell 12821 "Plainsrunning" is still in the client
  (name + icon, no tooltip), reshaped server-side into a stacking MOD_INCREASE_SPEED aura (strongest speed effect wins,
  ignored mounted); a stack per 35 yd run (not swimming), stacks freeze while mounted/flying/dead; settings
  SoloWoW.Plainsrunning.*. Still to do: tauren bots and townsfolk, a tooltip (client patch).

## 4. Wildlife
- **Done Nov 16 (night), not yet seen in game:** hunting pressure per ground (sim_ground): sim kills add to it, it
  decays 40% a day; a hard-hunted ground yields as little as 35% and adventurers drift elsewhere. Neglect: a ground near
  a town unhunted for 7+ days can spill a band onto the nearest road (a camp caused by something, with its bounty).
  Herds (grazers by name: deer, boar, kodo, zhevra, plainstrider...) graze between beast grounds within 1500 yd every
  3 h, staying put in rain; predator packs (wolves, cats, bears, raptors...) follow the nearest herd by day and take
  one when they catch it, and go down to the roads at night (a road hazard mostly after dark), with a bounty that
  follows them. Up to 20 herds / 10 packs per continent; they appear as real creatures near you and walk to where the
  sim moves them. GM: .sim wildlife.
- **Thinning, done Nov 16 (late), not yet seen in game:** the game's own spawns feel the hunting. A normal creature
  killed on a hunted ground (anyone's kill) stays dead longer, up to ~3x its respawn (max +30 min) on the hardest-hunted
  grounds; the player's and their companions' kills count as hunting too (and stop a ground counting as neglected).
  Grid unloads forget the longer timer (respawn as normal then).
- Sim-owned, put in the world near you like camps: herds moving between grounds, predators following; real culling
  (over-hunted grounds go quiet, neglected ones swarm and spill onto roads); night prowlers, sheltering in storms;
  reclaiming cleared camps and emptied towns; untended dens escalating.

## 5. Adventurers and companions
- **Done Nov 16 (night), not yet seen in game:** `/invite <name>` or inviting your target reaches sim adventurers
  (GroupHandler hook -> BotBubble::InviteByName): near you (60 yd) they join free (NpcBot.Cost.Hire = 0) or whisper why
  not (HireRefusal); far away, the one who knows you best whispers where they are; at the NPCBots cap they say so.
  Fighting beside you builds regard (SharedKill, once an hour); dying while with you costs it (-40, -15 in dungeons).
  Removing a sim companion from the party dismisses them (GroupHandler -> BotBubble::DismissFromParty). Still to do:
  neutral people coming when plans match, remove/keep the gossip hire option.
- **Persistent gear, done Nov 16 (night), not yet seen in game:** NPCBots already generates wanderer gear by class and
  level with level-gated slots (rings/neck/shoulders from 20, head/trinkets from 30) - correction to an earlier note.
  Now a sim body wears the kit it had last time (sim_agent.kit "slot:entry:suffix:enchant,..."), each level gained
  marks a piece due (kit_due, max 6) and the weakest due pieces are rerolled at the next spawn; anything equipped on a
  sim body (your gifts, their finds) is recorded. Still to do: the sim's fight odds from the real kit (gear tier is
  still abstract), quality curve and set coherence on top of NPCBots' generator.
- **Generated loot for the player (the user's pick, Nov 17 - next before PvP drops; noted, not started):** WoW's own
  items only - real items of the right level, slot and class use, each drop rolling its own random suffix/property
  ("of the Bear", "of the Monkey"...) from the client's ItemRandomSuffix/ItemRandomProperties data. No new item IDs, no
  client patch, never an invented stat formula (DESIGN.md "Loot", Nov 15). Comes from:
  1. generated quest rewards, by the quest's difficulty (stakes ladder band, grey never);
  2. dungeons once they're re-infested (the sim-taken old places);
  3. rarely, at random, from enemies of every kind - on top of the normal static loot, at world-drop-like odds.
  Ordinary items roll too (the user, Nov 17): any armour/weapon of the right level - not just WoW's own random-suffix
  items - can come with a rolled suffix, its size from WoW's suffix tables for that item's level, slot and quality.
  The aim: loot exciting again, like Diablo. Rolled suffixes on WoW's items are the safe first step; truly generated
  items (own names, stat rolls, item levels, made from existing models/icons) are the likely direction after it.
  New named uniques (option B) stay parked until then.
- **World-PvP gear drops (the user's design):** when someone is killed in world PvP (never in battlegrounds or
  Wintergrasp), there's a chance they drop one random piece of their equipped gear, which someone else can loot.
  **Sim people's gear only - never the player's.** Fits the war track (enemy bodies near you, sim clashes off-screen).
- Party-invite hiring (like playerbots), no gold: the sim decides (regard, level gap, caution, own party); answers
  as whispers; neutral people come if your plans match theirs. Kicking = dismissing. Gossip hire removed or free.
- Persistent gear: generated from class/level/luck/history, only exceptions stored (gifts, notable drops, heirlooms:
  item entry + suffix + enchant), real items only on bodies near you, item index built at boot. No DB growth.
- Visual progression rules: only the game's items by level band/class/armour type; piece-by-piece upgrades a few
  levels behind; slots fill in over levels (shoulders!); quality curve (white/green -> blue -> rare epics);
  coherence grows with level; class identity (shields, staves, bows, dual wield); mounts at the riding thresholds;
  townsfolk in work clothes, never adventuring armour.
- **Consumables v1, done Nov 16 (late), not yet seen in game:** healing potions per person (sim_agent.potions):
  bought when trading in town (2-6 by caution, at most a quarter of their purse, price grows with level), drunk off
  screen when a fight turns bad (wounds 75+ -> -40, so a carried potion saves them from a bad pull), and by bodies in
  front of you only while they have one (NPCBots' potion use now draws on the count; mana potions not counted yet).
  The older "supplies" meter stays as food/bandages. Still to do: mana potions, food/drink by count, elixirs, gifts.
- Consumables: counts per person (food, drink, healing/mana potions, bandages, later elixirs/scrolls), bought by
  level in towns (economy), used at crucial moments by bodies (player cooldowns, level-appropriate potions), for
  everyone incl. enemy war groups; gifts add to the stock.
- Bond: grows with time together (kills, bounties, close calls, rescues); close companions fight smarter, know more,
  bring friends, pace their levelling to stay near yours, ask for help with their ambitions, mail you; dropping
  people costs regard (with them and their friends).
- Friends list like playerbots: sim people as fake player entries (reserved ids): add friend, see level/zone,
  whisper (templated replies), they whisper you, invite from afar (they travel to meet you or say why not).
- Population: thousands as factions/map 530 arrive; keep enough people near the player's level in their region;
  after a death, their friends/kin/party are the next people you meet.
- Death that matters: usually retreat and recover; permanent death an event (notice, family requests, grief/blame,
  gear lost or brought back).

## 6. Difficulty and pacing
- **XP bands, built Nov 16 (late), ON since Nov 17 (the user approved the numbers):** SoloWoW.XPBands.Enable = 1 replaces the flat Rate.XP.*
  (0.5) for players, and sim adventurers level on the same curve. Default bands (kill/quest):
  1-10 1.0/0.85, 10-20 0.94/0.75, 20-30 0.85/0.65, 30-60 0.82/0.62, 60-80 1.0/1.0 - the kill numbers are this server's
  XP table as a share of vanilla's (vanilla needed 4.3M XP to 60, this table 3.38M), so grinding takes vanilla's time;
  quests (boosted after vanilla) are trimmed harder. Pure-grinding estimate at 60 same-level kills an hour:
  | band  | vanilla | now (flat 0.5x) | band curve |
  | 1-10  | 6 h     | 12 h            | 6 h        |
  | 10-20 | 19 h    | 37 h            | 19 h       |
  | 20-30 | 33 h    | 56 h            | 33 h       |
  | 30-40 | 50 h    | 82 h            | 50 h       |
  | 40-50 | 71 h    | 117 h           | 71 h       |
  | 50-60 | 93 h    | 153 h           | 93 h       |
  | 60-70 | -       | 280 h           | 140 h      |
  | 70-80 | -       | 559 h           | 279 h      |
  Real play mixes in quests (roughly half of vanilla levelling), so expect well under these. Tune in SoloWoW.XPBands.
- Levelling curve by band (replaces the flat 0.5x): multipliers from this server's XP table vs vanilla's, quests
  trimmed more than kills in 20-60; 60-70 TBC pace; 70-80 native. Show hours per band before switching on. Same
  curve for sim adventurers and companions.
- Vanilla difficulty beyond XP: tougher mobs, slower regen, scarcer gear (to design).
- **Revert the WotLK-era easing of vanilla and TBC content (the user, Nov 17 - VERY important to them).** 3.3.5 is not
  how levels 1-70 played: patches 3.0-3.3 (and some late TBC ones) made the old world and Outland much easier. Goal:
  1-60 plays like vanilla, 60-70 like TBC, Northrend as is. Method: go through the official patch notes 3.0.2 -> 3.3.5
  (and 2.3/2.4 for the older easing), list every change that touched 1-70 difficulty, and revert each one that's data
  or config (world DB, DBC, spell fixes, settings) - reversible, like the Era preset (originals kept in backup tables,
  -Off restores). Nothing below is confirmed yet; verify each against the patch notes before changing anything:
  - mob strength in old content: outdoor elites and group quests made non-elite/soloable, lower-level dungeon mob
    health/damage cut, dungeon/raid entry requirements (attunements, keys) removed;
  - levelling speed: XP to level cut for 20-60 and 60-70, quest XP and gold raised (the XP bands are part of this - check
    their numbers against the real pre-3.0 tables);
  - travel and comfort: riding levels/costs (done in Era), flight paths added, faster mounts, things that made the
    world smaller;
  - the player's side: class/talent power at low levels (the 3.0 talent redesign, mana and regen, ammo/reagents,
    heirlooms) - the hardest and riskiest to revert; decide item by item, some may stay.
  Where reverting needs code (e.g. a class change), it goes in as its own opt-in setting. Fits with the vanilla
  difficulty line above and the XP bands; together they become the Era preset's full scope.

## 7. Memory and performance
- **Server manager (the user's ask, Nov 17):** the server shutdown doesn't really work (the launcher leaves MySQL and
  servers in odd states; the PC felt slow after many restarts). Move to a launcher that manages the servers properly,
  like D:\AzerothManager: start/stop/restart each process, status, logs, clean shutdown with save, no orphans.
- Grid unloading: built, not yet seen in play (watch for "let go of N idle grids"). MySQL buffer pool 128M (done).
- Load spawn data only for the player's continent; fold authserver into worldserver; target ~1 GB idle, < 2 GB playing.

## 8. Endgame and expansion gates
- Level inhibitor at 60 and 70 (the no-XP flag the Experience Eliminator uses; rested XP still builds); a reused rare
  item (runtime-overridden, scripted) lifts it; drops from the last raid boss of the expansion (C'Thun/AQ40, Kil'jaeden/
  Sunwell) with bad-luck protection; Outland (by zone - map 530 also holds draenei/blood elf lands) and Northrend
  (boats/zeppelins/map 571) closed until unlocked.
- The world opens with you: the first unlock opens the Portal for the sim too (adventurers head to Outland over time).
- NPC gating: sim adventurers stop at 60/70 for a varying while; everyone in the raid when you get the item unlocks
  with you; the rest unlock on their own later; raid companies can earn it themselves.
- Raids with bots: an encounter layer per boss (Playerbots' boss knowledge, our code) + a generic fallback (out of
  ground effects, interrupts, facing, tank-first heals, dispels, adds). Three kinds of mechanics: movement/position,
  timing/roles, "do a thing" (Magtheridon cubes, Razorgore orb, Kael's weapons, Vashj cores, chess, Teron, KJ orbs) -
  bots do it via script or the player does the key part; last resort a small adjustment for bot raids. AQ40 and
  Sunwell first. Check what Playerbots covers (reference\mod-playerbots).
- Off-screen raid companies of sim adventurers (news, deaths, gear).

## 9. After 80
- Living economy: crafters, caravans (escort/rob/invest), prices that react to roads, camps and war.
- The war as the endless loop (sieges, capitals, faction leaders, seasons).
- Old places made dangerous: classic dungeons/raids scaled up and taken over by sim-owned bosses, dropping generated loot.
- The world escalates: ignored threats gather warbands, invasions, towns fall and are retaken.
- Your settlement -> player housing (your own house in your settlement or a claimed one in a town).
- Marriage and family: a spouse who trusts you deeply; children born into the world (a year per game week), growing up
  over a few real months, living their own lives.
- Legacy: retire a character into the world (they live on with family and friends); play the next one - even your
  own grown child - with your old companions as the veterans.

## 10. Client
- Now: stock 3.3.5 client + DXVK + ReShade (incl. screen-space GI) + HD assets via our patch pipeline + LAA patch.
  RTX Remix only as an experiment.
- Candidate: the WoWee-based client with DXR ray tracing and a custom denoiser (was on wotlk.net - placeholder now).
  Check: licence, activity, Windows-only DXR vs mobile, our server (login, bubble, NPCBots gossip/hire, party
  frames, boards, townsfolk menus, quest marks), long sessions, FPS on the 3060 Ti, addons.
- WoWee itself: watch-and-wait (smoothness, animations, terrain transitions, graphics features).
- Keep race/patch/asset tooling client-neutral.

## 11. Playable races (stock client)
- **Order (Nov 16, the user's call): Worgen, then Murloc.** Worgen from the public AzerothCore worgen module
  (its client patch + server side), adapted to our core. Murloc built by us, borrowing the Tauren client prefix for
  helmets (see RACES-SURVEY.md). Both must work as NPCs too: our own CreatureDisplayInfoExtra looks in our patch,
  the race in the sim's race pools (townsfolk, adventurers, NPCBots), and faction/reputation so guards and
  townsfolk treat them as their side.
- **Worgen, server side done Nov 16 (late night):** heyitsbench/mod-worgoblin (AGPL, same as AzerothCore) in
  core/modules/mod-worgoblin (not in our core patch - the installer must fetch it); its 30 DBCs copied into
  server/data/dbc (old ones in server/data/dbc-before-worgoblin); its SQL applied by the core updater (our copy uses
  REPLACE INTO so a re-run can't fail); Announce off. Worgen = race 12, start in Shadowglen (Teldrassil) with the night
  elves - what we wanted; Two Forms (human/worgen, wolf in combat) and Darkflight scripted. Goblin (race 9) comes
  with it, starting with the orcs in Durotar. Client: its patch folder packed as Data/patch-W.MPQ
  (scripts/make-mpq.py). Still needed: Wow.exe with the interface signature check removed (the module's
  recommended patcher, run by the player on the clean 12340 exe) so the new character-creation screen loads; then
  worgen NPCs (NPCBots' race tables stop at 11; looks; sim race pools) and the class list (the module also allows
  Death Knight).
- **Murloc v1, built Nov 17 (just after midnight), not yet seen in game:** scripts/races/murloc.py builds our own
  tables + client patch (Data/patch-X.MPQ, loads after patch-W) from the stock client and the module's glue screens;
  scripts/races/murloc.sql the server rows. Race 15 (the stock NPC "Skeleton" slot - only ~25 NPC looks use it; they
  may look wrong now), Horde, Orcish, Forsaken reputations (every Horde-wide skill/language/reputation row widened to
  include murlocs). Body: the game's creature murloc (weapons in hand), skin colours = the five murloc skins; armour
  counts but isn't drawn (solosim_races.cpp hides non-weapon visible slots, so the character list hides it too).
  Classes Warrior/Hunter/Rogue/Priest/Shaman/Mage, Undead starting kits. Racials for now from stock spells:
  Underwater Breathing, Nature Resistance, Quickness (dodge). Start: east shore of Brightwater Lake, Tirisfal (the
  clearest spot, ~60 yd from the nearest hostile; mobs there are 6-9, so head to Brill/Deathknell). Creation screen:
  a 13th race button, Undead icon/background for now. Unknowns to test: whether the stock exe lists a 13th race,
  animations missing from the creature model (jump, emotes, sitting), the camera. Next: our own racial spells, an icon,
  murloc NPCs (friendly murloc townsfolk), the 25 skeleton looks.
- **Murloc in game, Nov 17:** works - creation screen, entering the world, fighting. Fixed on the way: the skin is now
  baked into the model (the composited character skin came out blank), full customization grids (switching races
  crashed the client on missing entries), clothes left out of the creation preview, sheath points added on the back
  and hips. Start moved to Deathknell (Brightwater Lake was ringed by level 5-7s). Still: no jump/sit/emote
  animations (the creature model has none - needs animation merging from the murloc costume model or new ones), skin
  colours (one model per colour, or a working composite), our own racials and icon, the customization rows that do
  nothing, unsheathing outside combat to check.
- Classes (the user's convention, Nov 16): follow what Reforged allows in play, then adjust by each race's lore
  and nature as we go. Murloc: Warrior, Hunter, Rogue, Priest, Shaman, Mage (no Paladin, Death Knight, Warlock,
  Druid - as seen in their game; their file tables still list Warlock, so their server trims it). Worgen:
  Warrior, Hunter, Rogue, Priest, Mage, Warlock, Druid (their list; Cataclysm also had Death Knight - decide later).
- Start zones (proposed, not yet confirmed): Worgen in Teldrassil (Gilneas refugees taken in by the night elves;
  3.3.5's Gilneas terrain isn't playable) unless the module's own start is better; Murloc on Tirisfal's north coast
  (low-level murloc camps by Brill, matches Reforged's undead/troll starter kits) as Horde, or a neutral murloc
  village if we want them outside the war.
- Later, after Worgen and Murloc: **Furbolg** and **Naga**. Furbolg can start in the draenei area - the Stillpine
  furbolgs of Azuremyst Isle (Stillpine Hold) are already the draenei's allies there (needs map 530). Naga: Horde,
  starting in Eversong Woods (the user's pick) - Highborne kin of the blood elves, and Vashj's naga stood with
  Kael'thas; the Eversong coast suits them (also map 530).
- Murloc (and naga): understand how an existing custom-race client does it (survey their patch, diff tables against
  stock, find what their Wow.exe changes and why), then build our own from scripts - table edits, model conversion
  (creature model -> character model), creation screen, an exe patcher written from that understanding and public
  docs - never copying their files. Server: race in core + world DB (start zone, stats, racials, faction, language).
- Sooner, in the sim: murloc and naga people with their creature models (Winterfin Retreat is a neutral murloc town),
  naga as a faction or threat.

## 12. VERY LAST - only when everything above is done and dusted (the user, Nov 17)
- **Experiment: host the world for other players over Spacewar** (Steam's free test app, ID 480, which hobby projects
  use for Steam networking), so friends can join this private build without port forwarding or a public server: the
  host runs the servers, others' clients reach them through a Steam relay tunnel (the launcher on both ends wraps the
  auth/world connections). An experiment, not a promise. Ties in with DESIGN.md's day-one rule (no single-player
  assumptions in sim code: bubble = all players' bubbles, memory/war/reputation/hiring per player) - things to check
  then: everything keyed to "the player" (FindLocalPlayer, catch-up while away, bounties/requests, the bubble cap),
  bots and sim bodies with several players, latency through the relay, and Steam's terms for app 480.

## Notes
- "How dynamic is it" assessment (Nov 16): alive = adventurers' lives, consequences, the player's deeds, townsfolk,
  markets; templated = all text, 7 request kinds, 5 bounty kinds, threats/camps on dice timers; abstract = off-screen
  fights and travel; static = places come from spawns, normal monsters are the game's, economy is raw goods only.
- Not yet seen in game: escorts, stolen items, old-foe requests, camps, threats in a fight, board notices,
  "have you seen", advice, grid unloading, retirements, townsfolk generations, the launcher's new settings.
