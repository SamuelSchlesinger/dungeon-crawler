use std::collections::BTreeSet;

use bevy::prelude::*;

use itertools::Itertools;

use crate::components::*;
use crate::map;
use crate::resources::*;

const INITIAL_SCALE_FACTOR: f32 = 50.;

fn make_test_map() -> map::Map {
    let border_tile = map::Tile {
        sprite_index: 15 * 64 - 13,
        passable: false,
    };
    map::Map {
        player_sprite: 71,
        rooms: vec![map::Room {
            initial_position: (1, 1, 0),
            tiles: (-10..10)
                .cartesian_product(-10..10)
                .cartesian_product(-10..10)
                .map(|((i, j), k)| (i, j, k))
                .map(|(i, j, k)| {
                    (
                        (i, j, k),
                        map::Tile {
                            sprite_index: 960,
                            passable: true,
                        },
                    )
                })
                .chain(
                    (-10..=10)
                        .cartesian_product(-10..=10)
                        .map(|(i, k)| ((i, 10, k), border_tile.clone())),
                )
                .chain(
                    (-10..=10)
                        .cartesian_product(-10..=10)
                        .map(|(i, k)| ((i, -10, k), border_tile.clone())),
                )
                .chain(
                    (-10..=10)
                        .cartesian_product(-10..=10)
                        .map(|(i, k)| ((-10, -i, k), border_tile.clone())),
                )
                .chain(
                    (-10..=10)
                        .cartesian_product(-10..=10)
                        .map(|(i, k)| ((10, i, k), border_tile.clone())),
                )
                .collect(),
            enemies: vec![(
                (5, 5, 0),
                map::Enemy {
                    sprite_index: 74,
                    health: 10,
                    strength: 5,
                    wake_zone: (-3..3)
                        .cartesian_product(-3..3)
                        .map(|(i, j)| (5 + i, 5 + j, 0))
                        .collect::<BTreeSet<_>>(),
                },
            )]
            .into_iter()
            .collect(),
        }],
        initial_room: 0,
        player_health: 100,
        player_strength: 2,
    }
}

fn initialize_resources(commands: &mut Commands) {
    commands.insert_resource(ScaleFactor(INITIAL_SCALE_FACTOR));
    commands.insert_resource(MousePosition(Vec2::new(0., 0.)));
}

fn create_camera(commands: &mut Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(Camera);
}

fn get_tiles_texture_handle(
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) -> Handle<TextureAtlas> {
    let tiles_texture_handle = asset_server.load("tiles.png");
    let tiles_texture_atlas =
        TextureAtlas::from_grid(tiles_texture_handle, Vec2::new(32., 32.), 64, 48);
    texture_atlases.add(tiles_texture_atlas)
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let test_map = make_test_map();

    create_camera(&mut commands);

    initialize_resources(&mut commands);

    let initial_position = test_map
        .rooms
        .get(test_map.initial_room as usize)
        .unwrap()
        .initial_position;

    commands.insert_resource(Floor(initial_position.2));

    let tiles_texture_handle = get_tiles_texture_handle(&asset_server, &mut texture_atlases);

    commands.insert_resource(SpriteTexture(tiles_texture_handle.clone()));

    let room = test_map.rooms[test_map.initial_room as usize].clone();
    let mut tiles = Tiles::new();

    for ((x, y, z), tile) in (&room.tiles).into_iter() {
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
                visibility: Visibility {
                    is_visible: *z == initial_position.2,
                },
                ..default()
            })
            .insert(Position {
                x: *x,
                y: *y,
                z: *z,
            })
            .insert(Passable(tile.passable))
            .insert(Tile)
            .insert(SpriteIndex(tile.sprite_index as usize))
            .insert(ZLevel(0.))
            .id();
        tiles.insert((*x, *y, *z), id);
    }

    commands.insert_resource(tiles);

    let mut enemies = Enemies::new();

    for ((x, y, z), enemy) in (&room.enemies).into_iter() {
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
                visibility: Visibility {
                    is_visible: *z == initial_position.2,
                },
                ..default()
            })
            .insert(Position {
                x: *x,
                y: *y,
                z: *z,
            })
            .insert(Passable(false))
            .insert(WakeZone(enemy.wake_zone.clone()))
            .insert(Awake(false))
            .insert(Enemy)
            .insert(SpriteIndex(enemy.sprite_index as usize))
            .insert(ZLevel(0.01))
            .id();
        enemies.insert((*x, *y, *z), id);
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
            visibility: Visibility { is_visible: true },
            ..default()
        })
        .insert(Position {
            x: room.initial_position.0,
            y: room.initial_position.1,
            z: room.initial_position.2,
        })
        .insert(Player)
        .insert(Passable(false))
        .insert(SpriteIndex(test_map.player_sprite as usize))
        .insert(ZLevel(0.02));
}
