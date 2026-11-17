# Azeroth Alive

A single-player World of Warcraft 3.3.5a (Wrath of the Lich King) world that is alive whether or not you're playing.

You aren't the hero - you're someone living in the world. Adventurers and townsfolk have lives: homes, jobs, friends,
grudges, ambitions. They hunt, travel the real roads, take bounties, get hurt, die, retire, grow old and have children.
Roadside camps go up, named threats roam, and the towns talk about all of it. You play WoW's content, but the world
keeps running something new around you.

Built on [AzerothCore with NPCBots](https://github.com/trickerer/AzerothCore-wotlk-with-NPCBots). Local and offline:
one player, 127.0.0.1.

## What's in this repository
| Folder | What |
|---|---|
| `core-patch/` | Our changes to the core (the living-world sim, the bot bubble, townsfolk, boards, camps, threats), as a series of patches over a pinned upstream commit |
| `launcher/` | The launcher (Rust, egui): start/stop the world, play, settings, start fresh |
| `scripts/` | Build, configure, play/stop, reset, Era preset |
| `installer/` | The one-click installer (in progress) |
| `docs/` | Design, roadmap, backlog |

## What is NOT in this repository
No Blizzard files of any kind: no client, no game data, no extracted maps. You need your own 3.3.5a (build 12340)
client; the installer patches it locally. No databases or server binaries either - binaries come as release downloads.

## Install
In progress - see `docs/INSTALLER.md`. The goal: run one file, point it at your client, and get the server, the
launcher and a patched client.

## Status
Playable and in active development. See `docs/ROADMAP.md` for what's built and `docs/BACKLOG.md` for what's next.