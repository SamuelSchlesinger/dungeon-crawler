use bevy::prelude::*;

pub use positioning::Position;

use std::{
    collections::{BTreeSet, VecDeque},
};

#[derive(Component, Debug)]
pub struct HealthBar(pub Entity);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Strength(pub i64);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Health(pub i64);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct OriginalHealth(pub i64);

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
pub struct CameraMarker;

#[derive(Component, Debug)]
pub struct Passable(pub bool);

#[test]
fn test_adjacency() {
    let position = Position { x: 5, y: 5, z: 0 };
    let other = Position { x: 4, y: 5, z: 0 };
    assert!(position.is_adjacent_to(other));
}

#[derive(Component, Debug)]
pub struct MovementPath {
    pub age: usize,
    pub path: Option<VecDeque<Position>>,
}

#[derive(Component)]
pub struct Menu;

#[derive(Component)]
pub struct HealthGain;
