use std::collections::{BTreeMap, BTreeSet};

use crate::components::Position;

use bevy::prelude::*;

#[derive(Debug, Resource)]
pub struct Follow(pub bool);

#[derive(Debug, Resource)]
pub struct Floor(pub i64);

#[derive(Debug, Resource)]
pub struct ScaleFactor(pub f32);

#[derive(Debug, Resource)]
pub struct MousePosition(pub Vec2);

#[derive(Debug, Resource)]
pub struct Tiles(pub BTreeMap<Position, CachedTile>);

#[derive(Debug, Copy, Clone)]
pub struct CachedTile {
    pub entity: Entity,
    pub passable: bool,
}

impl Tiles {
    pub fn new() -> Self {
        Tiles(BTreeMap::new())
    }

    pub fn insert(&mut self, key: Position, cached_tile: CachedTile) {
        self.0.insert(key, cached_tile);
    }

    pub fn get(&self, key: &Position) -> Option<CachedTile> {
        self.0.get(key).copied()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CachedHealth {
    pub entity: Entity,
    pub health: i64,
}

#[derive(Debug, Resource)]
pub struct Healths(pub BTreeMap<Position, CachedHealth>);

impl Healths {
    pub fn new() -> Self {
        Healths(BTreeMap::new())
    }

    pub fn insert(&mut self, position: Position, cached_health: CachedHealth) {
        self.0.insert(position, cached_health);
    }

    pub fn remove(&mut self, position: Position) -> Option<CachedHealth> {
        self.0.remove(&position)
    }
}

#[derive(Debug, Resource)]
pub struct Enemies {
    entity_positions: BTreeMap<Entity, Position>,
    position_entities: BTreeMap<Position, BTreeSet<Entity>>,
}

#[derive(Debug, Resource)]
pub struct SpriteTexture(pub (Handle<Image>, Handle<TextureAtlasLayout>));

impl Enemies {
    pub fn new() -> Self {
        Enemies {
            entity_positions: BTreeMap::new(),
            position_entities: BTreeMap::new(),
        }
    }

    pub fn enemies_at(&self, position: Position) -> Option<&BTreeSet<Entity>> {
        self.position_entities.get(&position)
    }

    pub fn occupied_position(&self, position: Position) -> bool {
        self.position_entities
            .get(&position)
            .map_or_else(|| false, |set| !set.is_empty())
    }

    pub fn insert(&mut self, position: Position, entity: Entity) {
        match self.entity_positions.entry(entity) {
            // If we don't have a mapping Entity => Position, then we have to insert one
            std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(position);
                match self.position_entities.entry(position) {
                    std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                        let mut set = BTreeSet::new();
                        set.insert(entity);
                        vacant_entry.insert(set);
                    }
                    std::collections::btree_map::Entry::Occupied(mut occupied_entry) => {
                        occupied_entry.get_mut().insert(entity);
                    }
                }
            }
            std::collections::btree_map::Entry::Occupied(mut occupied_entry) => {
                let current_position = occupied_entry.get().clone();
                occupied_entry.insert(position);
                match self.position_entities.entry(current_position) {
                    std::collections::btree_map::Entry::Vacant(_vacant_entry) => unreachable!(),
                    std::collections::btree_map::Entry::Occupied(mut occupied_entry) => {
                        occupied_entry.get_mut().remove(&entity);
                        match self.position_entities.entry(position) {
                            std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                                let mut set = BTreeSet::new();
                                set.insert(entity);
                                vacant_entry.insert(set);
                            }
                            std::collections::btree_map::Entry::Occupied(mut occupied_entry) => {
                                occupied_entry.get_mut().insert(entity);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_enemies() {
    let mut enemies = Enemies::new();
    for i in 0..30u32 {
        enemies.insert(
            Position {
                x: (i / 4) as i64,
                y: (i / 4) as i64,
                z: (i / 4) as i64,
            },
            Entity::from_raw(i),
        );
    }
    for i in 0..30u32 {
        enemies.insert(
            Position {
                x: (i / 4 + 1) as i64,
                y: (i / 4 + 1) as i64,
                z: (i / 4 + 1) as i64,
            },
            Entity::from_raw(i),
        );
    }
}
