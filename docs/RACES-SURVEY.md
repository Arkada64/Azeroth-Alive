# Custom races: what the Project Reforged client tells us (survey, Nov 16 2026)

Read-only survey of the Project Reforged launcher's staged client (about 45 GB). We read its tables to learn how they
did it. None of their files are copied into this repo, and none will be. Our races get built from our own tables and
models. The tools are in reference\reforged-survey (StormLib + small Python readers); the `reference/` folder is not
in git.

## What the client is
- An Ascension-derived 3.3.5a client: `patch-M` and the rest carry Ascension's tables (classless "Character
  Advancement", Hand of Fate...). Project Reforged's own work sits on top, mostly in `patch-Z`, `patch-ZZ` and
  `patch-ZZZ` (ZZZ is the newest), with the `ForeverUI` interface (WoW Forever).
- Their own exe (`Project Reforged.exe`) plus `Extensions.dll` (12 MB). The DLL reads extra tables (HD character
  sections and hair geosets, HelmetGeosetVisData, CreatureDisplayInfoGeosetData, UICameraAppearanceChrRaces), and it
  takes **table patches sent by the server at runtime**: SMSG_PATCH_SKILL_RACE_CLASS_INFO,
  SMSG_PATCH_HELMET_GEOSET_VIS_DATA, SMSG_PATCH_CREATURE_DISPLAY_INFO_GEOSET_DATA and SMSG_PATCH_UI_CAMERA_APPEARANCE_CHR_RACES.
  So part of each race (which classes it can be, how helmets hide hair) comes from their server, not the files. Some
  racial tooltips even say an effect is "awaiting server activation".
- The exe knows race IDs beyond the stock 11 ("unsupported race ID(%d)" in the race/class checks), so the stock
  Wow.exe needs patching for new playable races. That's the exe-patcher work in BACKLOG track 11.

