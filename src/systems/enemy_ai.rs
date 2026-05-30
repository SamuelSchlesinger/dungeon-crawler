use std::collections::BTreeSet;
use std::collections::VecDeque;

use bevy::prelude::*;
use positioning::pathfinding;
use positioning::pathfinding::Heuristic;

use crate::components::*;
use crate::resources::*;
use crate::tuning;
use crate::utils::{grid_to_world_center, world_to_grid};

/// Real-time enemy movement.
///
/// Awake enemies move continuously toward the player. Pathing is done on the
/// tile grid (the existing A*), but instead of teleporting one tile per tick the
/// enemy steers smoothly toward the next waypoint's world center, re-pathing on
/// a timer. The grid `Position` is re-derived by rounding each frame so fog,
/// visibility, and pathfinding occupancy stay correct. Ghosts keep their
/// distance (defensive behavior) when healthy-but-close handling differs per
/// type via `enemy_speed_tiles`.
///
/// Disjoint queries: enemies (`With<Enemy>, Without<Player>`) and player
/// (`With<Player>, Without<Enemy>`) never overlap.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn enemy_move(
    time: Res<Time>,
    tiles: Res<Tiles>,
    scale_factor: Res<ScaleFactor>,
    mut enemies_res: ResMut<Enemies>,
    mut enemy_query: Query<
        (
            Entity,
            &mut Position,
            &mut WorldPos,
            &mut Transform,
            &mut Facing,
            &mut Knockback,
            &Awake,
            &mut MovementPath,
            &mut RepathTimer,
            &AIBehavior,
            &EnemyType,
            &Health,
            &OriginalHealth,
            Option<&ChargeState>,
            Option<&BossAttacks>,
        ),
        (With<Enemy>, Without<Player>),
    >,
    player_query: Query<(&WorldPos, &Position), (With<Player>, Without<Enemy>)>,
) {
    let Some((player_world, player_grid)) = player_query.iter().next() else {
        return;
    };
    let dt = time.delta_secs();
    let scale = scale_factor.0;

    // Snapshot for pathfinding occupancy (avoids borrowing the resource mutably
    // while iterating the grid). Rebuilt cheaply each frame.
    for (
        entity,
        mut position,
        mut world_pos,
        mut transform,
        mut facing,
        mut knockback,
        awake,
        mut movement_path,
        mut repath,
        ai_behavior,
        enemy_type,
        health,
        original_health,
        charge_state,
        boss_attacks,
    ) in enemy_query.iter_mut()
    {
        // Always apply (and decay) knockback so even idle enemies get shoved.
        let knock = knockback.0;
        knockback.0 *= (1.0 - tuning::KNOCKBACK_DECAY * dt).max(0.0);
        if knockback.0.length_squared() < 1.0 {
            knockback.0 = Vec2::ZERO;
        }

        if !awake.0 {
            apply_move(&mut world_pos, knock * dt, &tiles, scale, position.z);
            sync(&mut position, &mut transform, &world_pos, &mut enemies_res, entity, scale);
            continue;
        }

        // Wave 4: the charger and boss drive their OWN position while their charge
        // state machine is active (windup / dash / recover). Suppress the default
        // chase steering for those phases so the two systems never fight over
        // WorldPos (they still get knockback applied below, but no chase delta).
        let self_driven = match ai_behavior {
            AIBehavior::Charging => charge_state
                .map(|c| c.phase != ChargePhase::Walking)
                .unwrap_or(false),
            AIBehavior::BossPattern => boss_attacks
                .map(|b| b.charge.phase != ChargePhase::Walking)
                .unwrap_or(false),
            _ => false,
        };
        if self_driven {
            // Only apply knockback; the special AI moves the actor.
            apply_move(&mut world_pos, knock * dt, &tiles, scale, position.z);
            sync(&mut position, &mut transform, &world_pos, &mut enemies_res, entity, scale);
            continue;
        }

        let health_fraction = health.0 as f32 / original_health.0 as f32;
        let dist_tiles = (player_world.0 - world_pos.0).length() / scale;

        // Behavior: defensive enemies retreat at low HP / keep distance when close;
        // archers (Kiting) keep their preferred range, retreating when too close.
        let should_retreat = (matches!(ai_behavior, AIBehavior::Defensive)
            && (health_fraction < 0.3 || dist_tiles < 2.5))
            || (matches!(ai_behavior, AIBehavior::Kiting)
                && dist_tiles < tuning::ARCHER_RETREAT_RANGE);
        let should_chase = match ai_behavior {
            AIBehavior::Aggressive => true,
            AIBehavior::Defensive => !should_retreat,
            AIBehavior::Patrol => dist_tiles < 6.0,
            // Archer: only close the gap if it's farther than its preferred range.
            AIBehavior::Kiting => !should_retreat && dist_tiles > tuning::ARCHER_PREFERRED_RANGE,
            // Bomber: always rush. Charger (Walking) + Boss (slow): chase.
            AIBehavior::Exploding => true,
            AIBehavior::Charging => true,
            AIBehavior::BossPattern => true,
        };

        let speed = tuning::enemy_speed_tiles(*enemy_type) * scale;

        // Re-path on a timer (or when the path is empty/missing).
        repath.0.tick(time.delta());
        let need_path = repath.0.is_finished()
            || movement_path.path.as_ref().is_none_or(|p| p.is_empty());
        if need_path {
            repath.0 = Timer::from_seconds(tuning::ENEMY_REPATH_INTERVAL, TimerMode::Once);
            movement_path.path = if should_chase {
                find_shortest_path(&tiles, *position, *player_grid)
            } else if should_retreat {
                // Pick a passable tile directly away from the player.
                let away = Position {
                    x: position.x + (position.x - player_grid.x).signum(),
                    y: position.y + (position.y - player_grid.y).signum(),
                    z: position.z,
                };
                find_shortest_path(&tiles, *position, away)
            } else {
                None
            };
        }

        // Determine the steering target (world center of the next waypoint).
        let mut target_world: Option<Vec2> = None;
        if let Some(ref mut path) = movement_path.path {
            // Drop waypoints we have effectively reached.
            while let Some(front) = path.front().copied() {
                let center = grid_to_world_center(front.x, front.y, scale);
                if (center - world_pos.0).length() < scale * 0.35 {
                    path.pop_front();
                } else {
                    target_world = Some(center);
                    break;
                }
            }
        }

        // Aggressive/closing enemies that have line of nothing just home straight
        // in if no path waypoint is available but the player is close.
        if target_world.is_none() && should_chase && dist_tiles < 1.6 {
            target_world = Some(player_world.0);
        }

        let mut delta = knock * dt;
        if let Some(target) = target_world {
            let dir = (target - world_pos.0).normalize_or_zero();
            if dir != Vec2::ZERO {
                facing.0 = dir;
                delta += dir * speed * dt;
            }
        }

        apply_move(&mut world_pos, delta, &tiles, scale, position.z);
        sync(&mut position, &mut transform, &world_pos, &mut enemies_res, entity, scale);
    }
}

