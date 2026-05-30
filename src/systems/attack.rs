use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::particle_system::spawn_particle;
use crate::systems::projectile::{kill_juice, on_enemy_killed, KillRewards};
use crate::tuning;

/// Mouse-aimed player attack, branching on the active weapon type (Wave 3).
///
/// On left mouse button or Space (and off cooldown), the player attacks toward
/// the mouse cursor. The behavior depends on `ActiveWeapon`:
/// - **Melee**: a cone swing -- every enemy within the (effective) range and
///   inside the `+/- half-angle` arc takes damage, knockback, and a hit-flash.
/// - **Ranged**: fires one (or, with the projectile boon, a small spread of)
///   `Projectile`(s) toward the aim; hits resolve in `move_projectiles`.
/// - **AoE**: a radial burst -- every enemy within `AOE_RADIUS_TILES` takes
///   damage and knockback away from the player.
///
/// All damage/range/cooldown/knockback values are read as EFFECTIVE values from
/// `PlayerStats` (base x boon modifiers), so boons feed back here uniformly.
///
/// Query disjointness (B0001): the player query uses `With<Player>` +
/// `Without<Enemy>`; the enemy query uses `With<Enemy>` + `Without<Player>`. Both
/// touch `WorldPos`/`Health` but over disjoint entity sets, so no overlap.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn player_attack(
    mut commands: Commands,
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_position: Res<MousePosition>,
    scale_factor: Res<ScaleFactor>,
    mut rewards: KillRewards,
    mut juice: ResMut<Juice>,
    mut sfx: MessageWriter<SfxEvent>,
    player_stats: Res<PlayerStats>,
    active_weapon: Res<ActiveWeapon>,
    camera_query: Query<(&Camera, &GlobalTransform), With<CameraMarker>>,
    mut player_query: Query<
        (&WorldPos, &Strength, &mut Health, &mut AttackCooldown, &mut Facing),
        (With<Player>, Without<Enemy>),
    >,
    mut enemy_query: Query<
        (Entity, &WorldPos, &mut Health, &mut Knockback, &Position, &EnemyType),
        (With<Enemy>, Without<Player>),
    >,
    health_bars: Query<(Entity, &HealthBar)>,
) {
    let Some((player_world, strength, mut player_health, mut cooldown, mut facing)) =
        player_query.iter_mut().next()
    else {
        return;
    };

    cooldown.0.tick(time.delta());

    let wants_attack =
        mouse_button.just_pressed(MouseButton::Left) || keyboard.just_pressed(KeyCode::Space);
    if !wants_attack || !cooldown.0.is_finished() {
        return;
    }

    // Aim toward the mouse cursor (converted to world space via the camera).
    let player_pos = player_world.0;
    let mouse_world = cursor_to_world(&camera_query, mouse_position.0).unwrap_or(player_pos);
    let mut aim = mouse_world - player_pos;
    if aim.length_squared() < 1e-3 {
        aim = facing.0; // Degenerate (cursor on player) -> reuse last facing.
    }
    let aim = aim.normalize_or_zero();
    if aim == Vec2::ZERO {
        return;
    }
    facing.0 = aim;

    // Reset the cooldown to the EFFECTIVE per-weapon cooldown.
    let cd = player_stats.effective_attack_cooldown(active_weapon.weapon_type);
    cooldown.0 = Timer::from_seconds(cd, TimerMode::Once);

    let scale = scale_factor.0;
    let knockback_impulse = player_stats.effective_knockback() * scale;

    // Swing/fire whoosh on every attack (ranged uses the same airy cue).
    sfx.write(SfxEvent::MeleeSwing);

    match active_weapon.weapon_type {
        WeaponType::Melee => melee_swing(
            &mut commands,
            player_pos,
            aim,
            scale,
            strength.0,
            knockback_impulse,
            &player_stats,
            &mut player_health,
            &mut rewards,
            &mut juice,
            &mut sfx,
            &mut enemy_query,
            &health_bars,
        ),
        WeaponType::Ranged => fire_projectiles(
            &mut commands,
            player_pos,
            aim,
            scale,
            strength.0,
            knockback_impulse,
            &player_stats,
        ),
        WeaponType::Aoe => aoe_burst(
            &mut commands,
            player_pos,
            scale,
            strength.0,
            &player_stats,
            &mut player_health,
            &mut rewards,
            &mut juice,
            &mut sfx,
            &mut enemy_query,
            &health_bars,
        ),
    }
}

