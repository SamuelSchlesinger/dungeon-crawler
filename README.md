# Dungeon Crawler

An attempt at a basic, customizable dungeon crawling game. It has a map system
already with a great number of sprites, as well as player and camera movement.

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
2. Procedural generation of maps.
3. User interface improvements.
