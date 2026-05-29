use std::collections::BTreeSet;

use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

/// How far the player can see (in tiles, Chebyshev radius).
const SIGHT_RADIUS: i64 = 8;

/// Computes the player's line of sight for the current floor and records it.
///
/// Casts a Bresenham ray from the player to every tile within `SIGHT_RADIUS`.
/// A ray is blocked the moment it crosses an opaque tile (`passable == false`),
/// so walls cast shadows. Tiles reached by an unobstructed ray are "currently
/// visible" (stored in `VisibleTiles`) and are also added to the permanent
/// `Revealed` set so explored areas stay on the map.
pub fn fog_of_war(
    player_query: Query<&Position, With<Player>>,
    tiles: Res<Tiles>,
    mut revealed: ResMut<Revealed>,
    mut visible: ResMut<VisibleTiles>,
) {
    let Some(player_pos) = player_query.iter().next().copied() else {
        return;
    };

    let mut now_visible: BTreeSet<Position> = BTreeSet::new();
    // The player always sees their own tile.
    now_visible.insert(player_pos);

    let z = player_pos.z;
    for dx in -SIGHT_RADIUS..=SIGHT_RADIUS {
        for dy in -SIGHT_RADIUS..=SIGHT_RADIUS {
            // Restrict the sight area to a circle for a nicer look.
            if dx * dx + dy * dy > SIGHT_RADIUS * SIGHT_RADIUS {
                continue;
            }
            let target = Position::new(player_pos.x + dx, player_pos.y + dy, z);
            cast_ray(&tiles, player_pos, target, &mut now_visible);
        }
    }

    for pos in &now_visible {
        revealed.0.insert(*pos);
    }
    visible.0 = now_visible;
}

/// Walks a Bresenham line from `from` to `to`, marking each tile visible until an
/// opaque tile is hit (which is itself visible, but nothing beyond it is).
fn cast_ray(tiles: &Tiles, from: Position, to: Position, visible: &mut BTreeSet<Position>) {
    let mut x = from.x;
    let mut y = from.y;
    let z = from.z;
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();
    let sx = if from.x < to.x { 1 } else { -1 };
    let sy = if from.y < to.y { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        let pos = Position::new(x, y, z);
        visible.insert(pos);

        // Stop once we reach the target.
        if x == to.x && y == to.y {
            break;
        }

        // An opaque tile blocks the ray (but is itself visible, marked above).
        // The origin tile never blocks.
        if pos != from {
            let opaque = tiles
                .get(&pos)
                .map(|t| !t.passable)
                .unwrap_or(false);
            if opaque {
                break;
            }
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}
