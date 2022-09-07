use crate::components::Position;
use crate::map;
use itertools::Itertools;

pub fn unbeatable() -> map::Map {
    let border_tile = map::Tile {
        sprite_index: 15 * 64 - 13,
        passable: false,
    };
    map::Map {
        player_sprite: 31 * 64 + 20,
        room: map::Room {
            initial_position: Position { x: 0, y: 0, z: -9 },
            healths: vec![(
                Position::new(1, 1, 0),
                map::Health {
                    sprite_index: 64 * 23 + 45,
                    health: 10,
                },
            )]
            .into_iter()
            .collect(),
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
                .flat_map(|k| {
                    let make_enemy = |x: i32, y: i32, z: i32| {
                        (
                            Position { x, y, z },
                            map::Enemy {
                                sprite_index: 74,
                                health: 100,
                                strength: 10,
                                wake_zone: (-6..=6)
                                    .cartesian_product(-6..=6)
                                    .cartesian_product(-6..=6)
                                    .map(|((dx, dy), dz)| Position {
                                        x: x + dx,
                                        y: y + dy,
                                        z: k + dz,
                                    })
                                    .collect(),
                            },
                        )
                    };
                    vec![
                        make_enemy(5, 5, k),
                        make_enemy(4, 5, k),
                        make_enemy(5, 4, k),
                        make_enemy(-5, 5, k),
                        make_enemy(-4, 5, k),
                        make_enemy(-5, 4, k),
                        make_enemy(-5, -5, k),
                        make_enemy(-4, -5, k),
                        make_enemy(-5, -4, k),
                        make_enemy(5, -5, k),
                        make_enemy(4, -5, k),
                        make_enemy(5, -4, k),
                    ]
                })
                .collect(),
        },
        player_health: 4000,
        player_strength: 10,
        victory_condition: map::VictoryCondition::Or(vec![
            map::VictoryCondition::Extermination,
            map::VictoryCondition::Arrival(Position { x: 0, y: 0, z: 9 }),
        ]),
    }
}
