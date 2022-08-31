use itertools::Itertools;
use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, BTreeSet};

use crate::components::Position;

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Room {
    pub initial_position: Position,
    pub tiles: PositionMap<Tile>,
    pub enemies: PositionMap<Enemy>,
}

impl Room {
    pub fn new(initial_position: Position) -> Self {
        Room {
            initial_position,
            tiles: PositionMap(BTreeMap::new()),
            enemies: PositionMap(BTreeMap::new()),
        }
    }

    pub fn add_tile(&mut self, position: Position, tile: Tile) -> &mut Self {
        self.tiles.0.insert(position, tile);
        self
    }

    pub fn add_enemy(&mut self, position: Position, enemy: Enemy) -> &mut Self {
        self.enemies.0.insert(position, enemy);
        self
    }
}

#[derive(Deserialize, Serialize, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct Tile {
    pub sprite_index: u32,
    pub passable: bool,
}

impl Tile {
    pub fn new(sprite_index: u32, passable: bool) -> Self {
        Tile {
            sprite_index,
            passable,
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct Enemy {
    pub sprite_index: u32,
    pub health: u32,
    pub strength: u32,
    pub wake_zone: BTreeSet<Position>,
}

impl Enemy {
    pub fn new(
        sprite_index: u32,
        health: u32,
        strength: u32,
        wake_zone: BTreeSet<Position>,
    ) -> Self {
        Enemy {
            sprite_index,
            health,
            strength,
            wake_zone,
        }
    }

    pub fn circular_wake_zone(center: Position, radius: i32) -> BTreeSet<Position> {
        (-radius..=radius)
            .cartesian_product(-radius..=radius)
            .filter_map(|(dx, dy)| {
                if (dx.pow(2) as f32 + dy.pow(2) as f32).sqrt() <= radius as f32 {
                    Some(Position {
                        x: center.x + dx,
                        y: center.y + dy,
                        z: center.z,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Deserialize, Serialize, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct ItemId(u32);

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Map {
    pub room: Room,
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
