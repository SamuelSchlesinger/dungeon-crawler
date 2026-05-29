use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::particle_system::spawn_particle;
use crate::tuning;
use crate::utils::world_to_grid;

/// Continuous, real-time player movement.
///
/// Hold W/A/S/D to move smoothly; diagonals are normalized so they aren't
/// faster. Movement is `velocity * dt` in world units, with circle-vs-tile wall
/// collision resolved per-axis so the player slides along walls instead of
/// sticking. The grid `Position` is re-derived from the continuous `WorldPos`
/// each frame by rounding, which keeps every existing grid system (fog, victory
/// arrival, pathfinding targets, floor visibility) working unchanged.
///
/// Dash bursts and knockback are layered on top of the WASD velocity here so all
/// player translation flows through one collision check.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn move_player(
    mut commands: Commands,
    mut query: Query<
        (
            &mut Position,
            &mut WorldPos,
            &mut Transform,
            &mut Facing,
            &mut Knockback,
            &Dash,
        ),
        With<Player>,
    >,
    mut enemies: Query<(&WakeZone, &mut Awake), With<Enemy>>,
    scale_factor: Res<ScaleFactor>,
    tiles: Res<Tiles>,
    mut floor: ResMut<Floor>,
    player_stats: Res<PlayerStats>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Some((mut position, mut world_pos, mut transform, mut facing, mut knockback, dash)) =
        query.iter_mut().next()
    else {
        return;
    };

    let dt = time.delta_secs();
    let scale = scale_factor.0;

    // Manual floor change (kept for the layered/avoidance maps which use z).
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        position.z += 1;
        floor.0 = position.z;
        // Re-anchor the continuous position onto the new floor's grid tile.
        let c = crate::utils::grid_to_world_center(position.x, position.y, scale);
        world_pos.0 = c;
    } else if keyboard_input.just_pressed(KeyCode::KeyQ) {
        position.z -= 1;
        floor.0 = position.z;
        let c = crate::utils::grid_to_world_center(position.x, position.y, scale);
        world_pos.0 = c;
    }

    // Desired movement direction from WASD.
    let mut dir = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyW) {
        dir.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        dir.y -= 1.0;
    }
    if dir != Vec2::ZERO {
        dir = dir.normalize();
        facing.0 = dir;
    }

    // Compose this frame's displacement: walk + dash burst + decaying knockback.
    // Walk speed is the EFFECTIVE move speed (base x boon modifier).
    let walk = dir * player_stats.effective_move_speed() * scale;
    let dash_vel = if dash.dashing {
        dash.dir * tuning::DASH_SPEED_TILES * scale
    } else {
        Vec2::ZERO
    };
    let delta = (walk + dash_vel + knockback.0) * dt;

    // Decay knockback exponentially.
    knockback.0 *= (1.0 - tuning::KNOCKBACK_DECAY * dt).max(0.0);
    if knockback.0.length_squared() < 1.0 {
        knockback.0 = Vec2::ZERO;
    }

    // Per-axis collision so the player slides along walls instead of stopping.
    let z = position.z;
    let radius = tuning::PLAYER_RADIUS_TILES * scale;

    let mut new_pos = world_pos.0;
    let candidate_x = Vec2::new(new_pos.x + delta.x, new_pos.y);
    if !blocked(candidate_x, radius, &tiles, scale, z) {
        new_pos.x = candidate_x.x;
    }
    let candidate_y = Vec2::new(new_pos.x, new_pos.y + delta.y);
    if !blocked(candidate_y, radius, &tiles, scale, z) {
        new_pos.y = candidate_y.y;
    }
    world_pos.0 = new_pos;

    // Wave 5: leave an afterimage trail while dashing (a faint blue puff at the
    // player's position each dash frame). Particles carry no actor markers, so
    // they are excluded from collision/AI queries (B0001-safe).
    if dash.dashing {
        spawn_particle(
            &mut commands,
            ParticleType::DashTrail,
            Vec3::new(new_pos.x, new_pos.y, 0.015),
        );
    }

    // Drive the visual transform from the continuous position.
    transform.translation.x = new_pos.x;
    transform.translation.y = new_pos.y;

    // Sync the grid position (the substrate every other system reads).
    let new_grid = world_to_grid(new_pos, scale, z);
    if *position != new_grid {
        *position = new_grid;
        floor.0 = new_grid.z;
    }

    // Wake nearby sleeping enemies, as the old grid mover did.
    for (wake_zone, mut wake) in enemies.iter_mut() {
        if wake_zone.0.contains(&position) {
            wake.0 = true;
        }
    }
}

/// True if the player's collision box (a square of half-extent `radius` centered
/// at `world`) would overlap any solid (non-passable) or off-map (untiled) tile.
///
/// Checks EVERY grid tile the box overlaps, not just a few sample points. The old
/// 4-cardinal-point probe missed wall corners on the diagonal, letting the player
/// clip through corners and escape the dungeon into the empty void. Enumerating
/// the covered tiles is corner-proof for any radius.
fn blocked(world: Vec2, radius: f32, tiles: &Tiles, scale: f32, z: i64) -> bool {
    let min = world_to_grid(Vec2::new(world.x - radius, world.y - radius), scale, z);
    let max = world_to_grid(Vec2::new(world.x + radius, world.y + radius), scale, z);
    for gx in min.x..=max.x {
        for gy in min.y..=max.y {
            let tile = Position::new(gx, gy, z);
            if tiles.get(&tile).is_none_or(|cached| !cached.passable) {
                return true;
            }
        }
    }
    false
}
