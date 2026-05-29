use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::particle_system::spawn_particle;
use crate::tuning;

/// Mouse-aimed melee swing.
///
/// On left mouse button or Space (and off cooldown), the player swings a melee
/// cone toward the mouse cursor: every enemy within `ATTACK_RANGE` and inside
/// the `+/- ATTACK_HALF_ANGLE` arc takes `Strength` damage, knockback away from
/// the player, and a red hit-flash. A short-lived triangle sprite visualizes the
/// swing in the aim direction. Replaces the old auto-adjacency `combat` system
/// and the passive `TargetIndicator`.
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
    mut statistics: ResMut<Statistics>,
    mut weapon_drops: ResMut<WeaponDrops>,
    sprite_texture: Res<SpriteTexture>,
    camera_query: Query<(&Camera, &GlobalTransform), With<CameraMarker>>,
    mut player_query: Query<
        (&WorldPos, &Strength, &mut AttackCooldown, &mut Facing),
        (With<Player>, Without<Enemy>),
    >,
    mut enemy_query: Query<
        (Entity, &WorldPos, &mut Health, &mut Knockback, &Position),
        (With<Enemy>, Without<Player>),
    >,
    health_bars: Query<(Entity, &HealthBar)>,
) {
    let Some((player_world, strength, mut cooldown, mut facing)) = player_query.iter_mut().next()
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
    cooldown.0.reset();

    let scale = scale_factor.0;
    let range = tuning::ATTACK_RANGE_TILES * scale;
    let cos_half = tuning::ATTACK_HALF_ANGLE.cos();

    // Hit every enemy inside the cone.
    for (entity, enemy_world, mut health, mut knockback, grid_pos) in enemy_query.iter_mut() {
        let to_enemy = enemy_world.0 - player_pos;
        let dist = to_enemy.length();
        if dist > range {
            continue;
        }
        let dir = to_enemy.normalize_or_zero();
        // Allow point-blank hits (dir == 0) and anything inside the arc.
        if dir != Vec2::ZERO && aim.dot(dir) < cos_half {
            continue;
        }

        health.0 -= strength.0;
        statistics.damage_dealt += strength.0;

        // Knockback away from the player.
        let push = if dir == Vec2::ZERO { aim } else { dir };
        knockback.0 += push * tuning::ATTACK_KNOCKBACK_TILES * scale;

        // Hit flash + spark particles.
        commands
            .entity(entity)
            .insert(HitFlash(Timer::from_seconds(
                tuning::HIT_FLASH_DURATION,
                TimerMode::Once,
            )));
        spawn_particle(
            &mut commands,
            ParticleType::HitSpark,
            Vec3::new(enemy_world.0.x, enemy_world.0.y, 0.06),
        );

        if health.0 <= 0 {
            spawn_particle(
                &mut commands,
                ParticleType::Death,
                Vec3::new(enemy_world.0.x, enemy_world.0.y, 0.06),
            );
            commands.entity(entity).despawn();
            statistics.enemies_killed += 1;

            // Despawn the matching health bar.
            for (bar_entity, HealthBar(owner)) in health_bars.iter() {
                if *owner == entity {
                    commands.entity(bar_entity).despawn();
                }
            }

            // ~30% chance to drop a weapon at the enemy's grid tile.
            let drop_pos = *grid_pos;
            if rand::random::<f32>() < 0.30 && !weapon_drops.0.contains_key(&drop_pos) {
                let stats = WeaponStats::random();
                let (texture_image, texture_layout) = &sprite_texture.0;
                let drop_entity = commands
                    .spawn((
                        Sprite::from_atlas_image(
                            texture_image.clone(),
                            TextureAtlas {
                                layout: texture_layout.clone(),
                                index: WEAPON_SPRITE_INDEX,
                            },
                        ),
                        Transform::from_xyz(enemy_world.0.x, enemy_world.0.y, 0.008),
                        Visibility::Visible,
                        drop_pos,
                        Passable(true),
                        WeaponDrop,
                        stats,
                        SpriteIndex(WEAPON_SPRITE_INDEX),
                        ZLevel(0.008),
                    ))
                    .id();
                weapon_drops.insert(drop_pos, drop_entity);
            }
        }
    }

    // Spawn the swing visual: an orange triangle pointing in the aim direction.
    spawn_swing_visual(&mut commands, player_pos, aim, scale);
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
fn spawn_swing_visual(commands: &mut Commands, origin: Vec2, aim: Vec2, scale: f32) {
    let reach = tuning::ATTACK_RANGE_TILES * scale;
    let center = origin + aim * reach * 0.5;
    let angle = aim.y.atan2(aim.x);

    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.85, 0.2, 0.7),
            // A wide, short rectangle reads as a swipe arc when oriented to the aim.
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
