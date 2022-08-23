use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl From<Vec2> for Position {
    fn from(v: Vec2) -> Self {
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
        }
    }
}

#[derive(Component)]
pub struct ZLevel(pub f32);

#[derive(Component)]
pub struct SpriteIndex(pub usize);

#[derive(Component)]
pub struct Tile;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Camera;
