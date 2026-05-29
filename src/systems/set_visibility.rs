use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::state::GameState;
use crate::tuning;

/// Controls per-entity visibility, combining floor (z-plane) gating with the
/// line-of-sight fog of war computed by `fog_of_war`.
///
/// Three tiers for tiles:
/// - In current line of sight  -> fully visible at base color.
/// - Explored but not in sight -> visible but DIMMED (base color * dim factor).
/// - Never seen                -> hidden.
///
/// Enemies and pickups only render while currently in line of sight, so they
/// stay hidden in explored-but-unseen ("remembered") areas. Anything off the
/// current floor is hidden.
///
/// B0001: the two `&mut Visibility`/`&Position` queries are made disjoint by
/// `With<Tile>` vs `Without<Tile>`, so they never alias the same entity.
#[allow(clippy::type_complexity)]
pub fn set_visibility(
    state: Res<State<GameState>>,
    floor: Res<Floor>,
    revealed: Res<Revealed>,
    visible_tiles: Res<VisibleTiles>,
    mut tile_query: Query<
        (&mut Visibility, &mut Sprite, &Position, &TileBaseColor),
        With<Tile>,
    >,
    mut other_query: Query<(&mut Visibility, &Position, Has<Enemy>), Without<Tile>>,
) {
    if state.get() == &GameState::Menu {
        return;
    }

    // Tiles: three-tier (visible / dimmed / hidden) with a color multiply.
    for (mut visibility, mut sprite, position, base) in tile_query.iter_mut() {
        if position.z != floor.0 {
            *visibility = Visibility::Hidden;
            continue;
        }

        let in_los = visible_tiles.0.contains(position);
        let explored = revealed.0.contains(position);

        if in_los {
            *visibility = Visibility::Visible;
            sprite.color = base.0;
        } else if explored {
            *visibility = Visibility::Visible;
            sprite.color = dim(base.0, tuning::FOG_DIM_FACTOR);
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    // Enemies / pickups / player: visible only when their tile is in sight (for
    // enemies) or in sight / explored (for static pickups + the player marker).
    for (mut visibility, position, is_enemy) in other_query.iter_mut() {
        if position.z != floor.0 {
            *visibility = Visibility::Hidden;
            continue;
        }

        let in_los = visible_tiles.0.contains(position);
        let explored = revealed.0.contains(position);
        let should_show = if is_enemy { in_los } else { in_los || explored };

        *visibility = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Multiplies a color's RGB toward black by `factor` (keeping alpha) for the
/// "explored but out of sight" dimmed look.
fn dim(color: Color, factor: f32) -> Color {
    let c = color.to_srgba();
    Color::srgba(c.red * factor, c.green * factor, c.blue * factor, c.alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs `set_visibility` with a tile (in-LOS), a tile (explored only), a
    /// tile (never seen), and an enemy (explored-but-not-in-LOS) all present.
    /// Asserts the system does not panic (B0001 disjointness) and applies the
    /// correct three-tier visibility/dimming.
    #[test]
    fn three_tier_visibility_no_query_conflict() {
        let mut world = World::new();
        world.insert_resource(State::new(GameState::Playing));
        world.insert_resource(Floor(0));

        let mut revealed = Revealed::new();
        let mut visible = VisibleTiles::new();
        // In LOS: (0,0). Explored only: (5,0). Never seen: (9,0).
        let in_los = Position::new(0, 0, 0);
        let explored_only = Position::new(5, 0, 0);
        let never_seen = Position::new(9, 0, 0);
        visible.0.insert(in_los);
        revealed.0.insert(in_los);
        revealed.0.insert(explored_only);
        world.insert_resource(revealed);
        world.insert_resource(visible);

        let make_tile = |w: &mut World, p: Position| {
            w.spawn((
                Sprite::default(),
                Visibility::Visible,
                p,
                Tile,
                TileBaseColor(Color::WHITE),
            ))
            .id()
        };
        let e_los = make_tile(&mut world, in_los);
        let e_explored = make_tile(&mut world, explored_only);
        let e_hidden = make_tile(&mut world, never_seen);
        // An enemy on an explored-but-not-in-LOS tile must be hidden.
        let enemy = world
            .spawn((Sprite::default(), Visibility::Visible, explored_only, Enemy))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(set_visibility);
        schedule.run(&mut world);

        assert_eq!(*world.get::<Visibility>(e_los).unwrap(), Visibility::Visible);
        assert_eq!(
            *world.get::<Visibility>(e_explored).unwrap(),
            Visibility::Visible
        );
        assert_eq!(
            *world.get::<Visibility>(e_hidden).unwrap(),
            Visibility::Hidden,
            "never-seen tile must be hidden"
        );
        assert_eq!(
            *world.get::<Visibility>(enemy).unwrap(),
            Visibility::Hidden,
            "enemy not in current LOS must be hidden"
        );

        // Dimmed tile is darker than the in-LOS tile.
        let los_color = world.get::<Sprite>(e_los).unwrap().color.to_srgba();
        let dim_color = world.get::<Sprite>(e_explored).unwrap().color.to_srgba();
        assert!(dim_color.red < los_color.red, "explored tile should be dimmed");
    }
}
