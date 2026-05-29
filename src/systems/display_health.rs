use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

pub fn display_health(
    scale_factor: Res<ScaleFactor>,
    floor: Res<Floor>,
    visible_tiles: Res<VisibleTiles>,
    mut position_health_query: Query<
        (Entity, &Position, &Health, &OriginalHealth, Has<Enemy>),
        Or<(With<Enemy>, With<Player>)>,
    >,
    mut health_query: Query<
        (&HealthBar, &mut Visibility, &mut Sprite, &mut Transform),
        (Without<Enemy>, Without<Player>, Without<CameraMarker>),
    >,
) {
    for (entity, position, health, original_health, is_enemy) in position_health_query.iter_mut() {
        for (health_bar, mut visibility, mut sprite, mut transform) in health_query.iter_mut() {
            if entity == health_bar.0 {
                // On the current floor, and (for enemies) only when in line of
                // sight so fog-hidden enemies don't reveal themselves via bars.
                let on_floor = position.z == floor.0;
                let visible = on_floor && (!is_enemy || visible_tiles.0.contains(position));
                *visibility = if visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                let fraction_of_health = health.0 as f32 / original_health.0 as f32;
                if fraction_of_health <= 0.2 {
                    sprite.color = Color::srgb(1.0, 0., 0.);
                } else if fraction_of_health <= 0.5 {
                    sprite.color = Color::srgb(1., 1., 0.);
                } else {
                    sprite.color = Color::srgb(0., 1., 0.);
                }
                sprite.custom_size = Some(Vec2::new(
                    scale_factor.0 as f32 / 2. * health.0 as f32 / original_health.0 as f32,
                    scale_factor.0 as f32 / 8.,
                ));
                *transform = Transform::from_xyz(
                    (position.x as f32 - 0.5) * scale_factor.0,
                    (position.y as f32 - 0.5) * scale_factor.0 - scale_factor.0 / 3.,
                    0.05,
                );
            }
        }
    }
}
