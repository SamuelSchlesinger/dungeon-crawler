mod combat;
mod components;
mod map;
mod resources;

use bevy::{prelude::*, time::FixedTimestep};
use components::*;
use itertools::Itertools;
use resources::*;

const SCALE_FACTOR: f32 = 50.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::steps_per_second(60.))
                .with_system(animate_sprites),
        )
        .add_system(move_camera)
        .add_system(move_player)
        .run();
}

fn animate_sprites(
    mut query: Query<(
        &mut Transform,
        &mut TextureAtlasSprite,
        &Position,
        &ZLevel,
        &SpriteIndex,
    )>,
    scale_factor: Res<ScaleFactor>,
) {
    for (mut transform, mut sprite, pos, zlevel, sprite_index) in query.iter_mut() {
        *transform = Transform::from_xyz(
            pos.x as f32 * scale_factor.0,
            pos.y as f32 * scale_factor.0,
            zlevel.0,
        );
        sprite.index = sprite_index.0;
        sprite.custom_size = Some(Vec2::new(scale_factor.0, scale_factor.0));
    }
}

fn move_camera(
    mut query: Query<&mut Transform, With<Camera>>,
    mut scale_factor: ResMut<ScaleFactor>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if let Some(mut transform) = query.iter_mut().next() {
        let translation = transform.translation;
        if keyboard_input.just_pressed(KeyCode::Left) {
            *transform =
                transform.with_translation(translation + Vec3::new(-scale_factor.0, 0., 0.));
        }
        if keyboard_input.just_pressed(KeyCode::Right) {
            *transform =
                transform.with_translation(translation + Vec3::new(scale_factor.0, 0., 0.));
        }
        if keyboard_input.just_pressed(KeyCode::Up) {
            *transform =
                transform.with_translation(translation + Vec3::new(0., scale_factor.0, 0.));
        }
        if keyboard_input.just_pressed(KeyCode::Down) {
            *transform =
                transform.with_translation(translation + Vec3::new(0., -scale_factor.0, 0.));
        }
    }
    if keyboard_input.just_pressed(KeyCode::PageUp) {
        scale_factor.0 = scale_factor.0 - 5.0;
    } else if keyboard_input.just_pressed(KeyCode::PageDown) {
        scale_factor.0 = scale_factor.0 + 5.0;
    }
}

fn move_player(mut query: Query<&mut Position, With<Player>>, keyboard_input: Res<Input<KeyCode>>) {
    if let Some(mut position) = query.iter_mut().next() {
        if keyboard_input.just_pressed(KeyCode::A) {
            position.x -= 1;
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            position.x += 1;
        }
        if keyboard_input.just_pressed(KeyCode::W) {
            position.y += 1;
        }
        if keyboard_input.just_pressed(KeyCode::S) {
            position.y -= 1;
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
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
                .collect(),
            enemies: vec![(
                (5, 5),
                map::Enemy {
                    sprite_index: 74,
                    health: 10,
                    strength: 5,
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

    commands.insert_resource(ScaleFactor(SCALE_FACTOR));

    let tiles_texture_handle = asset_server.load("tiles.png");
    let tiles_texture_atlas =
        TextureAtlas::from_grid(tiles_texture_handle, Vec2::new(32., 32.), 64, 48);
    let tiles_texture_handle = texture_atlases.add(tiles_texture_atlas);

    let room = test_map.rooms[test_map.initial_room as usize].clone();

    for ((x, y), tile) in (&room.tiles).into_iter() {
        commands
            .spawn_bundle(SpriteSheetBundle {
                transform: Transform::from_xyz(
                    *x as f32 * SCALE_FACTOR,
                    *y as f32 * SCALE_FACTOR,
                    0.,
                ),
                texture_atlas: tiles_texture_handle.clone(),
                sprite: TextureAtlasSprite {
                    index: tile.sprite_index as usize,
                    custom_size: Some(Vec2::new(SCALE_FACTOR, SCALE_FACTOR)),
                    ..default()
                },
                ..default()
            })
            .insert(Position { x: *x, y: *y })
            .insert(Tile)
            .insert(SpriteIndex(tile.sprite_index as usize))
            .insert(ZLevel(0.));
    }

    for ((x, y), enemy) in (&room.enemies).into_iter() {
        commands
            .spawn_bundle(SpriteSheetBundle {
                transform: Transform::from_xyz(
                    *x as f32 * SCALE_FACTOR,
                    *y as f32 * SCALE_FACTOR,
                    0.01,
                ),
                texture_atlas: tiles_texture_handle.clone(),
                sprite: TextureAtlasSprite {
                    index: enemy.sprite_index as usize,
                    custom_size: Some(Vec2::new(SCALE_FACTOR, SCALE_FACTOR)),
                    ..default()
                },
                ..default()
            })
            .insert(Position { x: *x, y: *y })
            .insert(Enemy)
            .insert(SpriteIndex(enemy.sprite_index as usize))
            .insert(ZLevel(0.01));
    }
    commands
        .spawn_bundle(SpriteSheetBundle {
            transform: Transform::from_xyz(
                room.initial_position.0 as f32 * SCALE_FACTOR,
                room.initial_position.1 as f32 * SCALE_FACTOR,
                0.02,
            ),
            texture_atlas: tiles_texture_handle.clone(),
            sprite: TextureAtlasSprite {
                index: test_map.player_sprite as usize,
                custom_size: Some(Vec2::new(SCALE_FACTOR, SCALE_FACTOR)),
                ..default()
            },
            ..default()
        })
        .insert(Position {
            x: room.initial_position.0,
            y: room.initial_position.1,
        })
        .insert(Player)
        .insert(SpriteIndex(test_map.player_sprite as usize))
        .insert(ZLevel(0.02));
}
