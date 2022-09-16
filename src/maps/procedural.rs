use crate::{
    components::Position,
    map::{Map, Room, VictoryCondition},
};

// TODO Procedurally generate a map
pub fn procedural() -> Map {
    let mut room = Room::new(Position { x: 0, y: 0, z: 0 });
    let mut victory_condition = VictoryCondition::Unwinnable;

    Map {
        player_health: compute_reasonable_player_health(&room),
        player_strength: compute_reasonable_player_strength(&room),
        room,
        player_sprite: 32 * 64 + 45,
        victory_condition,
    }
}

// TODO
fn compute_reasonable_player_strength(room: &Room) -> u64 {
    10
}

// TODO
fn compute_reasonable_player_health(room: &Room) -> u64 {
    10
}
