# Azeroth Alive (SoloWoW) - backlog

**North star:** you aren't the hero, you're someone living in the world - you play WoW's content, but the sim always
runs something new, so you feel small. A living fantasy world.

Tracks, roughly in the order we work them. Each step should be playable on its own. Done items move to ROADMAP.md.

## 0. Repo and install (Nov 16)
- GitHub: github.com/Arkada64/Azeroth-Alive (public). Never commit Blizzard files, client, databases, binaries, logs.
- One-click installer: installs our server + launcher and patches a client the player provides (realmlist, LAA,
  later our patch MPQ). Server binaries ship as release downloads, not in git. Core changes live as a patch over a
  pinned trickerer/AzerothCore-wotlk-with-NPCBots commit.

## 0b. The user's order, Nov 17 (night) - biggest first; as far as we get this week
1. **Difficulty revert (ours, not mod-individual-progression - that one is only a reference: MIT, its data and approach).**
   Patch 2.3 did most of the easing: XP needed 20-60 cut ~15-20%, quest XP 30-60 up, outdoor and dungeon-entrance elites
   made normal, elite quests soloable, old dungeon level ranges narrowed (SFK retuned), dungeon bosses' greens made blue.
   3.0.2: less XP 60-70. 3.1: creature armour cut globally. 3.2: riding at 20, 1.5 s mount cast. 3.3: no daze for 1-5
   and less for 6-10, cheaper low-level spells, better low-level regen. Wrath's talents and classes: far stronger players.
   The revert: vanilla's XP curve and quest XP (into the XP bands), elites and elite quests back, the old dungeon ranges,
   a power penalty for the player in vanilla/TBC content (damage, healing), creature armour as before 3.1, daze and early
   regen as they were, riding at 40/60 and the 3 s cast, old dungeon loot; the sim paced to match. Also: **where hostile
   creatures became neutral** (find and restore). To decide with the user: which comforts stay (later flight paths, the
   dungeon finder, heirlooms...).
   **Decided (Nov 17 22:30): keep only the later flight paths.** Dual spec, heirlooms, the barbershop and the cheap respecs go back to vanilla/TBC.
   **Done (Nov 17 night):** era-db.ps1 applies the vanilla/TBC facts (mod-individual-progression, pinned, fetched at
   install, allow-list only): factions - **the "hostile made neutral" were the starting areas' foes** (Defias Thugs,
   Frostmane, Scarlet converts, Night Web and Webwood spiders, Bristleback, Durotar's boars and demons, the Sunstrider and
   Ammen Vale foes: faction 7 since 3.x; nothing above level 7 was touched) - elite rank (outdoor elites on the old
   continents ~485 -> 768), levels, health/damage/armour, aggro range, speed, money; quest levels and objective counts;
   vanilla's XP curve (4.08M to 60); each quest's old XP (core hook); dual spec, heirlooms and barbers gone; mounts take
   3 s again; the player's side at 0.75 in vanilla content and 0.85 in TBC content (you, pets, NPCBots). XP bands now 1.0.
   **Also done:** generated quests pay XP by difficulty and distance - an ordinary quest of their level after the revert
   (median of real quests: 420 XP at 10, 1850 at 30, 3550 at 50, 6600 at 60) times the work (kills or goods against an
   ordinary quest's eight, a rare 1.8, a threat 3, a group bounty 1.5) and up to +60% for the way there; money stays the
   townsfolk's own (under a quest's, as decided Nov 17). Northrend measured: its elites are already there (8.5% of outdoor
   spawns, against 7.5% on the reverted old continents), so it gets what the revert gave the old world's ordinary
   creatures - x1.22 damage, x1.06 armour (1891 creatures, reversible).
   **Left:** **rares for Northrend** (28 there, against ~336 in the old world - the sim should field generated rares), old
   dungeon loot (boss greens/blues), regen (the reference is unsure of its values), respec prices (check), and a play test
   of the 0.75/0.85 power numbers.
   - **Generated quests reviewed with the revert:** requests and board bounties pay XP and money by difficulty and
     distance, matched to ordinary quests of the same level after the revert.
   - **Wrath content to the same rules:** review Northrend; where it lacks what vanilla/TBC had (outdoor elites, elite
     rares outside dungeons, group quests, real danger), apply the same formula - original difficulty for the first time.
