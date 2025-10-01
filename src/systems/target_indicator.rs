use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

/// Updates the target indicator to show which enemy the player will attack
/// based on mouse position. Highlights the closest adjacent enemy to the cursor.
pub fn update_target_indicator(
    mut commands: Commands,
    player_query: Query<&Position, With<Player>>,
    enemy_query: Query<(Entity, &Position, &Transform), With<Enemy>>,
    mouse_position: Res<MousePosition>,
    scale_factor: Res<ScaleFactor>,
    mut existing_indicator: Query<(Entity, &mut Transform, &mut Visibility), With<TargetIndicator>>,
    mut targeted_enemies: Query<Entity, With<TargetedEnemy>>,
) {
    // Clear previous targeted markers
    for entity in targeted_enemies.iter_mut() {
        commands.entity(entity).remove::<TargetedEnemy>();
    }

    let Some(player_pos) = player_query.iter().next() else {
        // Hide indicator if no player
        if let Some((_, _, mut vis)) = existing_indicator.iter_mut().next() {
            *vis = Visibility::Hidden;
        }
        return;
    };

    // Find adjacent enemies
    let adjacent_enemies: Vec<(Entity, Position, Vec3)> = enemy_query
        .iter()
        .filter(|(_, enemy_pos, _)| enemy_pos.is_adjacent_to(*player_pos))
        .map(|(e, p, t)| (e, *p, t.translation))
        .collect();

    if adjacent_enemies.is_empty() {
        // Hide indicator if no adjacent enemies
        if let Some((_, _, mut vis)) = existing_indicator.iter_mut().next() {
            *vis = Visibility::Hidden;
        }
        return;
    }

    // Find closest enemy to mouse cursor
    let mouse_world_pos = Vec2::new(mouse_position.0.x, mouse_position.0.y);
    let closest_enemy = adjacent_enemies
        .iter()
        .min_by(|(_, _, pos_a), (_, _, pos_b)| {
            let dist_a = mouse_world_pos.distance(Vec2::new(pos_a.x, pos_a.y));
            let dist_b = mouse_world_pos.distance(Vec2::new(pos_b.x, pos_b.y));
            dist_a.partial_cmp(&dist_b).unwrap()
        });

    if let Some((target_entity, _, target_transform)) = closest_enemy {
        // Mark the targeted enemy
        commands.entity(*target_entity).insert(TargetedEnemy);

        // Update or spawn indicator
        if let Some((_, mut transform, mut vis)) = existing_indicator.iter_mut().next() {
            transform.translation = Vec3::new(
                target_transform.x,
                target_transform.y + scale_factor.0 * 0.6,
                0.06,
            );
            *vis = Visibility::Visible;
        } else {
            // Spawn new indicator
            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 1.0, 0.0),
                    custom_size: Some(Vec2::new(scale_factor.0 * 0.3, scale_factor.0 * 0.1)),
                    ..default()
                },
                Transform::from_xyz(
                    target_transform.x,
                    target_transform.y + scale_factor.0 * 0.6,
                    0.06,
                ),
                Visibility::Visible,
                TargetIndicator,
            ));
        }
    }
}
