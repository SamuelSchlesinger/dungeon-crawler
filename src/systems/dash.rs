use bevy::prelude::*;

use crate::components::*;
use crate::resources::PlayerStats;
use crate::tuning;

/// Dash / dodge.
///
/// Left Shift or right mouse button triggers a short, fast burst in the current
/// movement direction (or the player's facing if standing still). The burst
/// grants brief invulnerability (i-frames) and then a cooldown. The actual
/// translation is applied in `move_player` (which reads `Dash.dashing`/`dir`);
/// this system only drives the state machine and timers.
pub fn dash(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    player_stats: Res<PlayerStats>,
    mut query: Query<(&mut Dash, &Facing), With<Player>>,
) {
    let Some((mut dash, facing)) = query.iter_mut().next() else {
        return;
    };

    let dt = time.delta();
    dash.cooldown.tick(dt);
    // i-frames tick independently of the movement burst so the invulnerability
    // window (DASH_IFRAMES) can outlast the burst (DASH_DURATION).
    dash.iframes.tick(dt);
    if dash.dashing {
        dash.active.tick(dt);
        if dash.active.is_finished() {
            dash.dashing = false;
        }
    }

    let wants_dash = keyboard.just_pressed(KeyCode::ShiftLeft)
        || mouse_button.just_pressed(MouseButton::Right);

    if wants_dash && !dash.dashing && dash.cooldown.is_finished() {
        // Dash in the current facing (move_player keeps Facing pointing the way
        // the player is moving, and toward the aim when attacking).
        let dir = if facing.0 == Vec2::ZERO {
            Vec2::new(1.0, 0.0)
        } else {
            facing.0.normalize()
        };
        dash.dir = dir;
        dash.dashing = true;
        dash.active = Timer::from_seconds(tuning::DASH_DURATION, TimerMode::Once);
        dash.iframes = Timer::from_seconds(tuning::DASH_IFRAMES, TimerMode::Once);
        // Dash cooldown is the EFFECTIVE cooldown (base x boon reduction).
        dash.cooldown =
            Timer::from_seconds(player_stats.effective_dash_cooldown(), TimerMode::Once);
    }
}

/// True while the player has active dash invulnerability (the i-frame window,
/// which may outlast the movement burst).
pub fn player_invulnerable(dash: &Dash) -> bool {
    !dash.iframes.is_finished()
}
