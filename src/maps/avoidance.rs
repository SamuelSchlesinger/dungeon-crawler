use crate::components::Position;
use crate::map::*;

pub fn avoidance() -> Map {
    let mut room = Room::new(Position::new(0, 0, 0));
    let victory_position = Position::new(20, 5, 0);

    for x in 0..=20 {
        for y in 0..=10 {
            if x == victory_position.x && y == victory_position.y {
                room.add_tile(Position::new(x, y, 0), Tile::new(960 + 64 + 30, true));
            } else {
                room.add_tile(Position::new(x, y, 0), Tile::new(960, true));
            }
        }
    }

    for x in -1..=21 {
        room.add_tile(Position::new(x, 11, 0), Tile::new(15 * 64 - 13, false));
        room.add_tile(Position::new(x, -1, 0), Tile::new(15 * 64 - 13, false));
    }

    for y in -1..=11 {
        room.add_tile(Position::new(21, y, 0), Tile::new(15 * 64 - 13, false));
        room.add_tile(Position::new(-1, y, 0), Tile::new(15 * 64 - 13, false));
    }

    for y in 0..=10 {
        let enemy_position = Position::new(15, y, 0);

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

    Map {
        room,
        player_health: 200,
        player_strength: 0,
        player_sprite: 71,
        victory_condition: VictoryCondition::Arrival(Position::new(20, 5, 0)),
    }
}
