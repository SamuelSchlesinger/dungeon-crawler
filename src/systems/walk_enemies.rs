use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::VecDeque;

use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

pub fn walk_enemies(
    tiles: Res<Tiles>,
    mut enemies: Query<(&mut Position, &Awake, &mut MovementPath), With<Enemy>>,
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

fn find_shortest_path(
    tiles: &Tiles,
    starting_position: Position,
    ending_position: Position,
) -> Option<VecDeque<Position>> {
    let mut distances_from_start: BTreeMap<Position, i32> = BTreeMap::new();
    let all_passable_tile_positions: Vec<Position> = tiles
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
    distances_from_start.insert(starting_position, 0);

    loop {
        // TODO Find shortest path, feeling lazy rn
    }
}
