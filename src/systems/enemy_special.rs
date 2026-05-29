//! Wave 4 special enemy behaviors: Archer (ranged kiter), Charger (telegraphed
//! lunge), Bomber (exploder), and the boss combined attack pattern, plus the
//! shared explosion resolver and room-hazard DoT.
//!
//! ## B0001 discipline
//! Every system here that touches both enemies and the player makes the two
//! queries disjoint with `With<Enemy>, Without<Player>` vs
//! `With<Player>, Without<Enemy>`. Systems that only need to *read* the player's
//! position take a separate, read-only player query (Bevy allows multiple
//! shared borrows of the same component across queries; the conflicts that
//! panic are mutable aliases over the SAME entity set, which the marker filters
//! rule out). Movement systems here mutate enemy `WorldPos`/`Transform`; they
//! run in `Update` ordered after `enemy_move` so the default mover and these
//! never alias the same component within one system.

use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::dash::player_invulnerable;
use crate::systems::particle_system::spawn_particle;
use crate::tuning;
use crate::utils::world_to_grid;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Spawns an enemy projectile traveling `dir` (unit) from `from`.
#[allow(clippy::too_many_arguments)]
fn spawn_enemy_projectile(
    commands: &mut Commands,
    from: Vec2,
    dir: Vec2,
    scale: f32,
    speed_tiles: f32,
    range_tiles: f32,
    damage: i64,
    knockback_tiles: f32,
    color: Color,
) {
    let dir = dir.normalize_or_zero();
    if dir == Vec2::ZERO {
        return;
    }
    let angle = dir.y.atan2(dir.x);
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(scale * 0.35, scale * 0.18)),
            ..default()
        },
        Transform {
            translation: Vec3::new(from.x, from.y, 0.07),
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        Visibility::Visible,
        Projectile {
            velocity: dir * speed_tiles * scale,
            remaining: range_tiles * scale,
            damage: damage.max(1),
            knockback: knockback_tiles * scale,
            crit: false,
            faction: ProjectileFaction::Enemy,
        },
    ));
}

/// Per-axis wall collision for a continuous move (mirrors `enemy_ai::apply_move`).
fn apply_move(world_pos: &mut WorldPos, delta: Vec2, tiles: &Tiles, scale: f32, z: i64) {
    let radius = tuning::PLAYER_RADIUS_TILES * scale;
    let mut p = world_pos.0;
    let cx = Vec2::new(p.x + delta.x, p.y);
    if !blocked(cx, radius, tiles, scale, z) {
        p.x = cx.x;
    }
    let cy = Vec2::new(p.x, p.y + delta.y);
    if !blocked(cy, radius, tiles, scale, z) {
        p.y = cy.y;
    }
    world_pos.0 = p;
}

fn blocked(world: Vec2, radius: f32, tiles: &Tiles, scale: f32, z: i64) -> bool {
    let probes = [
        Vec2::new(world.x + radius, world.y),
        Vec2::new(world.x - radius, world.y),
        Vec2::new(world.x, world.y + radius),
        Vec2::new(world.x, world.y - radius),
    ];
    probes.iter().any(|p| {
        let tile = world_to_grid(*p, scale, z);
        tiles.get(&tile).is_none_or(|c| !c.passable)
    })
}

/// True if a step from `from` to `to` would cross a solid tile (cheap sampled
/// raycast), so a charge stops at a wall instead of tunneling through it.
fn hits_wall(from: Vec2, to: Vec2, tiles: &Tiles, scale: f32, z: i64) -> bool {
    let tile = world_to_grid(to, scale, z);
    tiles.get(&tile).is_none_or(|c| !c.passable) || {
        let mid = (from + to) * 0.5;
        let mtile = world_to_grid(mid, scale, z);
        tiles.get(&mtile).is_none_or(|c| !c.passable)
    }
}

// ---------------------------------------------------------------------------
// Archer
// ---------------------------------------------------------------------------

