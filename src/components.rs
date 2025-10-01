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

#[derive(Component, Debug, Clone, Copy)]
pub enum AIBehavior {
    Aggressive,   // Always chase player
    Defensive,    // Retreat when health < 30%
    Patrol,       // Random movement, chase when close
}

impl AIBehavior {
    pub fn for_enemy_type(enemy_type: EnemyType) -> Self {
        match enemy_type {
            EnemyType::Skeleton => AIBehavior::Aggressive,
            EnemyType::Orc => AIBehavior::Patrol,
            EnemyType::Ghost => AIBehavior::Defensive,
        }
    }
}

#[derive(Component, Debug)]
pub struct ZLevel(pub f32);

#[derive(Component, Debug)]
pub struct SpriteIndex(pub usize);

#[derive(Component, Debug)]
pub struct Tile;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    Skeleton,  // Fast, weak (sprite: 2700)
    Orc,       // Balanced (sprite: 2701)
    Ghost,     // Slow, strong (sprite: 2702)
}

impl EnemyType {
    pub fn get_stats(&self, floor: i64) -> (i64, i64) {
        // Returns (health, strength) scaled by floor
        let floor_multiplier = 1.0 + (floor as f32 * 0.15);  // 15% increase per floor
        let base_stats = match self {
            EnemyType::Skeleton => (3, 1),   // Fast but fragile
            EnemyType::Orc => (7, 2),        // Balanced
            EnemyType::Ghost => (10, 3),     // Strong and tanky
        };
        (
            ((base_stats.0 as f32) * floor_multiplier) as i64,
            ((base_stats.1 as f32) * floor_multiplier) as i64,
        )
    }

    pub fn sprite_index(&self) -> usize {
        match self {
            EnemyType::Skeleton => 2700,
            EnemyType::Orc => 2701,
            EnemyType::Ghost => 2702,
        }
    }

    pub fn random() -> Self {
        let r: f32 = rand::random();
        if r < 0.4 {
            EnemyType::Skeleton
        } else if r < 0.7 {
            EnemyType::Orc
        } else {
            EnemyType::Ghost
        }
    }
}

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

#[derive(Component)]
pub struct TargetIndicator;

#[derive(Component)]
pub struct TargetedEnemy;

#[derive(Component)]
pub struct Particle {
    pub lifetime: f32,
    pub velocity: Vec2,
}

#[derive(Component, Clone, Copy)]
pub enum ParticleType {
    HitSpark,
    Death,
    HealthPickup,
}
