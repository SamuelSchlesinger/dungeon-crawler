use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, BTreeSet};

use crate::components::Position;

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Room {
    pub initial_position: Position,
    pub tiles: PositionMap<Tile>,
    pub enemies: PositionMap<Enemy>,
}

#[derive(Deserialize, Serialize, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct Tile {
    pub sprite_index: u32,
    pub passable: bool,
}

#[derive(Deserialize, Serialize, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct Enemy {
    pub sprite_index: u32,
    pub health: u32,
    pub strength: u32,
    pub wake_zone: BTreeSet<Position>,
}

#[derive(Deserialize, Serialize, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct ItemId(u32);

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Map {
    pub room: Room,
    pub initial_room: u16,
    pub player_health: u32,
    pub player_strength: u32,
    pub player_sprite: u32,
    pub victory_condition: VictoryCondition,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum VictoryCondition {
    Arrival(Position),
    Extermination,
    Or(Vec<VictoryCondition>),
    And(Vec<VictoryCondition>),
}

#[derive(PartialEq, Eq, Clone)]
pub struct PositionMap<A>(BTreeMap<Position, A>);

impl<'a, A> IntoIterator for &'a PositionMap<A> {
    type Item = (&'a Position, &'a A);

    type IntoIter = std::collections::btree_map::Iter<'a, Position, A>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<A> IntoIterator for PositionMap<A> {
    type Item = (Position, A);

    type IntoIter = std::collections::btree_map::IntoIter<Position, A>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<A> FromIterator<(Position, A)> for PositionMap<A> {
    fn from_iter<T: IntoIterator<Item = (Position, A)>>(iter: T) -> Self {
        PositionMap(FromIterator::from_iter(iter))
    }
}

impl<A: Serialize + Clone> Serialize for PositionMap<A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v: Vec<_> = self
            .0
            .iter()
            .map(|(Position { x, y, z }, a)| {
                (
                    Position {
                        x: *x,
                        y: *y,
                        z: *z,
                    },
                    a.clone(),
                )
            })
            .collect();
        v.serialize(serializer)
    }
}

impl<'de, A: Deserialize<'de>> Deserialize<'de> for PositionMap<A> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v: Vec<_> = Deserialize::deserialize(deserializer)?;
        Ok(PositionMap(v.into_iter().collect()))
    }
}
