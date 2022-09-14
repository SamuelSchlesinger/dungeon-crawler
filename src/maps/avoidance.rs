use crate::components::Position;
use crate::map::*;

pub fn avoidance() -> Map {
    const N_FLOORS: i32 = 10;
    let mut room = Room::new(Position::new(5, 5, 0));
    let victory_position = Position::new(0, 5, N_FLOORS * 2 - 1);

    for z in 0..(N_FLOORS * 2) {
        if z % 2 == 0 {
            for x in 0..=20 {
                for y in 0..=10 {
                    if Position::new(x, y, z) == victory_position {
                        room.add_tile(Position::new(x, y, z), Tile::new(960 + 64 + 30, true));
                    } else if z % 4 == 0 && x == 20 && y == 5 {
                        room.add_tile(Position::new(x, y, z), Tile::new(64 * 15 + 42, true));
                    } else if z % 4 == 2 && x == 0 && y == 5 {
                        room.add_tile(Position::new(x, y, z), Tile::new(64 * 15 + 42, true));
                    } else if z % 4 == 0 && x == 0 && y == 5 && z > 0 {
                        room.add_tile(Position::new(x, y, z), Tile::new(64 * 15 + 41, true));
                    } else if z % 4 == 2 && x == 20 && y == 5 {
                        room.add_tile(Position::new(x, y, z), Tile::new(64 * 15 + 41, true));
                    } else {
                        room.add_tile(Position::new(x, y, z), Tile::new(960, true));
                    }
                    if x == 10 && y == 5 {
                        room.add_health(
                            Position::new(x, y, z),
                            Health {
                                sprite_index: 64 * 23 + 45,
                                health: 100,
                            },
                        );
                    }
                }
            }

            for x in -1..=21 {
                room.add_tile(Position::new(x, 11, z), Tile::new(15 * 64 - 13, false));
                room.add_tile(Position::new(x, -1, z), Tile::new(15 * 64 - 13, false));
            }

            for y in -1..=11 {
                room.add_tile(Position::new(21, y, z), Tile::new(15 * 64 - 13, false));
                room.add_tile(Position::new(-1, y, z), Tile::new(15 * 64 - 13, false));
            }

            for y in 0..=10 {
                let enemy_position = Position::new(if z % 4 == 0 { 15 } else { 5 }, y, z);

                if y % 2 == 0 {
                    room.add_enemy(
                        enemy_position.clone(),
                        Enemy::new(
                            74,
                            100,
                            100,
                            Enemy::circular_wake_zone(enemy_position.clone(), 5),
                        ),
                    );
                }
            }
        } else {
            for x in 0..=20 {
                for y in 0..=10 {
                    if z % 4 == 1 && x == 20 && y == 5 {
                        room.add_tile(Position::new(x, y, z), Tile::new(64 * 15 + 42, true));
                    } else if z % 4 == 3 && x == 0 && y == 5 {
                        room.add_tile(Position::new(x, y, z), Tile::new(64 * 15 + 42, true));
                    } else {
                        room.add_tile(Position::new(x, y, z), Tile::new(15 * 64 - 13, false));
                    }
                }
            }
        }
    }

    Map {
        room,
        player_health: 1000,
        player_strength: 20,
        player_sprite: 31 * 64 + 20,
        victory_condition: VictoryCondition::Arrival(victory_position),
    }
}
