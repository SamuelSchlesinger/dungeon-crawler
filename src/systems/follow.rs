use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

pub fn follow(
    follow: Res<Follow>,
    scale_factor: Res<ScaleFactor>,
    mut floor: ResMut<Floor>,
    mut player_query: Query<&Position, With<Player>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    if follow.0 {
        camera_query.iter_mut().next().and_then(|transform| {
            player_query
                .iter_mut()
                .next()
                .map(|position| (transform, position))
        })
    } else {
        None
    }
    .map(|(mut transform, position)| {
        floor.0 = position.z;
        *transform = transform.with_translation(Vec3::new(
            (position.x as f32) * scale_factor.0,
            (position.y as f32) * scale_factor.0,
            1.,
        ));
    });
}
