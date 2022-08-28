use bevy::prelude::*;

use std::collections::BTreeSet;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
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
