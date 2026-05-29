use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::tuning;

/// Color for damage dealt TO enemies (player's hits): bright gold.
pub const DAMAGE_TO_ENEMY: Color = Color::srgb(1.0, 0.85, 0.2);
/// Color for damage dealt TO the player (enemy hits): hot red.
pub const DAMAGE_TO_PLAYER: Color = Color::srgb(1.0, 0.3, 0.3);
/// Color for critical hits dealt to enemies: bright orange-white.
pub const DAMAGE_CRIT: Color = Color::srgb(1.0, 0.55, 0.1);
/// Color for gold "+N" reward numbers.
pub const GOLD_COLOR: Color = Color::srgb(1.0, 0.82, 0.0);

/// Spawns a floating damage number at a world position. `amount` is the damage
/// value shown; `color` distinguishes who took the hit (see constants above).
pub fn spawn_damage_number(commands: &mut Commands, world: Vec2, amount: i64, color: Color) {
    commands.spawn((
        Text2d::new(format!("{amount}")),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(color),
        // Above the actor, on top of everything else in the world.
        Transform::from_xyz(world.x, world.y + 12.0, 0.5),
        Visibility::Visible,
        DamageNumber {
            timer: Timer::from_seconds(tuning::DAMAGE_NUMBER_LIFETIME, TimerMode::Once),
            color,
        },
    ));
}

/// Spawns a floating gold "+N" reward number at a world position. Uses the same
/// rise/fade animation as damage numbers (it is a `DamageNumber` under the hood).
pub fn spawn_gold_number(commands: &mut Commands, world: Vec2, amount: i64) {
    commands.spawn((
        Text2d::new(format!("+{amount}g")),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(GOLD_COLOR),
        // Offset slightly to the side so it doesn't overlap the damage number.
        Transform::from_xyz(world.x + 14.0, world.y + 12.0, 0.5),
        Visibility::Visible,
        DamageNumber {
            timer: Timer::from_seconds(tuning::DAMAGE_NUMBER_LIFETIME, TimerMode::Once),
            color: GOLD_COLOR,
        },
    ));
}

/// Ticks floating damage numbers: rise upward, fade out, then despawn.
pub fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    scale_factor: Res<ScaleFactor>,
    mut query: Query<(Entity, &mut DamageNumber, &mut Transform, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    let rise = tuning::DAMAGE_NUMBER_RISE_TILES * scale_factor.0 * dt;
    for (entity, mut number, mut transform, mut color) in query.iter_mut() {
        number.timer.tick(time.delta());
        if number.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.y += rise;
        let alpha = 1.0 - number.timer.fraction();
        let c = number.color.to_srgba();
        color.0 = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}