/// Per-axis collision against solid tiles, applied to a continuous position.
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
    // Box-overlap (every tile the actor's box covers), matching the player's
    // collision. The old 4-cardinal-probe missed wall corners on the diagonal,
    // letting enemies clip through corners.
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

/// Re-derive grid `Position`, drive the transform, and keep the `Enemies`
/// occupancy resource in sync with the new tile.
fn sync(
    position: &mut Position,
    transform: &mut Transform,
    world_pos: &WorldPos,
    enemies_res: &mut Enemies,
    entity: Entity,
    scale: f32,
) {
    transform.translation.x = world_pos.0.x;
    transform.translation.y = world_pos.0.y;
    let new_grid = world_to_grid(world_pos.0, scale, position.z);
    if *position != new_grid {
        *position = new_grid;
    }
    enemies_res.insert(*position, entity);
}

fn find_shortest_path(
    tiles: &Tiles,
    starting_position: Position,
    ending_position: Position,
) -> Option<VecDeque<Position>> {
    // Path over ALL passable tiles, ignoring enemy occupancy. Enemies don't
    // physically collide with each other, and excluding occupied tiles made one
    // enemy standing in a 1-tile-wide corridor block the A* path of every enemy
    // behind it -- the whole pack stalled. Occupancy is irrelevant to reachability.
    let all_passable_tile_positions: BTreeSet<Position> = tiles
        .0
        .iter()
        .filter_map(|(position, cached_tile)| {
            if cached_tile.passable {
                Some(*position)
            } else {
                None
            }
        })
        .collect();
    pathfinding::HammingDistance.find_shortest_path(
        &all_passable_tile_positions,
        starting_position,
        ending_position,
    )
}