/// Applies lifesteal: heals the player by `lifesteal * dmg` (capped at max HP).
fn apply_lifesteal(player_health: &mut Health, dmg: i64, stats: &PlayerStats) {
    if stats.lifesteal > 0.0 {
        let heal = (dmg as f32 * stats.lifesteal).round() as i64;
        if heal > 0 {
            player_health.0 = (player_health.0 + heal).min(stats.effective_max_hp());
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn melee_swing(
    commands: &mut Commands,
    player_pos: Vec2,
    aim: Vec2,
    scale: f32,
    base_strength: i64,
    knockback_impulse: f32,
    stats: &PlayerStats,
    player_health: &mut Health,
    rewards: &mut KillRewards,
    juice: &mut Juice,
    sfx: &mut MessageWriter<SfxEvent>,
    enemy_query: &mut Query<
        (Entity, &WorldPos, &mut Health, &mut Knockback, &Position, &EnemyType),
        (With<Enemy>, Without<Player>),
    >,
    health_bars: &Query<(Entity, &HealthBar)>,
) {
    let range = stats.effective_attack_range() * scale;
    let cos_half = stats.effective_attack_half_angle().cos();

    for (entity, enemy_world, mut health, mut knockback, grid_pos, enemy_type) in
        enemy_query.iter_mut()
    {
        let to_enemy = enemy_world.0 - player_pos;
        let dist = to_enemy.length();
        if dist > range {
            continue;
        }
        let dir = to_enemy.normalize_or_zero();
        if dir != Vec2::ZERO && aim.dot(dir) < cos_half {
            continue;
        }

        let (dmg, crit) = stats.roll_damage(base_strength);
        health.0 -= dmg;
        rewards.statistics.damage_dealt += dmg;
        apply_lifesteal(player_health, dmg, stats);

        let color = if crit {
            crate::systems::damage_numbers::DAMAGE_CRIT
        } else {
            crate::systems::damage_numbers::DAMAGE_TO_ENEMY
        };
        crate::systems::damage_numbers::spawn_damage_number(commands, enemy_world.0, dmg, color);

        let push = if dir == Vec2::ZERO { aim } else { dir };
        knockback.0 += push * knockback_impulse;

        commands.entity(entity).insert(HitFlash(Timer::from_seconds(
            tuning::HIT_FLASH_DURATION,
            TimerMode::Once,
        )));
        spawn_particle(
            commands,
            ParticleType::HitSpark,
            Vec3::new(enemy_world.0.x, enemy_world.0.y, 0.06),
        );
        sfx.write(SfxEvent::Hit);

        if health.0 <= 0 {
            kill_juice(juice, sfx, *enemy_type);
            on_enemy_killed(
                commands,
                entity,
                enemy_world.0,
                *grid_pos,
                &mut rewards.statistics,
                &mut rewards.gold,
                &mut rewards.weapon_drops,
                &rewards.sprite_texture,
                health_bars,
                *enemy_type,
                scale,
            );
        }
    }

    spawn_swing_visual(commands, player_pos, aim, scale, stats.effective_attack_range());
}

#[allow(clippy::too_many_arguments)]
fn fire_projectiles(
    commands: &mut Commands,
    player_pos: Vec2,
    aim: Vec2,
    scale: f32,
    base_strength: i64,
    knockback_impulse: f32,
    stats: &PlayerStats,
) {
    let count = 1 + stats.extra_projectiles.max(0);
    let speed = tuning::PROJECTILE_SPEED_TILES * scale;
    let range = tuning::PROJECTILE_RANGE_TILES * scale;
    let base_angle = aim.y.atan2(aim.x);

    // Symmetric spread around the aim direction.
    for i in 0..count {
        let offset = if count == 1 {
            0.0
        } else {
            // Spread the shots: -spread*(n-1)/2 .. +spread*(n-1)/2
            (i as f32 - (count as f32 - 1.0) / 2.0) * tuning::PROJECTILE_SPREAD
        };
        let angle = base_angle + offset;
        let dir = Vec2::new(angle.cos(), angle.sin());

        // Each projectile rolls its own crit/damage.
        let (dmg, crit) = stats.roll_damage(base_strength);

        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.9, 0.4),
                custom_size: Some(Vec2::new(scale * 0.35, scale * 0.18)),
                ..default()
            },
            Transform {
                translation: Vec3::new(player_pos.x, player_pos.y, 0.07),
                rotation: Quat::from_rotation_z(angle),
                ..default()
            },
            Visibility::Visible,
            Projectile {
                velocity: dir * speed,
                remaining: range,
                damage: dmg,
                knockback: knockback_impulse,
                crit,
                faction: ProjectileFaction::Player,
            },
        ));
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn aoe_burst(
    commands: &mut Commands,
    player_pos: Vec2,
    scale: f32,
    base_strength: i64,
    stats: &PlayerStats,
    player_health: &mut Health,
    rewards: &mut KillRewards,
    juice: &mut Juice,
    sfx: &mut MessageWriter<SfxEvent>,
    enemy_query: &mut Query<
        (Entity, &WorldPos, &mut Health, &mut Knockback, &Position, &EnemyType),
        (With<Enemy>, Without<Player>),
    >,
    health_bars: &Query<(Entity, &HealthBar)>,
) {
    let radius = tuning::AOE_RADIUS_TILES * scale;
    let knockback_impulse = tuning::AOE_KNOCKBACK_TILES * scale * stats.knockback_mult;

    for (entity, enemy_world, mut health, mut knockback, grid_pos, enemy_type) in
        enemy_query.iter_mut()
    {
        let to_enemy = enemy_world.0 - player_pos;
        if to_enemy.length() > radius {
            continue;
        }

        let (dmg, crit) = stats.roll_damage(base_strength);
        health.0 -= dmg;
        rewards.statistics.damage_dealt += dmg;
        apply_lifesteal(player_health, dmg, stats);

        let color = if crit {
            crate::systems::damage_numbers::DAMAGE_CRIT
        } else {
            crate::systems::damage_numbers::DAMAGE_TO_ENEMY
        };
        crate::systems::damage_numbers::spawn_damage_number(commands, enemy_world.0, dmg, color);

        let push = to_enemy.normalize_or_zero();
        let push = if push == Vec2::ZERO { Vec2::new(1.0, 0.0) } else { push };
        knockback.0 += push * knockback_impulse;

        commands.entity(entity).insert(HitFlash(Timer::from_seconds(
            tuning::HIT_FLASH_DURATION,
            TimerMode::Once,
        )));
        spawn_particle(
            commands,
            ParticleType::HitSpark,
            Vec3::new(enemy_world.0.x, enemy_world.0.y, 0.06),
        );
        sfx.write(SfxEvent::Hit);

        if health.0 <= 0 {
            kill_juice(juice, sfx, *enemy_type);
            on_enemy_killed(
                commands,
                entity,
                enemy_world.0,
                *grid_pos,
                &mut rewards.statistics,
                &mut rewards.gold,
                &mut rewards.weapon_drops,
                &rewards.sprite_texture,
                health_bars,
                *enemy_type,
                scale,
            );
        }
    }

    spawn_aoe_visual(commands, player_pos, radius);
}

/// Converts a screen-space cursor position to world space via the camera.
fn cursor_to_world(
    camera_query: &Query<(&Camera, &GlobalTransform), With<CameraMarker>>,
    cursor: Vec2,
) -> Option<Vec2> {
    let (camera, camera_transform) = camera_query.iter().next()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

/// Spawns a brief triangular "slash" sprite in front of the player.
fn spawn_swing_visual(commands: &mut Commands, origin: Vec2, aim: Vec2, scale: f32, range_tiles: f32) {
    let reach = range_tiles * scale;
    let center = origin + aim * reach * 0.5;
    let angle = aim.y.atan2(aim.x);

    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.85, 0.2, 0.7),
            custom_size: Some(Vec2::new(reach, scale * 0.9)),
            ..default()
        },
        Transform {
            translation: Vec3::new(center.x, center.y, 0.07),
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        Visibility::Visible,
        TransientVisual(Timer::from_seconds(
            tuning::SWING_VISUAL_LIFETIME,
            TimerMode::Once,
        )),
    ));
}

/// Spawns a brief expanding ring-ish square visual for the AoE burst.
fn spawn_aoe_visual(commands: &mut Commands, origin: Vec2, radius: f32) {
    commands.spawn((
        Sprite {
            color: Color::srgba(0.4, 0.7, 1.0, 0.45),
            custom_size: Some(Vec2::splat(radius * 2.0)),
            ..default()
        },
        Transform::from_xyz(origin.x, origin.y, 0.07),
        Visibility::Visible,
        TransientVisual(Timer::from_seconds(
            tuning::SWING_VISUAL_LIFETIME * 1.6,
            TimerMode::Once,
        )),
    ));
}