2. **Outland and Northrend in the sim.** **Done (Nov 17 night):** 17 zones (Outland's eight, Northrend's nine: the TBC
   isles and Dalaran have no road nodes yet, Wintergrasp is a battlefield); 185 towns and outposts, ~1870 hunting
   grounds, ~610 adventurers at the land's level (Outland ~64, Northrend ~73), blood elves among the Horde there; the
   crossings - the Dark Portal, Stormwind-Valiance Keep, Menethil-Valgarde, Tirisfal-Vengeance Landing, Orgrimmar-
   Warsong Hold - with two-crossing trips (Outland to Kalimdor); levels to 80, threats up to 80. 263 crossings in a
   61-hour test; caravans in Outland. Dock towns cached (the first try ran 20x slower; now as fast as before).
   **Next there:** generated rares for Northrend; road nodes for the TBC isles (Eversong, Ghostlands, Azuremyst,
   Bloodmyst) and Dalaran so blood elves and draenei have homes; the Exodar boat and the Silvermoon orb.
3. **Full gear generation** (persistent gear for sim people; truly generated items for loot later).
   **Done (Nov 18, just after midnight):** every adventurer owns a real kit of the game's items, body or no body
   (solosim_gear.cpp), built with the rules their body uses (bot_ai::CanEquipFor - _canEquip's class/level/slot rules,
   split out so the sim can call them - and NPCBots' sorted gear lists): quality by level band, matching armour names
   from 30, slots opening with level, no ring or trinket twice. Level-ups, rare finds, savings spent and crafted gear
   mark pieces due; an hourly pass (400 people) replaces the weakest due pieces where something better fits, round the
   world in turn - not anyone whose body was near the player lately (the body wears and records the kit). The kit sets
   the gear tier the fights use, now against a normal item for the level (Outland's and Northrend's item levels too).
   99% of adventurers had a kit after a 61-hour test; 48 sim-hours still run in ~17 s.
   **Left:** purchases from town vendors and the AH (with the market), dungeon and raid loot (with that sim), truly
   generated items.
4. **Auction houses and the market sim:** prices from the game's own values (vendor price, quality, level, slot,
   usefulness), moved by supply and demand from the sim (robbed caravans, quiet mines, farmed areas), drifting over days;
   adventurers list what they don't want like a player (greys still to vendors), at prices by character (greedy high,
   impatient undercut); your auctions sell over time near the market, slowly or not at all above it, expire if you don't
   follow it; sim people buy what they need; Alliance, Horde and the neutral goblin houses are separate markets.
   **Auction houses done (Nov 18, ~1 am; solosim_market.cpp):** the sim's adventurers put up what they take off when
   their kits improve (greens and better; whites and greys to a vendor) under their own names (the friend guids), in
   their side's house or a goblin one, priced from the item's own value (vendor price by quality, kind, suffix) times
   supply and demand (listed of that kind and level in the house against adventurers of the side at that level), the
   greedy higher; a few hundred per house at most. Every five minutes each auction may sell: near the market within
   hours, rarely above, never at 2.5x; buyers are real adventurers of that side and level with the coin (paid from their
   purse); your auctions too (your money by the usual mail). Unsold sim listings go; sellers are paid through the
   auction-successful hook. Tested: 277 listings in three houses, trades, clean up. Not yet seen: a sim buyer for one
   of the player's auctions (same core steps as a player buyout). Later: buyers wear what they buy; raw materials
   from the gatherers and caravans; prices drifting over days.
   **Dungeons and raids done (Nov 18, ~1 am; solosim_dungeons.cpp):** 48 dungeons and 22 raids from the game's own
   teleports, levels from the creatures inside, bosses (rank boss / dungeon-boss flag) and their real loot tables (332;
   our own roller of the loot tables: groups, references, chances). Every six sim-hours some parties of one side at the
   level gather at the town by the entrance and go in (no body while inside); raid companies of the raid's size now and
   then. Boss by boss against levels, kits and numbers: XP and coin per boss, or a wipe (wounds; a defeat for some in a
   dungeon, never in a raid). Rolls: need (fits the class and beats their piece), greed (character or coin), pass; the
   need winner wears it (what came off to the AH or a vendor), a greed winner sells it (bound - to a vendor). The
   chronicle tells it ("... cleared Gundrak: Slad'ran dropped Cannibal's Legguards, won by ..."). 84 runs in 61 hours.
   **Left here:** rolls in the player's party (adventurer companions need/greed against you in a real dungeon;
   mercenaries never roll) - needs the group-loot hooks; boss order inside a raid (spawn order for now).
   **Dungeons and raids in the sim:** parties walk to the instance portal and disappear inside, simulated boss by boss
   (levels, gear, make-up) - loot or a wipe (deaths rarely final, as now); raid companies form off screen and roll for
   success per boss once the raid is unlocked (raid deaths never count). Loot rolls: need/greed/pass by class, armour type,
   upgrade or not, character; winners wear it (real items on their bodies), the rest to the AH or a vendor. Your party:
   adventurer companions roll too, weighing what you roll - they may need against you on a clear upgrade for their class
   (greedy and low-regard more, close companions less). **Mercenaries never roll** - all loot is yours.
