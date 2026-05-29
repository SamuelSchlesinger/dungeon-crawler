use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::state::GameState;

/// Controls per-entity visibility, combining the existing floor (z-plane) gating
/// with line-of-sight fog of war.
///
/// - Tiles / pickups / weapon drops on the current floor render if they are
///   currently in line of sight OR have been revealed before (explored).
/// - Enemies render only while currently in line of sight, so they stay hidden
///   in explored-but-unseen areas.
/// - Anything off the current floor is hidden, as before.
pub fn set_visibility(
    state: Res<State<GameState>>,
    floor: Res<Floor>,
    revealed: Res<Revealed>,
    visible_tiles: Res<VisibleTiles>,
    mut query: Query<(&mut Visibility, &Position, Has<Enemy>)>,
) {
    if state.get() == &GameState::Menu {
        return;
    }

    for (mut visibility, position, is_enemy) in query.iter_mut() {
        if position.z != floor.0 {
            *visibility = Visibility::Hidden;
            continue;
        }

        let in_los = visible_tiles.0.contains(position);
        let explored = revealed.0.contains(position);

        let should_show = if is_enemy {
            // Enemies only appear when actually in line of sight.
            in_los
        } else {
            // Static tiles / pickups remain visible once explored.
            in_los || explored
        };

        *visibility = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