/// Archers fire enemy arrows at the player on a cooldown when the player is awake,
/// within `ARCHER_FIRE_RANGE`. (Their kiting movement is handled in `enemy_move`.)
///
/// Disjoint queries: archers `With<Enemy>` (mut `ArcherShoot`) and the read-only
/// player query `With<Player>, Without<Enemy>`.
#[allow(clippy::type_complexity)]
pub fn archer_shoot(
    mut commands: Commands,
    time: Res<Time>,
    scale_factor: Res<ScaleFactor>,
    mut archers: Query<
        (&WorldPos, &Awake, &mut ArcherShoot, &Strength),
        (With<Enemy>, Without<Player>),
    >,
    player_query: Query<&WorldPos, (With<Player>, Without<Enemy>)>,
) {
    let Some(player_world) = player_query.iter().next() else {
        return;
    };
    let scale = scale_factor.0;
    let fire_range = tuning::ARCHER_FIRE_RANGE * scale;

    for (archer_world, awake, mut shoot, strength) in archers.iter_mut() {
        shoot.0.tick(time.delta());
        if !awake.0 {
            continue;
        }
        let to_player = player_world.0 - archer_world.0;
        if to_player.length() > fire_range {
            continue;
        }
        if shoot.0.is_finished() {
            shoot.0 = Timer::from_seconds(tuning::ARCHER_SHOOT_COOLDOWN, TimerMode::Once);
            spawn_enemy_projectile(
                &mut commands,
                archer_world.0,
                to_player,
                scale,
                tuning::ARCHER_PROJECTILE_SPEED_TILES,
                tuning::ARCHER_PROJECTILE_RANGE_TILES,
                strength.0,
                tuning::ENEMY_KNOCKBACK_TILES,
                Color::srgb(1.0, 0.7, 0.2),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Charger
// ---------------------------------------------------------------------------

/// Charger state machine. Walking -> (player close) WindingUp [telegraph scale
/// pulse] -> Dashing [fast straight lunge in the locked direction, contact slam]
/// -> Recovering -> Walking. While in any non-Walking phase the charger drives
/// its own `WorldPos`, so `enemy_move` skips its chase steering (its
/// `self_driven` check yields to this system).
///
/// Disjoint queries: chargers `With<Enemy>, Without<Player>` (mut WorldPos /
/// Transform / ChargeState), player `With<Player>, Without<Enemy>` (mut Health /
/// Knockback, read Dash). No shared mutable component over the same entity.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn charger_ai(
    mut commands: Commands,
    time: Res<Time>,
    tiles: Res<Tiles>,
    scale_factor: Res<ScaleFactor>,
    mut statistics: ResMut<Statistics>,
    mut chargers: Query<
        (
            &mut WorldPos,
            &mut Transform,
            &mut ChargeState,
            &Awake,
            &Strength,
            &Facing,
        ),
        (With<Enemy>, Without<Player>),
    >,
    mut player_query: Query<
        (Entity, &WorldPos, &mut Health, &mut Knockback, &Dash),
        (With<Player>, Without<Enemy>),
    >,
) {
    let Some((player_entity, player_world, mut player_health, mut player_kb, dash)) =
        player_query.iter_mut().next()
    else {
        return;
    };
    let dt = time.delta_secs();
    let scale = scale_factor.0;
    let trigger = tuning::CHARGER_TRIGGER_RANGE * scale;
    let hit_radius = tuning::CHARGER_HIT_RADIUS_TILES * scale;
    let invulnerable = player_invulnerable(dash);

    for (mut world_pos, mut transform, mut state, awake, strength, facing) in chargers.iter_mut() {
        if !awake.0 {
            continue;
        }
        state.timer.tick(time.delta());
        let to_player = player_world.0 - world_pos.0;
        let dist = to_player.length();
        let z = world_to_grid(world_pos.0, scale, 0).z;

        match state.phase {
            ChargePhase::Walking => {
                transform.scale = Vec3::ONE;
                if dist <= trigger {
                    state.phase = ChargePhase::WindingUp;
                    state.timer = Timer::from_seconds(tuning::CHARGER_WINDUP, TimerMode::Once);
                }
                // (Approach is handled by enemy_move for the Walking phase.)
            }
            ChargePhase::WindingUp => {
                // Telegraph: pulse scale up so the player can read + dodge.
                let f = state.timer.fraction();
                transform.scale = Vec3::splat(1.0 + 0.5 * f);
                if state.timer.is_finished() {
                    // Lock the dash direction toward the player's position NOW.
                    let dir = if to_player == Vec2::ZERO {
                        facing.0
                    } else {
                        to_player.normalize_or_zero()
                    };
                    state.dir = if dir == Vec2::ZERO { Vec2::new(1.0, 0.0) } else { dir };
                    state.hit_landed = false;
                    state.phase = ChargePhase::Dashing;
                    state.timer =
                        Timer::from_seconds(tuning::CHARGER_DASH_DURATION, TimerMode::Once);
                }
            }
            ChargePhase::Dashing => {
                transform.scale = Vec3::splat(1.15);
                let step = state.dir * tuning::CHARGER_DASH_SPEED_TILES * scale * dt;
                let target = world_pos.0 + step;
                if hits_wall(world_pos.0, target, &tiles, scale, z) {
                    // Slam into a wall: end the charge early.
                    state.phase = ChargePhase::Recovering;
                    state.timer =
                        Timer::from_seconds(tuning::CHARGER_RECOVERY, TimerMode::Once);
                } else {
                    apply_move(&mut world_pos, step, &tiles, scale, z);
                    transform.translation.x = world_pos.0.x;
                    transform.translation.y = world_pos.0.y;
                }

                // Contact slam (once per charge).
                if !state.hit_landed
                    && (player_world.0 - world_pos.0).length() <= hit_radius
                    && !invulnerable
                {
                    state.hit_landed = true;
                    let dmg = ((strength.0 as f32) * tuning::CHARGER_DAMAGE_MULT).round() as i64;
                    player_health.0 -= dmg;
                    statistics.damage_taken += dmg;
                    crate::systems::damage_numbers::spawn_damage_number(
                        &mut commands,
                        player_world.0,
                        dmg,
                        crate::systems::damage_numbers::DAMAGE_TO_PLAYER,
                    );
                    let push = state.dir;
                    player_kb.0 += push * tuning::CHARGER_KNOCKBACK_TILES * scale;
                    commands.entity(player_entity).insert(HitFlash(
                        Timer::from_seconds(tuning::HIT_FLASH_DURATION, TimerMode::Once),
                    ));
                    spawn_particle(
                        &mut commands,
                        ParticleType::HitSpark,
                        Vec3::new(player_world.0.x, player_world.0.y, 0.06),
                    );
                    if player_health.0 <= 0 {
                        spawn_particle(
                            &mut commands,
                            ParticleType::Death,
                            Vec3::new(player_world.0.x, player_world.0.y, 0.06),
                        );
                        commands.entity(player_entity).despawn();
                    }
                }

                if state.timer.is_finished() {
                    state.phase = ChargePhase::Recovering;
                    state.timer =
                        Timer::from_seconds(tuning::CHARGER_RECOVERY, TimerMode::Once);
                }
            }
            ChargePhase::Recovering => {
                transform.scale = Vec3::splat(0.9);
                if state.timer.is_finished() {
                    transform.scale = Vec3::ONE;
                    state.phase = ChargePhase::Walking;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Bomber
// ---------------------------------------------------------------------------

/// Bomber fuse. Once the player is within `BOMBER_FUSE_RANGE` (or the bomber is
/// killed, handled below by HP check), the fuse arms: the bomber flashes/scales,
/// and on expiry it detonates -- spawning an `Explosion` entity that
/// `resolve_explosions` turns into AoE damage + knockback. The bomber despawns on
/// detonation.
///
/// Disjoint queries: bombers `With<Enemy>, Without<Player>` and the read-only
/// player position query.
#[allow(clippy::type_complexity)]
pub fn bomber_ai(
    mut commands: Commands,
    time: Res<Time>,
    scale_factor: Res<ScaleFactor>,
    mut bombers: Query<
        (
            Entity,
            &WorldPos,
            &mut Transform,
            &mut BomberFuse,
            &Awake,
            &Strength,
        ),
        (With<Enemy>, Without<Player>),
    >,
    player_query: Query<&WorldPos, (With<Player>, Without<Enemy>)>,
    health_bars: Query<(Entity, &HealthBar)>,
) {
    let Some(player_world) = player_query.iter().next() else {
        return;
    };
    let scale = scale_factor.0;
    let fuse_range = tuning::BOMBER_FUSE_RANGE * scale;

    for (entity, world, mut transform, mut fuse, awake, strength) in bombers.iter_mut() {
        if !awake.0 {
            continue;
        }
        let dist = (player_world.0 - world.0).length();

        // Arm the fuse on proximity. (A bomber killed by the player explodes
        // immediately via `reap_dead_enemies`, which spawns the explosion there.)
        if !fuse.armed && dist <= fuse_range {
            fuse.armed = true;
            fuse.timer = Timer::from_seconds(tuning::BOMBER_FUSE, TimerMode::Once);
        }

        if fuse.armed {
            fuse.timer.tick(time.delta());
            // Flash + scale pulse as the telegraph.
            let f = fuse.timer.fraction();
            transform.scale = Vec3::splat(1.0 + 0.6 * f);

            if fuse.timer.is_finished() {
                // Detonate: spawn an explosion, a death particle, despawn bomber.
                let dmg =
                    ((strength.0 as f32) * tuning::BOMBER_DAMAGE_MULT).round().max(1.0) as i64;
                commands.spawn((Explosion {
                    center: world.0,
                    radius: tuning::BOMBER_EXPLOSION_RADIUS_TILES * scale,
                    damage: dmg,
                    knockback: tuning::BOMBER_KNOCKBACK_TILES * scale,
                },));
                spawn_explosion_visual(
                    &mut commands,
                    world.0,
                    tuning::BOMBER_EXPLOSION_RADIUS_TILES * scale,
                );
                spawn_particle(
                    &mut commands,
                    ParticleType::Death,
                    Vec3::new(world.0.x, world.0.y, 0.06),
                );
                // Despawn the bomber + its health bar (its kill is credited when
                // the explosion is harmless to itself; gold is awarded by the
                // killer's path if it died from damage, otherwise here).
                for (bar_entity, HealthBar(owner)) in health_bars.iter() {
                    if *owner == entity {
                        commands.entity(bar_entity).despawn();
                    }
                }
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Brief expanding orange ring visual for an explosion.
fn spawn_explosion_visual(commands: &mut Commands, center: Vec2, radius: f32) {
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.5, 0.1, 0.6),
            custom_size: Some(Vec2::splat(radius * 2.0)),
            ..default()
        },
        Transform::from_xyz(center.x, center.y, 0.07),
        Visibility::Visible,
        TransientVisual(Timer::from_seconds(
            tuning::SWING_VISUAL_LIFETIME * 2.0,
            TimerMode::Once,
        )),
    ));
}

// ---------------------------------------------------------------------------
// Explosion resolver (shared by bombers + boss adds)
// ---------------------------------------------------------------------------

/// Applies each pending `Explosion` to the player and to enemies in radius, then
/// despawns the explosion marker. Enemy damage is dealt via `Health`; the regular
/// `reap_dead_enemies` system credits any kills next.
///
/// Disjoint queries: explosions (`With<Explosion>`, no actor markers), enemies
/// (`With<Enemy>, Without<Player>`), player (`With<Player>, Without<Enemy>`).
#[allow(clippy::type_complexity)]
pub fn resolve_explosions(
    mut commands: Commands,
    mut statistics: ResMut<Statistics>,
    explosions: Query<(Entity, &Explosion)>,
    mut enemy_query: Query<
        (Entity, &WorldPos, &mut Health, &mut Knockback),
        (With<Enemy>, Without<Player>),
    >,
    mut player_query: Query<
        (Entity, &WorldPos, &mut Health, &mut Knockback, &Dash),
        (With<Player>, Without<Enemy>),
    >,
) {
    for (expl_entity, expl) in explosions.iter() {
        // Player.
        if let Some((player_entity, player_world, mut player_health, mut player_kb, dash)) =
            player_query.iter_mut().next()
        {
            let d = (player_world.0 - expl.center).length();
            if d <= expl.radius && !player_invulnerable(dash) {
                let dmg = expl.damage;
                player_health.0 -= dmg;
                statistics.damage_taken += dmg;
                crate::systems::damage_numbers::spawn_damage_number(
                    &mut commands,
                    player_world.0,
                    dmg,
                    crate::systems::damage_numbers::DAMAGE_TO_PLAYER,
                );
                let push = (player_world.0 - expl.center).normalize_or_zero();
                if push != Vec2::ZERO {
                    player_kb.0 += push * expl.knockback;
                }
                commands.entity(player_entity).insert(HitFlash(
                    Timer::from_seconds(tuning::HIT_FLASH_DURATION, TimerMode::Once),
                ));
                if player_health.0 <= 0 {
                    spawn_particle(
                        &mut commands,
                        ParticleType::Death,
                        Vec3::new(player_world.0.x, player_world.0.y, 0.06),
                    );
                    commands.entity(player_entity).despawn();
                }
            }
        }

        // Enemies caught in the blast (friendly fire makes bombers chaotic).
        for (entity, enemy_world, mut health, mut knockback) in enemy_query.iter_mut() {
            let d = (enemy_world.0 - expl.center).length();
            if d > expl.radius {
                continue;
            }
            health.0 -= expl.damage;
            statistics.damage_dealt += expl.damage;
            crate::systems::damage_numbers::spawn_damage_number(
                &mut commands,
                enemy_world.0,
                expl.damage,
                crate::systems::damage_numbers::DAMAGE_TO_ENEMY,
            );
            let push = (enemy_world.0 - expl.center).normalize_or_zero();
            if push != Vec2::ZERO {
                knockback.0 += push * expl.knockback;
            }
            commands.entity(entity).insert(HitFlash(Timer::from_seconds(
                tuning::HIT_FLASH_DURATION,
                TimerMode::Once,
            )));
        }

        commands.entity(expl_entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// Boss combined attack pattern
// ---------------------------------------------------------------------------

/// Boss attack pattern: interleaves a radial projectile burst, an occasional
/// telegraphed charge, and periodic summoning of weak adds. Each is on its own
/// cooldown. Movement (slow chase) is handled by `enemy_move`; the charge here
/// temporarily overrides position while dashing.
///
/// Disjoint queries: boss `With<Enemy>, Without<Player>` and read-only player
/// position. Add spawning goes through Commands (no live actor query reuse).
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn boss_ai(
    mut commands: Commands,
    time: Res<Time>,
    tiles: Res<Tiles>,
    scale_factor: Res<ScaleFactor>,
    sprite_texture: Res<SpriteTexture>,
    statistics: Res<Statistics>,
    mut boss_query: Query<
        (
            &mut WorldPos,
            &mut Transform,
            &Awake,
            &Strength,
            &mut BossAttacks,
        ),
        (With<Enemy>, With<Boss>, Without<Player>),
    >,
    player_query: Query<&WorldPos, (With<Player>, Without<Enemy>)>,
) {
    let Some(player_world) = player_query.iter().next() else {
        return;
    };
    let Some((mut boss_world, mut transform, awake, strength, mut atk)) =
        boss_query.iter_mut().next()
    else {
        return;
    };
    if !awake.0 {
        return;
    }
    let dt = time.delta_secs();
    let scale = scale_factor.0;
    let z = world_to_grid(boss_world.0, scale, 0).z;

    atk.burst.tick(time.delta());
    atk.charge_cd.tick(time.delta());
    atk.summon.tick(time.delta());

    // --- Charge sub-state (takes priority while active) ---
    match atk.charge.phase {
        ChargePhase::WindingUp => {
            atk.charge.timer.tick(time.delta());
            let f = atk.charge.timer.fraction();
            transform.scale = Vec3::splat(tuning::BOSS_SCALE * (1.0 + 0.25 * f));
            if atk.charge.timer.is_finished() {
                let dir = (player_world.0 - boss_world.0).normalize_or_zero();
                atk.charge.dir = if dir == Vec2::ZERO { Vec2::new(1.0, 0.0) } else { dir };
                atk.charge.phase = ChargePhase::Dashing;
                atk.charge.timer =
                    Timer::from_seconds(tuning::BOSS_CHARGE_DURATION, TimerMode::Once);
            }
            return;
        }
        ChargePhase::Dashing => {
            atk.charge.timer.tick(time.delta());
            transform.scale = Vec3::splat(tuning::BOSS_SCALE);
            let step = atk.charge.dir * tuning::BOSS_CHARGE_SPEED_TILES * scale * dt;
            let target = boss_world.0 + step;
            if !hits_wall(boss_world.0, target, &tiles, scale, z) {
                apply_move(&mut boss_world, step, &tiles, scale, z);
                transform.translation.x = boss_world.0.x;
                transform.translation.y = boss_world.0.y;
            } else {
                atk.charge.timer = finished_now();
            }
            if atk.charge.timer.is_finished() {
                atk.charge.phase = ChargePhase::Walking;
            }
            return;
        }
        _ => {
            transform.scale = Vec3::splat(tuning::BOSS_SCALE);
        }
    }

    // --- Trigger a charge ---
    if atk.charge_cd.is_finished() {
        atk.charge_cd = Timer::from_seconds(tuning::BOSS_CHARGE_COOLDOWN, TimerMode::Once);
        atk.charge.phase = ChargePhase::WindingUp;
        atk.charge.timer = Timer::from_seconds(tuning::BOSS_CHARGE_WINDUP, TimerMode::Once);
        return;
    }

    // --- Radial projectile burst ---
    if atk.burst.is_finished() {
        atk.burst = Timer::from_seconds(tuning::BOSS_BURST_COOLDOWN, TimerMode::Once);
        let n = tuning::BOSS_BURST_COUNT;
        // Aim the spray roughly toward the player but fan out a full ring.
        let base = (player_world.0 - boss_world.0).y.atan2(
            (player_world.0 - boss_world.0).x,
        );
        for i in 0..n {
            let angle = base + (i as f32) * std::f32::consts::TAU / (n as f32);
            let dir = Vec2::new(angle.cos(), angle.sin());
            spawn_enemy_projectile(
                &mut commands,
                boss_world.0,
                dir,
                scale,
                tuning::BOSS_PROJECTILE_SPEED_TILES,
                tuning::BOSS_PROJECTILE_RANGE_TILES,
                strength.0,
                tuning::ENEMY_KNOCKBACK_TILES,
                tuning::boss_tint(),
            );
        }
    }

    // --- Summon a couple of weak adds (skeletons) near the boss ---
    if atk.summon.is_finished() {
        atk.summon = Timer::from_seconds(tuning::BOSS_SUMMON_COOLDOWN, TimerMode::Once);
        let run_depth = statistics.floors_completed;
        for i in 0..tuning::BOSS_SUMMON_COUNT {
            let angle = (i as f32) * std::f32::consts::TAU / (tuning::BOSS_SUMMON_COUNT as f32);
            let offset = Vec2::new(angle.cos(), angle.sin()) * scale * 1.5;
            let spawn_at = boss_world.0 + offset;
            // Only spawn onto a passable tile.
            let tile = world_to_grid(spawn_at, scale, z);
            if tiles.get(&tile).is_none_or(|c| !c.passable) {
                continue;
            }
            spawn_add(
                &mut commands,
                &sprite_texture,
                spawn_at,
                tile,
                run_depth,
                scale,
            );
        }
    }
}

/// A `Timer` that is already finished (for immediately ending a phase).
fn finished_now() -> Timer {
    let mut t = Timer::from_seconds(0.0, TimerMode::Once);
    t.tick(std::time::Duration::from_secs_f32(0.001));
    t
}

/// Spawns a weak skeleton add for the boss summon. Mirrors the enemy-spawn block
/// in `setup_play` but for a single, already-awake skeleton.
fn spawn_add(
    commands: &mut Commands,
    sprite_texture: &SpriteTexture,
    world: Vec2,
    grid: Position,
    run_depth: i64,
    scale: f32,
) {
    let enemy_type = EnemyType::Skeleton;
    let (health, strength) = enemy_type.get_stats(run_depth);
    let (tint, scale_mult) = tuning::enemy_visual(enemy_type);
    let (image, layout) = &sprite_texture.0;
    let mut sprite = Sprite::from_atlas_image(
        image.clone(),
        TextureAtlas {
            layout: layout.clone(),
            index: enemy_type.sprite_index(),
        },
    );
    sprite.color = tint;
    sprite.custom_size = Some(Vec2::splat(scale * scale_mult));

    let add = commands
        .spawn((
            sprite,
            Transform::from_xyz(world.x, world.y, 0.01),
            Visibility::Visible,
            grid,
            Passable(false),
            WakeZone(std::collections::BTreeSet::new()),
            Awake(true),
            Health(health.max(1)),
            OriginalHealth(health.max(1)),
            Strength(strength.max(1)),
            Enemy,
        ))
        .id();
    commands.entity(add).insert((
        enemy_type,
        AIBehavior::for_enemy_type(enemy_type),
        ActorBaseColor(tint),
        MovementPath { path: None },
        SpriteIndex(enemy_type.sprite_index()),
        ZLevel(0.01),
        WorldPos(world),
        Facing::default(),
        Knockback::default(),
        EnemyAttack {
            telegraph: Timer::from_seconds(tuning::ENEMY_TELEGRAPH, TimerMode::Once),
            cooldown: Timer::from_seconds(tuning::ENEMY_ATTACK_COOLDOWN, TimerMode::Once),
            winding_up: false,
        },
        RepathTimer(Timer::from_seconds(0.0, TimerMode::Once)),
    ));
    // Health bar for the add.
    commands.spawn((
        Sprite {
            color: Color::srgb(0., 1., 0.),
            custom_size: Some(Vec2::new(scale / 2., scale / 8.)),
            ..default()
        },
        Transform::from_xyz(world.x, world.y, 0.05),
        HealthBar(add),
    ));
}

// ---------------------------------------------------------------------------
// Room hazards (spikes / lava DoT)
// ---------------------------------------------------------------------------

/// Damages any actor (player or enemy) standing on a hazard tile, on a per-tile
/// DoT tick. The hazard's own timer paces the damage so standing still on lava
/// isn't instant death but is a steady drain.
///
/// Disjoint queries: hazards (`With<Hazard>`, a Tile entity -- has `Position`
/// but no actor markers), enemies (`With<Enemy>, Without<Player>`), player
/// (`With<Player>, Without<Enemy>`). The hazard query and actor queries never
/// alias the same entity.
#[allow(clippy::type_complexity)]
pub fn hazard_tick(
    mut commands: Commands,
    time: Res<Time>,
    floor: Res<Floor>,
    mut statistics: ResMut<Statistics>,
    mut hazards: Query<(&Position, &mut Hazard), (Without<Enemy>, Without<Player>)>,
    mut enemy_query: Query<
        (&Position, &WorldPos, &mut Health),
        (With<Enemy>, Without<Player>),
    >,
    mut player_query: Query<
        (Entity, &Position, &WorldPos, &mut Health, &Dash),
        (With<Player>, Without<Enemy>),
    >,
) {
    for (hazard_pos, mut hazard) in hazards.iter_mut() {
        hazard.0.tick(time.delta());
        if !hazard.0.is_finished() {
            continue;
        }
        hazard.0 = Timer::from_seconds(tuning::HAZARD_TICK_INTERVAL, TimerMode::Once);
        if hazard_pos.z != floor.0 {
            continue;
        }

        // Player on this hazard tile (i-frames don't save you from lava).
        if let Some((player_entity, player_grid, player_world, mut player_health, _dash)) =
            player_query.iter_mut().next()
        {
            if *player_grid == *hazard_pos {
                player_health.0 -= tuning::HAZARD_DAMAGE;
                statistics.damage_taken += tuning::HAZARD_DAMAGE;
                crate::systems::damage_numbers::spawn_damage_number(
                    &mut commands,
                    player_world.0,
                    tuning::HAZARD_DAMAGE,
                    crate::systems::damage_numbers::DAMAGE_TO_PLAYER,
                );
                commands.entity(player_entity).insert(HitFlash(
                    Timer::from_seconds(tuning::HIT_FLASH_DURATION, TimerMode::Once),
                ));
                if player_health.0 <= 0 {
                    spawn_particle(
                        &mut commands,
                        ParticleType::Death,
                        Vec3::new(player_world.0.x, player_world.0.y, 0.06),
                    );
                    commands.entity(player_entity).despawn();
                }
            }
        }

        // Enemies on this hazard tile.
        for (enemy_grid, enemy_world, mut enemy_health) in enemy_query.iter_mut() {
            if *enemy_grid == *hazard_pos {
                enemy_health.0 -= tuning::HAZARD_DAMAGE;
                crate::systems::damage_numbers::spawn_damage_number(
                    &mut commands,
                    enemy_world.0,
                    tuning::HAZARD_DAMAGE,
                    crate::systems::damage_numbers::DAMAGE_TO_ENEMY,
                );
            }
        }
    }
}
