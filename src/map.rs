use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Room {
    pub initial_position: (i32, i32, i32),
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
    pub wake_zone: BTreeSet<(i32, i32, i32)>,
}

#[derive(Deserialize, Serialize, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct ItemId(u32);

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Map {
    pub rooms: Vec<Room>,
    pub initial_room: u16,
    pub player_health: u32,
    pub player_strength: u32,
    pub player_sprite: u32,
}

#[derive(PartialEq, Eq, Clone)]
pub struct PositionMap<A>(BTreeMap<(i32, i32, i32), A>);

impl<'a, A> IntoIterator for &'a PositionMap<A> {
    type Item = (&'a (i32, i32, i32), &'a A);

    type IntoIter = std::collections::btree_map::Iter<'a, (i32, i32, i32), A>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<A> IntoIterator for PositionMap<A> {
    type Item = ((i32, i32, i32), A);

    type IntoIter = std::collections::btree_map::IntoIter<(i32, i32, i32), A>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<A> FromIterator<((i32, i32, i32), A)> for PositionMap<A> {
    fn from_iter<T: IntoIterator<Item = ((i32, i32, i32), A)>>(iter: T) -> Self {
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
            .map(|((i, j, k), a)| ((*i, *j, *k), a.clone()))
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