5. **Launcher / installer:** create and manage accounts on the server (and the graphics step as an option - Reforged and
   especially LuxVelum are heavy: off, or on per part: HD models, LuxVelum, DXVK, Lexara).
6. **New civilian types:**
   - **Mercenaries:** hang about inns and towns, a few per town; hired for money; generated people at about the zone's
     band with their own gear, level and class; price by level (higher help in higher zones costs more); follow you for
     about a day, then must be hired again; no relationship (plain hired help); die for good; won't follow into other
     zones, never into raids, help in dungeons near their level; somewhere too far above them, a zone change, a raid, or
     the day running out - they hearth home and leave the party.
   - **Visible jobs:** fishermen fishing (docks, lakes, rivers), herbalists gathering for the alchemists, hunters and
     trappers after game (hides, meat), miners and prospectors, woodcutters, couriers, pilgrims and travelling traders.
     Adventurers do the same in the open to level professions. **Spread out** (a rule for the whole sim): people pushed to
     use every ground, fishing spot, herb patch and road - not crowding in one place, not crawling forever between places.
7. **Double name fields (WoW Forever) in 3.3.5**, roleplay naming rules enforced as usual.
8. **Bugs (Nov 17 night):** everyone is a retired adventurer; seated NPCs float above the seat; road signs without their posts.

Also noted the same evening:
- **Camping, part two:** the fire lasts about 10 minutes (Forever: 10 min, 5 min cooldown) and anyone can light one
  (Cooking makes it better); camp objects by profession beside it, a limited number per fire, each an hour's buff for
  everyone resting there (anvil: repair/armour; cauldron or mana well: mana; bandage kit: health; cooking pot: Well Fed;
  bedroll: rested XP; tanning rack or drums; lantern...); NPCs who carry them set them down at their camps too.
