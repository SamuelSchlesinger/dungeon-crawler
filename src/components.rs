use bevy::prelude::*;
use itertools::Itertools;

use std::{collections::BTreeSet, ops::Add};

#[derive(Component, Debug)]
pub struct TextOverEntity(pub Entity);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Strength(pub i32);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Health(pub i32);

#[derive(
    Component,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Position {
    pub fn is_adjacent_to(self, other: Position) -> bool {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y) + self.z.abs_diff(other.z) == 1
    }

    pub fn adjacent(self) -> Box<dyn Iterator<Item = Position>> {
        Box::new(
            (-1..=1)
                .cartesian_product(-1..=1)
                .cartesian_product(-1..=1)
                .map(move |((dx, dy), dz)| Position {
                    x: self.x + dx,
                    y: self.y + dy,
                    z: self.z + dz,
                })
                .filter(move |pos| self.is_adjacent_to(*pos)),
        )
    }
}

#[test]
fn position_adjacency_test() {
    let a = Position { x: 1, y: 1, z: 1 };
    let b = Position { x: 2, ..a };
    assert!(a.is_adjacent_to(b));
    let c = Position { y: 2, ..a };
    assert!(a.is_adjacent_to(c));
    let d = Position { z: 2, ..a };
    assert!(a.is_adjacent_to(d));
    let e = Position { y: 2, ..b };
    assert!(!a.is_adjacent_to(e));
    let f = Position { z: 2, ..b };
    assert!(!a.is_adjacent_to(f));

    let adjacent_results: BTreeSet<Position> = a.adjacent().collect();
    let ground_truth: BTreeSet<Position> = vec![
        Position { x: 2, y: 1, z: 1 },
        Position { x: 0, y: 1, z: 1 },
        Position { x: 1, y: 2, z: 1 },
        Position { x: 1, y: 0, z: 1 },
        Position { x: 1, y: 1, z: 2 },
        Position { x: 1, y: 1, z: 0 },
    ]
    .into_iter()
    .collect();
    assert_eq!(adjacent_results, ground_truth);
}

impl From<Vec3> for Position {
    fn from(v: Vec3) -> Self {
        fn convert(f: f32) -> i32 {
            if f < 0. {
                -(-f as i32)
            } else {
                f as i32 + 1
            }
        }
        Position {
            x: convert(v.x),
            y: convert(v.y),
            z: convert(v.z),
        }
    }
}

impl Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

#[derive(Component, Debug)]
pub struct WakeZone(pub BTreeSet<Position>);

#[derive(Component, Debug)]
pub struct Awake(pub bool);

#[derive(Component, Debug)]
pub struct ZLevel(pub f32);

#[derive(Component, Debug)]
pub struct SpriteIndex(pub usize);

#[derive(Component, Debug)]
pub struct Tile;

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Camera;

#[derive(Component, Debug)]
pub struct Passable(pub bool);

#[test]
fn test_adjacency() {
    let position = Position { x: 5, y: 5, z: 0 };
    let other = Position { x: 4, y: 5, z: 0 };
    assert!(position.is_adjacent_to(other));
}
