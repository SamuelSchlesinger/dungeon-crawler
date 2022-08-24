use std::collections::BTreeSet;

use bevy::prelude::*;

use itertools::Itertools;

use crate::components::*;
use crate::map;
use crate::resources::*;

const INITIAL_SCALE_FACTOR: f32 = 50.;

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let border_tile = map::Tile {
        sprite_index: 15 * 64 - 13,
        passable: false,
    };
    let test_map = map::Map {
        player_sprite: 71,
        rooms: vec![map::Room {
            initial_position: (1, 1),
            tiles: (-10..10)
                .cartesian_product(-10..10)
                .map(|(i, j)| {
                    (
                        (i, j),
                        map::Tile {
                            sprite_index: 960,
                            passable: true,
                        },
                    )
                })
                .chain((-10..=10).map(|i| ((i, 10), border_tile.clone())))
                .chain((-10..=10).map(|i| ((i, -10), border_tile.clone())))
                .chain((-10..=10).map(|i| ((-10, -i), border_tile.clone())))
                .chain((-10..=10).map(|i| ((10, i), border_tile.clone())))
                .collect(),
            enemies: vec![(
                (5, 5),
                map::Enemy {
                    sprite_index: 74,
                    health: 10,
                    strength: 5,
                    wake_zone: (-3..3)
                        .cartesian_product(-3..3)
                        .map(|(i, j)| (5 + i, 5 + j))
                        .collect::<BTreeSet<_>>(),
                },
            )]
            .into_iter()
            .collect(),
        }],
        initial_room: 0,
        player_health: 100,
        player_strength: 2,
    };

    serde_json::to_writer(
        std::fs::File::options()
            .write(true)
            .create(true)
            .open("map.json")
            .unwrap(),
        &test_map,
    )
    .unwrap();

    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(Camera);

    commands.insert_resource(ScaleFactor(INITIAL_SCALE_FACTOR));
    commands.insert_resource(MousePosition(Vec2::new(0., 0.)));

    let tiles_texture_handle = asset_server.load("tiles.png");
    let tiles_texture_atlas =
        TextureAtlas::from_grid(tiles_texture_handle, Vec2::new(32., 32.), 64, 48);
    let tiles_texture_handle = texture_atlases.add(tiles_texture_atlas);

    commands.insert_resource(SpriteTexture(tiles_texture_handle.clone()));

    let room = test_map.rooms[test_map.initial_room as usize].clone();
    let mut tiles = Tiles::new();

    for ((x, y), tile) in (&room.tiles).into_iter() {
        let id = commands
            .spawn_bundle(SpriteSheetBundle {
                transform: Transform::from_xyz(
                    (*x as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    (*y as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    0.,
                ),
                texture_atlas: tiles_texture_handle.clone(),
                sprite: TextureAtlasSprite {
                    index: tile.sprite_index as usize,
                    custom_size: Some(Vec2::new(INITIAL_SCALE_FACTOR, INITIAL_SCALE_FACTOR)),
                    ..default()
                },
                ..default()
            })
            .insert(Position { x: *x, y: *y })
            .insert(Passable(tile.passable))
            .insert(Tile)
            .insert(SpriteIndex(tile.sprite_index as usize))
            .insert(ZLevel(0.))
            .id();
        tiles.insert((*x, *y), id);
    }

    commands.insert_resource(tiles);

    let mut enemies = Enemies::new();

    for ((x, y), enemy) in (&room.enemies).into_iter() {
        let id = commands
            .spawn_bundle(SpriteSheetBundle {
                transform: Transform::from_xyz(
                    (*x as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    (*y as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    0.01,
                ),
                texture_atlas: tiles_texture_handle.clone(),
                sprite: TextureAtlasSprite {
                    index: enemy.sprite_index as usize,
                    custom_size: Some(Vec2::new(INITIAL_SCALE_FACTOR, INITIAL_SCALE_FACTOR)),
                    ..default()
                },
                ..default()
            })
            .insert(Position { x: *x, y: *y })
            .insert(Passable(false))
            .insert(WakeZone(enemy.wake_zone.clone()))
            .insert(Awake(false))
            .insert(Enemy)
            .insert(SpriteIndex(enemy.sprite_index as usize))
            .insert(ZLevel(0.01))
            .id();
        enemies.insert((*x, *y), id);
    }

    commands.insert_resource(enemies);

    commands
        .spawn_bundle(SpriteSheetBundle {
            transform: Transform::from_xyz(
                (room.initial_position.0 as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                (room.initial_position.1 as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                0.02,
            ),
            texture_atlas: tiles_texture_handle.clone(),
            sprite: TextureAtlasSprite {
                index: test_map.player_sprite as usize,
                custom_size: Some(Vec2::new(INITIAL_SCALE_FACTOR, INITIAL_SCALE_FACTOR)),
                ..default()
            },
            ..default()
        })
        .insert(Position {
            x: room.initial_position.0,
            y: room.initial_position.1,
        })
        .insert(Player)
        .insert(Passable(false))
        .insert(SpriteIndex(test_map.player_sprite as usize))
        .insert(ZLevel(0.02));
}