- **The text generator (decided: ours, not an AI model - within the templated-text rule):** what someone knows (memories,
  relations, news, what they saw, weather, danger, job, ambitions) + how they feel (mood that moves with events and decays
  to their temperament; traits colour it: neutral, happy, scared, confused, angry, grieving, bored...) -> what they want to
  say (gossip, complaint, boast, warning, question, reply, teasing; real exchanges over several turns) -> how they say it
  (grammar pieces, phrase banks; **race voice true to lore**: troll patter and "mon", dwarf brogue, orc bluntness, Forsaken
  grim wit, night elf old formality, gnome rapid tech-talk, goblin deals, tauren slow earthiness, draenei devotion, plain
  human; personality shading) -> delivery (/say, /yell, whisper by mood and distance; emotes /lol /cry /cheer /angry
  /cower /shrug /bow /thank...; body language). Generated from the sim's own facts - never contradicts the world.
  **First version done (Nov 18, ~2 am; solosim_talk.cpp):** moods from memories and state (content, happy, proud,
  grieving, angry, scared, hurt, tired, bored); acts picked by weight from real facts - town news (spoken without the
  chronicle's brackets; their own news told as "I..."), the danger near home, a dungeon run and what they won, a rare
  brought down, a friend lost, a beating or a clash with the other side, their ambition, a friend or rival in front of
  them - then the town's small talk when there's nothing of their own; eleven race voices (openers, agreement, surprise,
  comfort, praise, curses, fear, tags, greetings, scorn); personality shading; /yell for a proud boast or a warning by
  the bold; emotes (cry, cheer, flex, point, roar, wave, laugh, rude, no, yes); the listener's answer by what they know,
  the relation and the speaker's state. Used by townsfolk and by adventurers near you; GM .sim talk [n] prints samples.
  Also: generated names are checked against a list of hateful/crude words (the troll kit had made "Nazi") and existing
  ones renamed on load. **Next:** longer exchanges (three or four turns), questions to the player, whispers, more
  phrase banks per voice, the gossip/menu Rumor() through Speakable too.

### Lighter world save (user, Nov 17 22:26 - fit in alongside other sim work)
- The 10-minute save rewrites every sim table (~4 MB, ~12 MB on disk with InnoDB overhead). Write only rows changed since the last save (dirty flags per agent/table; events are append-only already).

### Bodies keep their spot across restarts (user, Nov 17 21:41)
- Today the plan/step survives a restart and downtime is caught up, but bodies respawn at the town anchor: 40 in a pile.
- Save each live body's position, facing, stand state and step every few minutes and at shutdown.
- On start, after catch-up: same step -> exact saved spot (same chair, already seated); new step -> straight to that step's spot; travellers placed along their road by trip progress.
- Stagger spawn-in over a few seconds.

## 1. Correctness (now)
- **The last batch, Nov 17 (06:55), not yet seen in game:**
  - Requests in the quest log: a pool of 1000 reserved quests (95000-95999, scripts/sim_quests.sql) filled at runtime
    (solosim_quests.cpp): the title ("Rockhide Boar Trouble", "The Road to Goldshire", "Word for Kela", "Missing: ...",
    "An Old Score: ...", "Stolen: ..."), the townsperson's own words, objectives the game counts (kills of the
    creature - found by name and level -, items to bring, or an event the sim marks: an escort arriving, word given,
    the missing found), a rough area on the world map and the way back, the request's money plus a little XP.
    Taking a request adds it; "remind me" adds it again if abandoned; finishing it with the townsperson completes and
    rewards it (items and money through the quest); a request that lapses leaves the log with a note. The quest id
    follows the request's id, so open ones are refilled at login. Board bounties are not quests yet.
  - Text: townsfolk talk ~3x the lines, by time of day, and about what really threatens their town (the nearest bandit
    camp, pack or roaming threat, by name and place); the townsperson spoken to answers; rumours open 8 ways;
    adventurers have more lines for every case plus the tired, the new (under 6) and the old hands (50+).
  - Adventurers using the world: idle in town, a third of the time they read the board, check the mailbox, sit on a
    free bench seat (one each) or stand by a fire - as the townsfolk do.
  - Fixes from the 06:32 playtest: boards only where you can walk from the town square, at street level (Goldshire's
    was up in the floors); townsfolk never move where there's no walking path (they "flew" to seats upstairs and
    such; a door out of reach just takes them in) (murloc race fixes here since removed);
    starting camps (Northshire, Coldridge, Aldrassil, Shadowglen, the Valley of Trials, Camp Narache, Deathknell) have
    8 young adventurers each (1-6) and most newcomers begin there (they stood empty: only inn towns had adventurers);
    the innkeepers' 153 letters were deleted (with the user's OK).

  - Requests/quests from NPCs into the real quest log, tracking progress and saying what to do - **next batch**: a pool
    of reserved quest ids filled at runtime from the sim (text, kill/item/event objectives, quest-log map points);
    taking a request adds the quest, finishing completes it (WDB cache is cleared at launch; ids are never reused in a
    session).
  - Mailbox filled by innkeepers - **fixed:** a rare's bounty wrote to every player the sim knew, from every town (153
    letters for Myrtha); now only towns of your side (or neutral) where someone living there thinks well of you, at most
    one letter every two days.
  - Wildlife - **fixed:** herds and packs stood still at their spot - they now graze/pace about it (random movement
    whenever they're not being moved). The "non-hostile boars" were right: Rockhide Boars are faction 189, the game's
    own neutral creatures - herds keep their real templates; bands and packs are the hostile ones. Group sizes vary
    now (the user): herds 1-2 (25%), 3-5 (50%), 6-9 (25%); predators alone 30%, pairs 30%, 3-4 30%, 5-6 10% - lone
    ones and pairs get no bounty or warning, packs of 5+ say to bring a friend; road camps 2-7; a pack's danger scales
    with its size.
  - Adventurer bodies' pace - **done:** they walk in town, when recovering, and on the road when tired (fatigue 60+),
    hurt (wounds 50+) or unhurried by nature (3 in 10); they run otherwise, and always run once a fight starts.
    Still to do: adventurers using the world (board, mailbox, forge, benches) like the townsfolk now do.
  - "The game really needs more text": more lines, more variety, everywhere (town talk, adventurer talk, requests,
    rumours, board notices, letters) - with the dialogue/text-generator decision above.
 - the user's list.** Done the same morning, not yet seen in game: errands pay
  lvl^2 x 3 + 40 copper (was x12 + 150), escorts x distance (800 yd = 1x, 0.5-3x) x danger (camps/threats on the way,
  up to 3x), old foes lvl^2 x 6 + 100; requests posted at 3.5% a tick (was 20%) and trimmed to one in seven townsfolk;
  rumours and board notices only from news worth passing on (deaths, defeats, clashes, rares, bounties, comings and
  goings, ambitions done - not socialising); an escorted townsperson settles where you took them (their new home, the
  body stays); townsfolk hold still while you talk (45 s, renewed by each choice, "Good day." releases); the talk
  animation now comes with words in /say when you're within 25 yd (templated small talk by job, or the town's news, at
  most once a minute each). The list:
  - Fixes (quick): townsfolk work pays far too much (11 silver to walk someone Stormwind -> Goldshire) - pay by level,
    danger and distance, money shouldn't come easily; far fewer townsfolk with requests (more had one than not);
    rumours: drop the "X shared stories with Y" kind - only what a listener would want to know; an escorted townsperson
    vanished after paying - they should stay and live in that area; townsfolk walk away while you talk to them - stop
    them; townsfolk play the talk animation but nothing shows in /say; bounty boards float (a wall board standing in
    the open) - a better model, or the same one placed against a wall where it makes sense. **Boards, Nov 17 (05:40):**
    hung on a wall near the town centre (solid behind at chest and knee height, room to stand in front, not a corner),
    flat against it and facing out; the old open spot only when no wall is found. Finding one (the user couldn't find Stormwind's): ask any
    townsperson "Where's the bounty board?", or type .bounty - a map flag at the nearest board within 800 yd; the log
    records where each board went up.
  - Generated quests/requests: a quest marker (map/minimap POI) where to go - vague is fine, an idea is the point.
    **Done Nov 17 (05:40), not yet seen in game:** taking a townsperson's request or a board bounty puts a red flag on
    your map and minimap (the guards' direction marker), named for what's there; "remind me" shows it again. Searches
    (thinning, stolen goods, old foes, bounties, the missing, someone to carry word to) are 40-90 yd off - roughly
    there; an escort's destination is exact; fetching has no place.
  - Townsfolk lives: walking onto railings and tables, several sitting in the same spot, random paths - robotic. Real
    places to go and things to do (stalls, benches, wells, doors, each other), one person per seat. **First pass Nov 17
    (05:40):** spots are found at street level (not roofs), never on something raised half a yard above the ground
    around it (tables, crates, railings) and never where someone already stands - for strolling and in front of
    vendors (inn seats were already one each). Still to do: real destinations (benches, wells, stalls, doors).
  - Embodied count: 40 bodies at a time feels empty (even at night); live WoW areas had hundreds - optimise toward
    hundreds (lighter bodies far away, sleepers indoors, townsfolk cheap). **Raised Nov 17 (the user's call, to test):**
    150 adventurer bodies + 100 townsfolk (150 in capitals), spawn/recycle 8/10 per check; configure.ps1 defaults and
    the launcher's sliders (now to 300/200) follow. Watch CPU/RAM in play and tune.
  - New world fills at once - **done Nov 17 (05:25), not yet seen in game:** every inn town has an adventurer
    population (14/18/24/30 by size, 120 per capital shared between its inns) and is filled to it at once - on a new
    world before its fortnight of history, and on load for an older one (this world: 770 -> 1636 adventurers, 3165
    people with the townsfolk; levels <10: 569, 10-29: 655, 30-49: 343, 40+: 226). Replacement: the dead and departed
    are still replaced one for one, but three in ten newcomers now arrive experienced, at the land's level, in the
    towns most short of hands (the rest set out at 1-6 from the starting towns); any other shortfall closes by a tenth
    a day (was one newcomer on 60% of days). Cost: 48 sim hours in 28 s (a week's catch-up after time away ~100 s).
  - Danger recognition (adventurers and townsfolk): leave areas, avoid roads or sneak through when a place is judged
    too dangerous (camps, threats, war, recent deaths) - builds on immigration's push and the route costs.
    **Done Nov 17 (05:55), not yet seen in game:** a danger map (every 3 sim hours): camps 1.5 / packs 1 within 700 yd,
    a threat 3 within 1500 yd, each death there in the last 3 days 1, and per side the other side's adventurers
    hunting there (0.7 each, up to 4). The careful pick safer grounds (by caution against bravery; avenging or revenge
    targets exempt), and the careful leave a home at danger 3+; on the road the careful slip past camps they can't beat
    and keep off the road when they see the other side first; farmhands and labourers stay in when danger at home is
    2+. With the world twice as full, clashes are at 6% (grounds) and 3%/6% (roads, day/night): a 48 h test gave 104
    clashes, 117 defeats, 17 deaths among ~1600 adventurers.
  - Crafters: people working the professions - goods, armour, weapons, potions - who gather or buy every ingredient,
    need the right place (forge and anvil, alchemy lab...) and sell what they make (shops, markets, the player).
  - Cooks: buy ingredients or hunt for them, cook, sell to taverns and shops (the inns' food comes from someone).
    **Crafters and cooks done Nov 17 (07:20), not yet seen in game:** every town gets a cook; a smith where there's
    an anvil or forge within 90 yd of the square; an alchemist and a leatherworker in towns of 10+. They work at their
    station (anvil, forge, fire) turning ore, herbs, hides and fish (cooks fish when they have nothing) into stock
    per town (sim_crafted: potions, food, arms, leather; capped at 60, food spoils fastest). Adventurers buy food,
    potions and gear from that stock - with no food in town supplies only reach 70%, no potions means none - and the
    shops and innkeeper shelve the town's potions and food (bought out, the stock drops). 48 h test: 143 crafters,
    stocks steady, adventurers carry ~0.8 potions. **Bounties as quests:** taking a board bounty puts it in your
    quest log (96000+) with its place on the map and progress; turn-in pays once. **Rain talk** only when it rains.
  - Adventurers: interact with the world around them and talk with each other (in /say, near you).
    **Talk done Nov 17 (05:50), not yet seen in game:** within 30 yd of you, now and then (one line every few seconds
    at most, a minute between lines each) an adventurer body speaks: to the sim person beside them by name - friends,
    rivals, the other side, the one they swore revenge on, townsfolk - or about their own day (revenge, avenging a
    friend, wounds, a bounty, saving gold, a place they want to see, the creatures they're hunting, rain, their class).
    Templated, filled with sim facts. **The war (the user, Nov 17):** the Alliance and the Horde are at war and share no
    tongue - the other side's people say nothing to each other; adventurers and townsfolk speak in their side's
    language (Common / Orcish), so the other side hears gibberish; no goodwill across sides - the sim never records a
    kind memory of a player from someone of the other side (only hostile ones), and older ones were forgotten the first
    time the bubble saw that player. Still to do: adventurers using the world (board, mail, forge) - their bodies move
    through NPCBots' own AI and the inn-seat code, a bigger job.
  - NPCs using the world around them (the user, Nov 17) - **townsfolk, done Nov 17 (05:55), not yet seen in game:**
    benches and chairs outside (one person per seat, benches by their slots, the chair's height), the bounty board
    (stand in front and read), mailboxes (a short visit), anvils and forges (labourers hammer there, one each), cooking
    fires (crouched by them). Idling picks one ~40% of the time; work sends labourers to the forge a third of the time;
    an evening out falls back to a bench when the inn is full; errands sometimes go by the mailbox.
  - Camping (from WoW Forever) brought to 3.3.5: a camp you set up to rest with your party, with benefits; sim people
    camp too, alone or together, between areas.
    **Done Nov 17 (13:10), not yet seen in game:** any Basic or Bright Campfire is a camp - near it you're resting
    (rested XP builds as at an inn; logging out there rests like an inn - core RestFlag REST_FLAG_AT_CAMP); sit by it a
    minute for an hour of Cozy Fire (Spirit 2 + level/5, half again for cooks with 75+), and a smith mends their gear,
    first aid sees to your wounds (SoloSim/solosim_camping.cpp). The sim's adventurers camp: out in the wild or at an
    outpost with no inn as bedtime comes, they make camp if the inn is further than they'd walk (2-12 min by
    character: the sociable and careful walk on, wanderers stop, rain makes it worse) and the place is safe enough; an
    evening by the fire (the sociable), then sleep - wounds heal 35 not 50, two points of food. Near you, a place's
    campers share one real Basic Campfire, sitting round it, asleep beside it, each in their own spot; campfire talk.
    48 h test at 2 am: 169 camping (153 fires, up to 5 at one) vs 1056 at inns. Still to come: camping on long
    journeys between areas (journeys are short at the sim's pace), your companions camping with you, camp objects.
  - **Longer trips, Nov 17 (13:30), not yet seen in game:** travelling pace 4.5 yd/s on foot (was 5.5: stretches of
    walking and jogging, stops), bodies on the road walk one three-minute stretch and jog the next; a journey that runs
    well past bedtime stops for the night on the way - an inn on the road that will have them, else a safe ground to
    camp at - and goes on in the morning; places worth seeing are really elsewhere now (2500 yd+, other continents,
    capitals three times as likely); a full pack and a greedy eye take the long road to a capital's auction house (a
    quarter more for the haul, up to 30-90 min away). Your companions sit down by your fire and get Cozy Fire too.
    48 h test: 158 places seen, 12 nights stopped at an inn on the way, 16 auction hauls, ~1000 camp nights vs ~14000
    at inns. Deaths up to ~21 per 61 h (12 of them in clashes: more road, more meetings - the war).
  - **Caravans, Nov 17 (13:50), not yet seen in game:** a town with plenty (60+ goods in its market or 40+ made things)
    sends half of it by wagon, in the morning, to a town of its side (or neutral) with less than half as much, 30 min
    to 10 h away at a wagon's pace (foot x 1.7). Two to four of the town's adventurers (of one side, within 15 min,
    who like coin or the road) come in and guard it; they walk beside it as one - the same legs, the same night at an
    inn or a camp on the way - and near you the lead guard has its pack animal (Pack Mule for the Alliance, Gizelton
    Caravan Kodo for the Horde). A bandit camp by the road tries it (60%): driven off (camp losses), or robbed (wagons
    emptied, guards hurt, the price on that camp up half again). At the end: goods into the market and shelves, guards
    paid (lvl^2 x 3 + road minutes x lvl / 2 + 1 s; a third if robbed). Caravans are news (rumours, boards). sim_caravan.
    48 h test: 18 set out, 17 arrived, 1 fought off, 1 robbed. The restless now move home further too (any fitting town
    up to four times as far).
  - **You and the caravans, Nov 17 (14:10), not yet seen in game:** when a caravan forms, a townsperson of the town
    (a trader if there is one) asks for one more guard - "Wagons for <town>", in your quest log, paid by the town's
    level and the road's length. Ride with them: the sim counts the road you share (any guard within 60 yd while the
    wagons roll); with them for at least half of it, the job's done - report to anyone in the town they were bound for
    (a third of the pay if they were robbed anyway). Hardly there: a letter says the trader won't pay. Bandits that
    fall on the wagons while you're with them come at you for real (2-4 of the camp's kind, at its level), your blade
    counts in the fight, and the camp's losses are credited to you. An enemy caravan: cut down every guard and the
    wagons are yours - coin and some of the goods (ore, herbs, leather, fish, potions, food at the region's tier); the
    road hears of it.
  - **Leading the wagons, and a price on raiders, Nov 17 (14:30), not yet seen in game:** (the user: no five-hour
    walks) once you've taken the job and you're by the guards, the caravan follows you at your pace (run; teleports up
    if you mount away) and you lead it - a map flag for where; reach the town (90 yd) with them and it rolls in: goods
    delivered, guards paid, your job done - report to anyone there. Gone another way for 10 minutes (or flying): they
    carry on without you and a letter says the trader won't pay. Each bandit camp you pass may try the wagons once -
    they come at you for real, and the fight costs the camp. Raiding an enemy caravan puts a price on you (heat 1-3,
    two days, then down a step a day): now and then (2 h / 80 / 50 min apart at most, one or two at a time, near your
    level - the robbed side's own adventurers who'd take the work, else newcomers drawn by the coin) hunters come for
    you - never in a town, at rest, in a dungeon, mid-fight or in the air; they lose the trail after 10 min of that and
    give up after 45. If they kill you the price is paid and it's over. GM tests: .sim caravan (one forms now in the
    nearest town), .sim hunters (a price on you, hunters on the next chance).
    Next: caravans at sea, your own wagon.
  - Dialogue: "might be time to consider a text generator" for natural talk about world events, life and rumours,
    with you and between NPCs. NOTE: the standing rule so far is templated NPC text only, no AI-generated text - the
    user is reopening that; options when we get there: a bigger procedural grammar (combinatorial templates fed by the
    sim's facts - still no AI) or a small local language model (cost: RAM/CPU on this PC, consistency, and it
    changes the rule). The user decides.
  - Graphics: Lumenveil (github.com/lumenveil/Lumenveil, GPL v2, no game assets; fog, light shafts, lamp lights,
    water, per-zone looks, longer view distance; copies files next to Wow.exe, hooks Direct3D in memory) into the
    installer (fetched, not vendored). Check its docs/INSTALL.md against our DXVK + ReShade setup and our MPQs first.
    "Could be used with Project Reforged models": their HD packs are their files - we don't ship or copy them.
    **Done Nov 17 (15:00), the user's pick: LuxVelum (Project Reforged's fork of CoAVolFog) rather than Lumenveil -
    both hook Direct3D, one at a time.** scripts/graphics/install-graphics.ps1 fetches the latest of each from its own
    project: St0ny's EXE patcher (preset Reforged - 4 GB, custom GlueXML, DLL loaders...) on the original Wow.exe,
    DXVK (Raz0r1337), LuxVelum + Reforged's preset, the 8-light shaders (Data\patch-5.MPQ\), Lexara HD fonts, and the
    Reforged packs A B C D E G (~15.5 GB; -Extras for M N P U S, -Interface for I, -SD). scripts/graphics/merge-dbc.py
    fits our patch-W to their models: 9 tables both carry rebuilt on Reforged's with our new races' rows added
    (Data\patch-Y.MPQ + the server's tables, originals in server\data\dbc-before-graphics); 8 helmet-geoset rows both
    changed stay Reforged's. Checked: the server boots and runs on the merged tables; the client starts and runs (45 s,
    LuxVelum hooked, DXVK in use) - not yet looked at in game. docs/GRAPHICS.md.

- **Audit fixes, done Nov 17 (05:10):** the sim covers the whole old world now - 17 more zones (Alterac, Hinterlands,
  Searing Gorge, Burning Steppes, Blasted Lands, Deadwind Pass, both Plaguelands | Thousand Needles, Feralas, Tanaris,
  Un'Goro, Silithus, Felwood, Winterspring, Azshara, Moonglade): 65 new towns and outposts (21 Horde), 1013 hunting
  grounds, 100 adventurers at their levels (settling now clamps to 60, not 50); highest 45 -> 60, 40+ from 14 to 99.
  Capitals: a table of capitals (centre, reach, people) files the Horde's towns under Orgrimmar (200 folk), the
  Undercity (140) and Thunder Bluff (100, had none) - 34 towns re-filed, 489 townsfolk moved in; the 70 bodies near
  you in capitals count the Horde's too. Hunting grounds: training dummies (no-XP) and ambient folk (civilian) left
  out; a ground's levels are its dominant creature's kind within 6 levels (no more 7-80). Wildlife caps 36 herds / 18
  packs per continent, threats roam grounds up to 60. Existing bad grounds stay until the world restart; new zones
  are clean. Backup before: db/sim-before-zones.sql.
- **Audit Nov 17 (01:55) - fix these (done above):**
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
  **Step one built Nov 17 (14:25), not yet seen in game** (SoloSim/solosim_loot.cpp, SoloWoW.Loot.Enable = 1): the
  game's own random suffixes ("of the Bear"...) on things that never had them - armour and weapons dropped with no
  random stats roll one (greens and blues 35%, whites 6% - a "magic" white at two thirds of a green's points); quest
  reward armour and weapons roll one (25% green+, 8% white; +5% a level the quest is over you, +15 elite or group,
  +20 dungeon or raid); any foe within six levels below you or above may carry a rare find - a world-drop green that
  rolls its own stats (0.6% normal, 2.5% elite, 10% rare or boss). Suffix pools are the groups the same kind of item
  already uses (56 groups, 57 kinds); 3188 world greens for levels 5-80. Core: GenerateEnchSuffixFactorAny (the
  suffix's strength for any armour or weapon; stock items unchanged). Still to come: re-infested dungeons as a source,
  truly generated items, PvP gear drops (sim gear only).
- **World-PvP gear drops (the user's design):** when someone is killed in world PvP (never in battlegrounds or
  Wintergrasp), there's a chance they drop one random piece of their equipped gear, which someone else can loot.
  **Sim people's gear only - never the player's.** Fits the war track (enemy bodies near you, sim clashes off-screen).
  **Built Nov 17 (14:40), not yet seen in game:** a sim person you (or your pet or companions) kill out in the world -
  a continent, never a battleground or Wintergrasp, someone hostile to you - drops one random piece of what they wear
  25% of the time: their body becomes lootable with that piece (its suffix kept), and they go a piece short (gear
  tier -4) until they replace it. Off-screen, a sim clash that ends in a defeat moves a little gear (up to 3 tiers)
  from the loser to the winner, a quarter of the time. Your own gear never drops.
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
- **Order (Nov 16, the user's call): Worgen, then Murloc** (custom races dropped Nov 17, see below). Worgen from the public AzerothCore worgen module
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
- **Custom races removed, Nov 17 (12:40, the user's call): Worgen and Goblin only.** Murloc (15), Naga (13), Tuskarr
  (17) and Furbolg (20) were built (murloc.py / newraces.py) but taken out: every playable race needs its own stock
  slot, and the NPC looks already in that slot then draw on the new body (the 37 Northrend Skeleton looks would have
  been furbolgs, the 25 Skeleton looks murlocs), on top of the extra work each body needed (animations, sheathing,
  skins). Removed: the characters (Roro), client patch-X.MPQ, the server tables (back to the worgoblin state), the
  world-DB rows and race-mask bits, solosim_races.cpp and the loot delay, the scripts (git history keeps them).
  The sim makes no people of these races either (only the stock creatures remain, as wildlife and threats).
- Worgen adventurers (Nov 17): 5% of the Alliance's newcomers, ~10% in night-elf lands, in their human form (human
  NPCBots bodies, human names - Gilneans) like the worgen townsfolk. Goblins stay townsfolk only: NPCBots has no goblin
  bodies. Death Knights stay (the user, Nov 17).

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
