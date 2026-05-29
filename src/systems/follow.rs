use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::tuning;

/// Smoothly lerps the camera toward the player every frame while follow is on
/// (the default), THEN applies the Wave 5 screen-shake offset on top.
///
/// Uses the player's continuous `WorldPos` so the camera tracks real-time
/// motion, not the rounded grid tile. Framerate-independent lerp via
/// `1 - exp(-k*dt)`.
///
/// ## Screen shake (Wave 5)
/// The shake is integrated INTO this system rather than a second system that
/// mutably borrows the camera `Transform` -- that would risk a B0001 conflict /
/// fight `follow` over the same component. Trauma (a 0..1 value in `Juice`)
/// decays each frame; the camera is offset by `trauma^2 * MAX_OFFSET * noise`.
/// The squared curve keeps small hits subtle. The offset is computed from the
/// follow TARGET (not the previous frame's already-shaken position), so the
/// shake never accumulates drift.
///
/// Disjoint queries: the camera query filters `With<CameraMarker>` and the
/// player query `With<Player>` (mutually exclusive markers), so there is no
/// B0001 overlap even though both touch `Transform`.
#[allow(clippy::type_complexity)]
pub fn follow(
    follow: Res<Follow>,
    mut floor: ResMut<Floor>,
    mut juice: ResMut<Juice>,
    real_time: Res<Time<Real>>,
    player_query: Query<(&WorldPos, &Position), (With<Player>, Without<CameraMarker>)>,
    mut camera_query: Query<&mut Transform, (With<CameraMarker>, Without<Player>)>,
) {
    // Trauma decays on REAL time so a hit-stop doesn't freeze the shake.
    let real_dt = real_time.delta_secs();
    if juice.trauma > 0.0 {
        juice.trauma = (juice.trauma - tuning::SHAKE_DECAY_PER_SEC * real_dt).max(0.0);
    }

    let Some(mut transform) = camera_query.iter_mut().next() else {
        return;
    };

    // Undo last frame's shake offset first so it never accumulates as drift.
    transform.translation -= juice.last_shake_offset.extend(0.0);

    // The point the camera is "anchored" to this frame, before shake. When
    // follow is on, lerp the anchor toward the player; otherwise keep the
    // current (manually-driven) position as the anchor.
    let mut anchor = transform.translation;
    if follow.0 {
        if let Some((world_pos, position)) = player_query.iter().next() {
            floor.0 = position.z;
            let target = Vec3::new(world_pos.0.x, world_pos.0.y, transform.translation.z);
            let t = 1.0 - (-tuning::CAMERA_LERP * real_dt).exp();
            anchor = transform.translation.lerp(target, t.clamp(0.0, 1.0));
        }
    }

    // Apply shake as an offset on top of the anchor. Sample two cheap,
    // decorrelated oscillators for the x/y offset; amplitude is trauma^2.
    let mut offset = Vec2::ZERO;
    if juice.trauma > 0.0001 {
        let amp = juice.trauma * juice.trauma * tuning::SHAKE_MAX_OFFSET;
        let t_secs = real_time.elapsed_secs();
        let phase = t_secs * tuning::SHAKE_FREQUENCY;
        // Mixed-frequency sines approximate band-limited noise without a PRNG.
        let nx = (phase).sin() * 0.6 + (phase * 2.13 + 1.7).sin() * 0.4;
        let ny = (phase * 1.31 + 4.2).sin() * 0.6 + (phase * 2.71 + 0.3).sin() * 0.4;
        offset = Vec2::new(nx, ny) * amp;
    }

    transform.translation = anchor + offset.extend(0.0);
    juice.last_shake_offset = offset;
}
