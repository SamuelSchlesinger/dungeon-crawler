use std::collections::BTreeSet;

use itertools::Itertools;

use crate::components::Position;
use crate::map;

pub fn unbeatable() -> map::Map {
    let border_tile = map::Tile {
        sprite_index: 15 * 64 - 13,
        passable: false,
    };
    map::Map {
        player_sprite: 71,
        room: map::Room {
            initial_position: Position { x: 1, y: 1, z: 0 },
            tiles: (-10i32..10)
                .flat_map(|k| {
                    (-(10 - k.abs())..(10 - k.abs()))
                        .map(|j| (k, j))
                        .collect::<Vec<_>>()
                })
                .flat_map(|(k, j)| {
                    (-(10 - k.abs())..(10 - k.abs()))
                        .map(|i| Position { x: i, y: j, z: k })
                        .collect::<Vec<_>>()
                })
                .map(|position| {
                    (
                        position,
                        map::Tile {
                            sprite_index: 960,
                            passable: true,
                        },
                    )
                })
                .chain(
                    (-10i32..=10)
                        .flat_map(|k| {
                            (-(10 - k.abs())..=(10 - k.abs()))
                                .map(|i| (i, k))
                                .collect::<Vec<_>>()
                        })
                        .map(|(i, k)| {
                            (
                                Position {
                                    x: i,
                                    y: 10 - k.abs(),
                                    z: k,
                                },
                                border_tile.clone(),
                            )
                        }),
                )
                .chain(
                    (-10i32..=10)
                        .flat_map(|k| {
                            (-(10 - k.abs())..=(10 - k.abs()))
                                .map(|i| (i, k))
                                .collect::<Vec<_>>()
                        })
                        .map(|(i, k)| {
                            (
                                Position {
                                    x: i,
                                    y: -(10 - k.abs()),
                                    z: k,
                                },
                                border_tile.clone(),
                            )
                        }),
                )
                .chain(
                    (-10i32..=10)
                        .flat_map(|k| {
                            (-(10 - k.abs())..=(10 - k.abs()))
                                .map(|i| (i, k))
                                .collect::<Vec<_>>()
                        })
                        .map(|(i, k)| {
                            (
                                Position {
                                    x: -(10 - k.abs()),
                                    y: -i,
                                    z: k,
                                },
                                border_tile.clone(),
                            )
                        }),
                )
                .chain(
                    (-10i32..=10)
                        .flat_map(|k| {
                            (-(10 - k.abs())..=(10 - k.abs()))
                                .map(|i| (i, k))
                                .collect::<Vec<_>>()
                        })
                        .map(|(i, k)| {
                            (
                                Position {
                                    x: 10 - k.abs(),
                                    y: i,
                                    z: k,
                                },
                                border_tile.clone(),
                            )
                        }),
                )
                .collect(),
            enemies: (-3..=3)
                .map(|k| {
                    (
                        Position { x: 5, y: 5, z: k },
                        map::Enemy {
                            sprite_index: 74,
                            health: 100,
                            strength: 5,
                            wake_zone: (-3..3)
                                .cartesian_product(-3..3)
                                .map(|(i, j)| Position {
                                    x: 5 + i,
                                    y: 5 + j,
                                    z: k,
                                })
                                .collect::<BTreeSet<_>>(),
                        },
                    )
                })
                .collect(),
        },
        initial_room: 0,
        player_health: 1000,
        player_strength: 2,
        victory_condition: map::VictoryCondition::Arrival(Position {
            x: 100,
            y: 100,
            z: 100,
        }),
    }
}
