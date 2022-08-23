mod combat;
mod components;
mod map;
mod resources;

use std::collections::BTreeSet;

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
        .add_system(change_sprite_for_awake_enemies)
        .add_system(track_mouse_movement)
        .add_system(mouse_button_handler)
        .run();
}

fn track_mouse_movement(
    mut cursor_moved_event_reader: EventReader<CursorMoved>,
    mut mouse_position: ResMut<MousePosition>,
) {
    if let Some(cursor_moved) = cursor_moved_event_reader.iter().last() {
        *mouse_position = MousePosition(cursor_moved.position);
    }
}

fn convert_mouse_position_to_world_coordinates(
    window: &Window,
    transform: &Transform,
    scaling_factor: f32,
    mouse_position: Vec2,
) -> Vec2 {
    Vec2::new(
        transform.translation.x / scaling_factor
            + (mouse_position.x - window.width() / 2.) / scaling_factor,
        transform.translation.y / scaling_factor
            + (mouse_position.y - window.height() / 2.) / scaling_factor,
    )
}

fn mouse_button_handler(
    windows: Res<Windows>,
    mouse_button: Res<Input<MouseButton>>,
    mouse_position: Res<MousePosition>,
    scale_factor: Res<ScaleFactor>,
    camera: Query<&Transform, With<Camera>>,
    tiles: Res<Tiles>,
    mut commands: Commands,
) {
    if let Some(transform) = camera.iter().next() {
        if let Some(window) = windows.get_primary() {
            if mouse_button.just_pressed(MouseButton::Left) {
                let coordinates = convert_mouse_position_to_world_coordinates(
                    window,
                    transform,
                    scale_factor.0,
                    mouse_position.0,
                );
                let position = Position::from(coordinates);
                if let Some(tile_entity) = tiles.get(&(position.x, position.y)) {
                    let mut ent = commands.entity(tile_entity);
                    ent.remove::<SpriteIndex>();
                    ent.insert(SpriteIndex(64 * 10 + 4));
                }
                println!("{:?} at {:?}", position, coordinates);
            } else if mouse_button.just_pressed(MouseButton::Right) {
            } else if mouse_button.just_pressed(MouseButton::Middle) {
            } else {
            }
        }
    }
}

fn change_sprite_for_awake_enemies(mut query: Query<(&mut SpriteIndex, &Awake), With<Enemy>>) {
    for (mut sprite_index, awake) in query.iter_mut() {
        println!("{:?}, {:?}", sprite_index, awake);
        if awake.0 {
            sprite_index.0 = 141;
        } else {
            sprite_index.0 = 74;
        }
    }
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
            (pos.x as f32 - 0.5) * scale_factor.0,
            (pos.y as f32 - 0.5) * scale_factor.0,
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

fn move_player(
    mut query: Query<(Entity, &mut Position), With<Player>>,
    mut enemies: Query<(&WakeZone, &mut Awake), With<Enemy>>,
    entities: Query<(Entity, &Position, &Passable), Without<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if let Some((entity, mut position)) = query.iter_mut().next() {
        let old_position = position.clone();
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

        for (other_entity, other_position, passable) in entities.iter() {
            if other_entity != entity {
                if *other_position == *position && !passable.0 {
                    *position = old_position;
                    return;
                }
            }
        }

        for (wake_zone, mut wake) in enemies.iter_mut() {
            if wake_zone.0.contains(&(position.x, position.y)) {
                wake.0 = true;
            }
        }
    }
}

fn setup(
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

    commands.insert_resource(ScaleFactor(SCALE_FACTOR));
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
                    (*x as f32 - 0.5) * SCALE_FACTOR,
                    (*y as f32 - 0.5) * SCALE_FACTOR,
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
                    (*x as f32 - 0.5) * SCALE_FACTOR,
                    (*y as f32 - 0.5) * SCALE_FACTOR,
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
                (room.initial_position.0 as f32 - 0.5) * SCALE_FACTOR,
                (room.initial_position.1 as f32 - 0.5) * SCALE_FACTOR,
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
        .insert(Passable(false))
        .insert(SpriteIndex(test_map.player_sprite as usize))
        .insert(ZLevel(0.02));
}
