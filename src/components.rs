use bevy::prelude::*;

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
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
