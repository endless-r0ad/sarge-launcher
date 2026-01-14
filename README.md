# Sarge Launcher


Sarge Launcher is a custom frontend for Quake 3 Arena and Q3A mods.
All features below are applicable to Quake 3, CPMA, Q3 Urban Terror, OpenArena, Defrag, Excessive Plus, and OSP mods.
The point is to give a consolidated experience for idtech 3 games, specifically those that are advertised on the Q3 master server.

## Features
- Level Browser - search for all levels your Q3 client can see, no limit
- Single Player - setup and launch with gamemode, difficulty, bot selection, etc
- Server Browser - Browse and join servers, favorite and trash servers, add custom servers
- Master Servers - toggle only the Q3 master servers you need including OpenArena and Urban Terror master servers
- Demo Browser - browse and launch your Q3 demos. Get additional details from .dm_66-.dm_68 demos
- Client Management - toggle relaunching, autoclose/loop demo playback, override client fs_gamename (launching OpenArena as defrag for example)
- and more!

![](./docs/static/sarge-launcher.gif)

## Development Setup

FYI the commit history here begins at the start of a rewrite. Tauri v1 -> v2, vue js options -> vue ts composition

### install all [Tauri v2](https://v2.tauri.app/start/prerequisites/) prerequisites for your system

### Install node_modules using nodejs lts/iron (v20.19.2)
```
npm install
```

### Compiles and hot-reloads for development
```
npm run tauri dev
```

### Compiles and minifies for production
```
npm run tauri build
```

Thanks to everyone who helped test, as well as [mwvdev/q3demo](https://github.com/mwvdev/q3demo) which I used as a reference
