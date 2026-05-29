# Dungeon Crawler

An attempt at a basic, customizable dungeon crawling game. It has a map system
already with a great number of sprites, as well as player and camera movement.
Built with [Bevy](https://bevyengine.org/) 0.17.

## Building & Running

This project uses a git submodule (`positioning`), so clone with submodules:

```bash
git clone --recurse-submodules <repo-url>
# or, if you already cloned without submodules:
git submodule update --init --recursive
```

### Native

```bash
cargo run            # debug
cargo run --release  # release (recommended for smooth play)
```

### Web (WASM)

The repository ships a prebuilt `web-build/` directory (which may back a live
deployment, so don't delete it). To regenerate it you need the
`wasm32-unknown-unknown` target and `wasm-bindgen-cli`:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli

# Build the wasm binary and regenerate the JS/WASM bindings in web-build/
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --out-dir ./web-build --target web \
    ./target/wasm32-unknown-unknown/release/dungeon-crawler.wasm
# (the bundled `build-for-web` script runs the equivalent commands)
```

Then serve the repository root over HTTP and open `index.html`, which loads the
module from `web-build/`:

```bash
python3 -m http.server 8000
# visit http://localhost:8000/index.html
```

## Controls

Menu (press a key to start a level):

| Key | Action |
| --- | ------ |
| `U` | Start the "unbeatable" combat level |
| `V` | Start the "avoidance" level |
| `P` | Start a procedurally generated roguelike run |

In game:

| Key | Action |
| --- | ------ |
| `W` `A` `S` `D` | Move the player |
| `Q` / `E` | Move down / up a floor (z-level) |
| Mouse | Target the adjacent enemy nearest the cursor |
| Arrow keys | Pan the camera |
| `,` / `.` | Change the visible floor |
| `PageUp` / `PageDown` | Zoom in / out |
| `R` | Restart to the menu after Victory or Defeat |

## Mechanics

- **Procedural dungeons (`P`)**: 6-12 rooms connected by L-shaped corridors,
  with scattered enemies and health pickups. Victory is reaching the room
  farthest from the start.
- **Multi-floor progression**: clearing a procedural floor descends to a fresh,
  harder floor (enemies scale with depth). Your health and accumulated strength
  carry over (with a small heal on descent). After 8 floors the run is a win.
- **Weapon drops**: slain enemies have a ~30% chance to drop a weapon. Walk over
  it to permanently boost your strength.
- **Fog of war**: tiles are only visible within line of sight; walls block sight.
  Explored tiles stay drawn, but enemies are hidden until they are in view.

## Gameplay

In this level, we have no strength, and cannot even withstand one attack, so
we must avoid the enemies and make it to the victory tile. In other levels, we
must kill all the enemies. In some levels, you can either kill all of the
enemies or make it to the tile, and sometimes you must do both. 

![Gameplay](/gameplay.gif)

The way combat works is that any enemy adjacent to you (up, down, left, right
of you) will deal damage to you every combat round proportional to their
strength. You will deal damage to a random enemy adjacent to you every combat
round proportional to your strength. Thus, the important aspect of combat is
to avoid being surrounded, as you will be taking more damage than you have to
if you fight every enemy individually.

## Future Steps

1. Map editor: this will allow me to much more easily construct scenarios and
   will allow users to do the same.
2. User interface improvements.
3. More weapon and item variety in the loot loop.
