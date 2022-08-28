use std::collections::BTreeMap;

use bevy::prelude::*;

#[derive(Debug)]
pub struct Follow(pub bool);

#[derive(Debug)]
pub struct Floor(pub i32);

#[derive(Debug)]
pub struct ScaleFactor(pub f32);

#[derive(Debug)]
pub struct MousePosition(pub Vec2);

#[derive(Debug)]
pub struct Tiles(pub BTreeMap<(i32, i32, i32), CachedTile>);

#[derive(Debug, Copy, Clone)]
pub struct CachedTile {
    pub entity: Entity,
    pub passable: bool,
}

impl Tiles {
    pub fn new() -> Self {
        Tiles(BTreeMap::new())
    }

    pub fn insert(&mut self, key: (i32, i32, i32), cached_tile: CachedTile) {
        self.0.insert(key, cached_tile);
    }

    pub fn get(&self, key: &(i32, i32, i32)) -> Option<CachedTile> {
        self.0.get(key).copied()
    }
}

#[derive(Debug)]
pub struct Enemies(pub BTreeMap<(i32, i32, i32), Entity>);

#[derive(Debug)]
pub struct SpriteTexture(pub Handle<TextureAtlas>);

impl Enemies {
    pub fn new() -> Self {
        Enemies(BTreeMap::new())
    }

    pub fn insert(&mut self, key: (i32, i32, i32), entity: Entity) {
        self.0.insert(key, entity);
    }
}
