# Core patch

Our changes to the NPCBots fork of AzerothCore, as a numbered series of patches over a pinned upstream commit.

- Upstream: https://github.com/trickerer/AzerothCore-wotlk-with-NPCBots (branch `npcbots_3.3.5`)
- Base commit: `c89fa107203f86a9b40adaf3bbfde2bec4b9805d` (Merge 3.3.5 to npcbots_3.3.5, 2026-11-14)

To build from source:

    git clone https://github.com/trickerer/AzerothCore-wotlk-with-NPCBots core
    cd core
    git checkout -b azeroth-alive c89fa107203f86a9b40adaf3bbfde2bec4b9805d
    git am ../core-patch/*.patch
    # then scripts\build.ps1 (Visual Studio 2022, CMake, MySQL 8.4 dev files, OpenSSL, Boost)

New code lives in its own files (src/server/game/SoloSim/*, AI/NpcBots/botbubble.*); hooks in upstream files are
small and marked `// SoloWoW`, so moving to a newer upstream commit is a rebase of this patch.