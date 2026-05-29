use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::tuning;

/// Smoothly lerps the camera toward the player every frame while follow is on
/// (the default). Uses the player's continuous `WorldPos` so the camera tracks
/// real-time motion, not the rounded grid tile. Framerate-independent lerp via
/// `1 - exp(-k*dt)`.
///
/// Disjoint queries: the camera query filters `With<CameraMarker>` and the
/// player query `With<Player>` (mutually exclusive markers), so there is no
/// B0001 overlap even though both touch `Transform`.
#[allow(clippy::type_complexity)]
pub fn follow(
    follow: Res<Follow>,
    mut floor: ResMut<Floor>,
    player_query: Query<(&WorldPos, &Position), (With<Player>, Without<CameraMarker>)>,
    mut camera_query: Query<&mut Transform, (With<CameraMarker>, Without<Player>)>,
    time: Res<Time>,
) {
    if !follow.0 {
        return;
    }
    let Some((world_pos, position)) = player_query.iter().next() else {
        return;
    };
    let Some(mut transform) = camera_query.iter_mut().next() else {
        return;
    };

    floor.0 = position.z;

    let target = Vec3::new(world_pos.0.x, world_pos.0.y, transform.translation.z);
    let t = 1.0 - (-tuning::CAMERA_LERP * time.delta_secs()).exp();
    transform.translation = transform.translation.lerp(target, t.clamp(0.0, 1.0));
}