## Their playable races (ChrRaces in patch-ZZZ)
| id | race | side | ClientPrefix | classes (CharBaseInfo) |
|----|------|------|--------------|------------------------|
| 9  | Goblin | Horde | Go | Warrior, Hunter, Rogue, Priest, Shaman, Mage, Warlock |
| 12 | Worgen | Alliance | Wo | Warrior, Hunter, Rogue, Priest, Mage, Warlock, Druid |
| 14 | Broken (draenei) | Horde | Br | Warrior, Hunter, Shaman, Mage, Warlock |
| 18 | Furbolg | Alliance | Fu | Warrior, Hunter, Shaman, Druid |
| 22 | Naga | Horde | Na | Warrior, Hunter, Priest, Shaman, Mage |
| 23 | Tuskarr | Alliance | Ts | Warrior, Hunter, Priest, Shaman, Mage |
| 24 | High Elf | Alliance | He | Warrior, Hunter, Rogue, Priest, Mage |
| 25 | Ogre | Horde | Og | Warrior, Hunter, Priest, Shaman, Mage, Warlock |
| 26 | Murloc | Horde | **Ta** (borrows Tauren's) | Warrior, Hunter, Rogue, Priest, Shaman, Mage, Warlock |
| 27 | Jinyu | Alliance | Jn | Warrior, Hunter, Rogue, Priest, Shaman, Mage |

Races 13, 16, 19, 20 and 33-63 are NPC-only appearance races, mostly Ascension's: Kul Tiran, Vulpera, Pandaren, Thin
Human, Fel Orc, Vrykul, Tuskarr (NPC), Forest/Ice/Drakkari/Leper Troll, Taunka, Earthen, "Race2" HD variants and
others. Languages: Common for the Alliance races (High Elves also get Thalassian), Orcish for the Horde ones.

## Racials (SkillLineAbility race masks + Spell.dbc; tooltips paraphrased)
- **Goblin** (Cataclysm's set): Time is Money (+1% attack/cast speed), Better Living Through Chemistry (+15 Alchemy),
  Pack Hobgoblin (bank access), Rocket Jump, Rocket Barrage, Best Deals Anywhere (best vendor discount).
- **Worgen** (Cataclysm's set): Viciousness (+crit), Aberration (Nature/Shadow resistance), Flayer (+Skinning, faster
  skinning), Darkflight (sprint).
- **Broken**: the Draenei set (Gift of the Naaru per class, Heroic Presence, Shadow Resistance).
- **Furbolg**: Ursine Fury (attack/cast speed burst), Thick Fur (2% less likely to be hit by Nature spells),
  Furbolg Endurance (+5% base health), Gift of the Grove (+15 Herbalism).
- **Naga** (two versions, the newer listed first): Tidal Surge (break roots/snares, brief immunity), Siren Song (AoE
  silence that also restores mana/rage/energy), Serpent Scales (Nature resistance), Gills (longer breath). Older:
  Blessing of the Tidestone (+2% haste), Sea Serpent (+100% swim speed, water breathing).
- **Tuskarr**: Harpoon Charge (AoE stun), Rime of the Ancient Mariner (Frost resistance), Fisherman's Patience
  (+3% Spirit), Harpooner (thrown weapon crit), Thick Fur (+5% armor and Frost resistance).
- **High Elf**: High-borne Authority (+1% damage/healing aura for the party), Advanced Civilization (+5 to crafting
  skills), Mana Well Infusion (restores 2% mana / 10 energy / 10 rage by class).
- **Ogre**: Enrage (+5% physical damage, +10% attack speed), Ogre Might (+3% Strength), Thick Skull (-10% stun
  duration), Mace Specialization (+5 Maces).
- **Murloc**: Amphibious (+swim speed, water breathing), Slippery Foe (dodge burst), Slippery (+dodge, +1% spell
  haste), Fishing Expert (+15 Fishing, less junk), Nature Resistance.
- **Jinyu**: Waterspeaker's Blessing (a heal over time), Riptide Escape (break roots/snares), Gift of the River
  (longer breath), Cold Currents (Frost resistance).
Each racial is a spell in their Spell.dbc (IDs 160100-161213 for the new ones). We would make our own spells: the
effects are ordinary aura types the server already supports (haste, stat %, resistances, skill bonuses, swim speed,
water breathing, stun reduction, dodge).

## How they handle gear
- **Helmets:** WoW picks a helmet's model by the race prefix + sex (`Helm_X_TaM.m2`). Reforged ships a full helmet set
  per new prefix, about 863 models each, for Na, Og, Ts, He, Fu (patch-ZZZ) and Br (patch-ZZ). Goblin and Worgen use
  Go/Wo sets. **Murloc sets its ClientPrefix to "Ta"** so it borrows every Tauren helmet with no new files. **Jinyu**
  (Jn) has no helmet models; presumably the server-patched HelmetGeosetVisData hides helmets or hair. Helmet models
  are copies re-fitted to each head; that's the bulk of the work per race.
- **Body armour:** the 3.3.5 armour textures are unisex per body region, painted onto the character model's geosets,
  so a new race's model needs the standard geosets and texture regions to wear gear. Their models live in
  `Character\Reforged<Race>\` (High Elf 1694 files, Naga 712, Ogre 436, Jinyu 137, Tuskarr 63, Murloc 23, Furbolg
  45 + an HD set; Broken 1530).
- **Starting gear** (CharStartOutfit): no new items. Every new race reuses another race's class starter kit: Naga and
  Ogre get the orc/troll kits, Tuskarr and High Elf the human ones, Murloc and Jinyu the undead/troll ones, Broken the
  draenei ones, Goblin and Worgen the Cataclysm starter items that are already in 3.3.5's item data.

## Starting zones
Not in the client: where a new character appears comes from the server (playercreateinfo). What the client hints at:
- **Worgen:** Gilneas, as a real zone on Eastern Kingdoms (area 5179 with ~30 subzones, levels 40-46) and Gilneas
  City as its own map (815).
- **High Elf:** "Thalassian Highlands" (area 5225) and "Alah'Thalas" (area 2040) on Eastern Kingdoms.
- **Furbolg:** maybe Hyjal ("Barkskin Village", "Barkskin Plateau" in zone 616), but those are marked level 60, so
  more likely an endgame area.
- Others: nothing dedicated found. Given the borrowed starter kits, they most likely start in an existing race's
  zone (Naga/Ogre with the orcs and trolls, Tuskarr/High Elf with the humans, Murloc/Jinyu with the undead and trolls).
- A custom map 1808 "Pilgrim's Isle" (Copper Town, Berrybuckle Farms, Harvest Hills) looks like a starting island,
  but its map entry isn't in the tables we read.

## As NPCs (what we need for townsfolk and NPCBots)
A humanoid NPC of a race needs that race's character model + CharSections in the client and a
CreatureDisplayInfoExtra look with DisplayRaceID = the race. In their tables: High Elf 1363 NPC looks, Goblin 1400,
Naga 243, Ogre 21, Tuskarr 7, Murloc 4; Worgen, Broken, Furbolg and Jinyu have none (players only). So **every race we
add must ship CreatureDisplayInfoExtra looks of our own** (our patch MPQ) for townsfolk/bots to use it, the same
mechanism our Looks() pool already reads (DisplayRaceID).

## What this means for us (plan, BACKLOG track 11)
1. Per race, server side: race ID in core (race masks, ChrRaces lookups), playercreateinfo (start spot, our choice:
   their homeland in our world, e.g. murlocs at Winterfin Retreat), stats, our own racial spells, faction, language.
2. Client side (our patch MPQ, our files): ChrRaces/CharBaseInfo/CharSections/CharHairGeosets rows, a character model
   with standard geosets, helmets (Murloc's trick of borrowing a prefix is the cheapest path), NPC looks
   (CreatureDisplayInfoExtra) for townsfolk and bots, plus the exe patch for race IDs above 11.
3. Cheapest first: **Murloc** (borrow the Tauren prefix for helmets, creature model as a base), then **High Elf**
   (borrow the Blood Elf model and prefix: almost no new art), then Naga.
