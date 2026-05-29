use std::collections::BTreeSet;

use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::tuning;

/// Computes the player's line of sight for the current floor and records it.
///
/// Casts a Bresenham ray from the player to every tile within `FOG_RADIUS`.
/// A ray is blocked the moment it crosses an opaque tile (`passable == false`),
/// so walls cast shadows. Tiles reached by an unobstructed ray are "currently
/// visible" (stored in `VisibleTiles`) and are also added to the permanent
/// `Revealed` set so explored areas stay on the map (rendered dimmed).
pub fn fog_of_war(
    player_query: Query<&Position, With<Player>>,
    tiles: Res<Tiles>,
    mut revealed: ResMut<Revealed>,
    mut visible: ResMut<VisibleTiles>,
) {
    let Some(player_pos) = player_query.iter().next().copied() else {
        return;
    };

    let radius = tuning::FOG_RADIUS;
    let mut now_visible: BTreeSet<Position> = BTreeSet::new();
    // The player always sees their own tile.
    now_visible.insert(player_pos);

    let z = player_pos.z;
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            // Restrict the sight area to a circle for a nicer look.
            if dx * dx + dy * dy > radius * radius {
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
            let opaque = tiles.get(&pos).map(|t| !t.passable).unwrap_or(false);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::CachedTile;

    /// Builds a `Tiles` grid: a long open corridor on the +x axis with one opaque
    /// wall, plus a far-away tile the player must never see from spawn.
    fn corridor_tiles() -> Tiles {
        let mut tiles = Tiles::new();
        let dummy = Entity::from_raw_u32(1).unwrap();
        // Open floor from x=0..=20 at y=0.
        for x in 0..=20 {
            tiles.insert(
                Position::new(x, 0, 0),
                CachedTile { entity: dummy, passable: true },
            );
        }
        // Opaque wall at x=4 blocks everything beyond it.
        tiles.insert(
            Position::new(4, 0, 0),
            CachedTile { entity: dummy, passable: false },
        );
        tiles
    }

    /// Runs `fog_of_war` in a real Bevy world and returns the resulting
    /// VisibleTiles + Revealed sets, so we can assert the net effect HIDES things.
    fn run_fog(player: Position, tiles: Tiles) -> (BTreeSet<Position>, BTreeSet<Position>) {
        let mut world = World::new();
        world.insert_resource(tiles);
        world.insert_resource(Revealed::new());
        world.insert_resource(VisibleTiles::new());
        world.spawn((player, Player));

        let mut schedule = Schedule::default();
        schedule.add_systems(fog_of_war);
        schedule.run(&mut world);

        let visible = world.resource::<VisibleTiles>().0.clone();
        let revealed = world.resource::<Revealed>().0.clone();
        (visible, revealed)
    }

    #[test]
    fn wall_blocks_line_of_sight() {
        let player = Position::new(0, 0, 0);
        let (visible, _) = run_fog(player, corridor_tiles());

        // Player sees its own tile and tiles up to the wall.
        assert!(visible.contains(&player));
        assert!(visible.contains(&Position::new(2, 0, 0)));
        assert!(visible.contains(&Position::new(4, 0, 0)), "wall itself is visible");
        // Anything beyond the wall is hidden.
        assert!(!visible.contains(&Position::new(6, 0, 0)), "tile behind wall must be hidden");
        assert!(!visible.contains(&Position::new(10, 0, 0)));
    }

    #[test]
    fn radius_limits_sight() {
        let player = Position::new(0, 0, 0);
        // Fully open corridor (no wall) so only FOG_RADIUS limits sight.
        let mut tiles = Tiles::new();
        let dummy = Entity::from_raw_u32(1).unwrap();
        for x in 0..=20 {
            tiles.insert(
                Position::new(x, 0, 0),
                CachedTile { entity: dummy, passable: true },
            );
        }
        let (visible, _) = run_fog(player, tiles);
        // A tile farther than FOG_RADIUS is never visible, proving fog hides.
        let far = Position::new(tuning::FOG_RADIUS + 2, 0, 0);
        assert!(!visible.contains(&far), "tile beyond FOG_RADIUS must be hidden");
    }

    #[test]
    fn revealed_accumulates_but_visible_is_current_only() {
        // First frame at origin.
        let (visible1, revealed1) = run_fog(Position::new(0, 0, 0), corridor_tiles());
        assert!(revealed1.contains(&Position::new(2, 0, 0)));
        // Visible set is exactly the current LOS (subset of revealed here).
        assert!(visible1.is_subset(&revealed1));
    }
}
