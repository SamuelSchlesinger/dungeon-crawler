use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::BinaryHeap;
use std::collections::VecDeque;
use std::ops::Add;

use bevy::prelude::*;
use priority_queue::DoublePriorityQueue;

use crate::components::*;
use crate::resources::*;

pub fn walk_enemies(
    tiles: Res<Tiles>,
    mut enemies: Query<(&mut Position, &Awake, &mut MovementPath), (With<Enemy>, Without<Player>)>,
    player: Query<&Position, With<Player>>,
) {
    println!("Walking enemies");
    for (mut position, awake, mut movement_path) in enemies.iter_mut() {
        match &mut movement_path.vertices {
            Some(ref mut path) => match path.pop_front() {
                Some(next_vertex) => {
                    let adjacency = position.is_adjacent_to(next_vertex);
                    let next_vertex_is_passable = tiles
                        .get(&(next_vertex.x, next_vertex.y, next_vertex.z))
                        .map_or_else(|| false, |cached_tile| cached_tile.passable);
                    if adjacency && next_vertex_is_passable {
                        *position = next_vertex;
                    } else {
                        movement_path.vertices = None;
                    }
                }
                None => {
                    movement_path.vertices = None;
                }
            },
            None => {
                if awake.0 {
                    if let Some(player_position) = player.iter().next() {
                        movement_path.vertices =
                            find_shortest_path(&tiles, *position, *player_position);
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
    let x = WithInfinity::Normal(1i32);
    let y = WithInfinity::Infinity;
    assert!(y > x);
}

fn find_shortest_path(
    tiles: &Tiles,
    starting_position: Position,
    ending_position: Position,
) -> Option<VecDeque<Position>> {
    let all_passable_tile_positions: BTreeSet<Position> = tiles
        .0
        .iter()
        .filter_map(|(position, cached_tile)| {
            if cached_tile.passable {
                Some(position)
            } else {
                None
            }
        })
        .copied()
        .map(|(x, y, z)| Position { x, y, z })
        .collect();
    let mut distances_from_start: BTreeMap<Position, WithInfinity<i32>> = BTreeMap::new();
    let mut predecessor: BTreeMap<Position, Option<Position>> = BTreeMap::new();
    let mut queue: DoublePriorityQueue<Position, WithInfinity<i32>> = DoublePriorityQueue::new();
    for position in all_passable_tile_positions.iter() {
        if *position != starting_position {
            distances_from_start.insert(*position, WithInfinity::Infinity);
            predecessor.insert(*position, None);
            queue.push(*position, WithInfinity::Infinity);
        }
    }

    loop {
        match queue.pop_max() {
            None => {
                break;
            }
            Some((position, distance)) => {
                for v in position
                    .adjacent()
                    .filter(|v| all_passable_tile_positions.contains(v))
                {
                    let alt = distance + WithInfinity::Normal(1);
                    if alt < *distances_from_start.get(&position).expect("boo!") {
                        distances_from_start.insert(position, alt);
                        predecessor.insert(v, Some(position));
                        queue.change_priority(&v, alt);
                    }
                }
            }
        }
    }

    if let Some(_distance) = distances_from_start.get(&ending_position) {
        let mut current_position = ending_position;
        let mut path = VecDeque::new();
        loop {
            if current_position == starting_position {
                break;
            }
            path.push_front(current_position);
            if let Some(Some(pre)) = predecessor.get(&current_position) {
                current_position = *pre;
            } else {
                panic!("should always have a path home");
            }
        }
        Some(path)
    } else {
        panic!("it should always at least be Infinity");
    }
}
