use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::ops::Add;

use bevy::prelude::*;
use positioning::pathfinding;
use positioning::pathfinding::Heuristic;
use priority_queue::DoublePriorityQueue;

use crate::components::*;
use crate::resources::*;

// TODO Make sure enemies don't collide, cause if they do they'll never come unstuck
// NB Maybe they can't already?
pub fn walk_enemies(
    tiles: Res<Tiles>,
    mut enemies_query: Query<
        (Entity, &mut Position, &Awake, &mut MovementPath),
        (With<Enemy>, Without<Player>),
    >,
    mut enemies: ResMut<Enemies>,
    player: Query<&Position, With<Player>>,
) {
    if let Some(player_position) = player.iter().next() {
        for (entity, mut position, awake, mut movement_path) in enemies_query.iter_mut() {
            if awake.0 {
                // attack!
                if position.is_adjacent_to(*player_position) {
                    return;
                }
                // random motion
                if rand::random() && rand::random() && rand::random() {
                    let potential_positions: Vec<Position> = position
                        .adjacent()
                        .filter(|neighbor| {
                            tiles
                                .get(&neighbor)
                                .map_or_else(|| false, |cached_tile| cached_tile.passable)
                                && enemies
                                    .enemies_at(*neighbor)
                                    .map_or_else(|| true, |s| s.is_empty())
                                && movement_path
                                    .path
                                    .iter()
                                    .map(|path| {
                                        path.get(0).map_or_else(
                                            || true,
                                            |intended_location| {
                                                neighbor.is_adjacent_to(*intended_location)
                                            },
                                        )
                                    })
                                    .all(|e| e)
                        })
                        .collect();
                    if !potential_positions.is_empty() {
                        *position = potential_positions
                            [rand::random::<usize>() % potential_positions.len()];
                        enemies.insert(*position, entity);
                        return;
                    }
                }
                if movement_path.age >= 5
                    || match &movement_path.path {
                        None => true,
                        Some(path) => path.is_empty(),
                    }
                {
                    movement_path.path =
                        find_shortest_path(&tiles, &mut enemies, *position, *player_position);
                    movement_path.age = 0;
                }
                movement_path.age += rand::random::<usize>() % 3;
                if let Some(ref mut path) = &mut movement_path.path {
                    if let Some(next_vertex) = path.pop_front() {
                        let adjacency = position.is_adjacent_to(next_vertex);
                        let next_vertex_is_passable = tiles
                            .get(&next_vertex)
                            .map_or_else(|| false, |cached_tile| cached_tile.passable);
                        if adjacency
                            && next_vertex_is_passable
                            && enemies.enemies_at(next_vertex).map_or_else(
                                || true,
                                |set| set.contains(&entity) && set.len() == 1 || set.is_empty(),
                            )
                        {
                            *position = next_vertex;
                            enemies.insert(next_vertex, entity);
                        } else {
                            movement_path.path = None;
                        }
                    } else {
                        movement_path.path = None;
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Copy, Clone)]
enum WithInfinity<I> {
    Normal(I),
    Infinity,
}

impl<I> Default for WithInfinity<I> {
    fn default() -> Self {
        WithInfinity::Infinity
    }
}

impl<I: Add<I, Output = I>> Add<WithInfinity<I>> for WithInfinity<I> {
    type Output = WithInfinity<I>;

    fn add(self, rhs: WithInfinity<I>) -> Self::Output {
        match self {
            WithInfinity::Normal(i) => match rhs {
                WithInfinity::Infinity => WithInfinity::Infinity,
                WithInfinity::Normal(j) => WithInfinity::Normal(i + j),
            },
            WithInfinity::Infinity => WithInfinity::Infinity,
        }
    }
}

#[test]
fn with_infinity_test() {
    let x = WithInfinity::Normal(1i64);
    let z = WithInfinity::Normal(0i64);
    let y = WithInfinity::Infinity;
    assert!(y > x);
    assert!(z < x);
    assert!(x + z < y);
}

fn find_shortest_path(
    tiles: &Tiles,
    enemies: &Enemies,
    starting_position: Position,
    ending_position: Position,
) -> Option<VecDeque<Position>> {
    fn dist_2d(a: Position, b: Position) -> i64 {
        ((a.x.abs_diff(b.x).pow(2) + a.y.abs_diff(b.y).pow(2) + a.z.abs_diff(b.z).pow(2)) as f32)
            .sqrt() as i64
    }
    let all_passable_tile_positions: BTreeSet<Position> = tiles
        .0
        .iter()
        .filter_map(|(position, cached_tile)| {
            if cached_tile.passable && !enemies.occupied_position(*position) {
                Some(position)
            } else {
                None
            }
        })
        .copied()
        .collect();
    pathfinding::HammingDistance.find_shortest_path(
        &all_passable_tile_positions,
        starting_position,
        ending_position,
    )
}
