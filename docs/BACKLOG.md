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

## 4. Wildlife
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
- Levelling curve by band (replaces the flat 0.5x): multipliers from this server's XP table vs vanilla's, quests
  trimmed more than kills in 20-60; 60-70 TBC pace; 70-80 native. Show hours per band before switching on. Same
  curve for sim adventurers and companions.
- Vanilla difficulty beyond XP: tougher mobs, slower regen, scarcer gear (to design).

## 7. Memory and performance
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
- Murloc (and naga): understand how an existing custom-race client does it (survey their patch, diff tables against
  stock, find what their Wow.exe changes and why), then build our own from scripts - table edits, model conversion
  (creature model -> character model), creation screen, an exe patcher written from that understanding and public
  docs - never copying their files. Server: race in core + world DB (start zone, stats, racials, faction, language).
- Sooner, in the sim: murloc and naga people with their creature models (Winterfin Retreat is a neutral murloc town),
  naga as a faction or threat.

## Notes
- "How dynamic is it" assessment (Nov 16): alive = adventurers' lives, consequences, the player's deeds, townsfolk,
  markets; templated = all text, 7 request kinds, 5 bounty kinds, threats/camps on dice timers; abstract = off-screen
  fights and travel; static = places come from spawns, normal monsters are the game's, economy is raw goods only.
- Not yet seen in game: escorts, stolen items, old-foe requests, camps, threats in a fight, board notices,
  "have you seen", advice, grid unloading, retirements, townsfolk generations, the launcher's new settings.
