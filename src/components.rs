use bevy::prelude::*;

use std::collections::{BTreeSet, VecDeque};

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct MovementPath {
    pub vertices: Option<VecDeque<Position>>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Position {
    pub fn is_adjacent_to(self, other: Position) -> bool {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y) + self.z.abs_diff(other.z) == 1
    }
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

#[derive(Component, Debug)]
pub struct WakeZone(pub BTreeSet<(i32, i32, i32)>);

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
