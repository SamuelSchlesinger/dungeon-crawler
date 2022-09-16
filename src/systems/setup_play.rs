use bevy::prelude::*;

use crate::components::*;
use crate::map;
use crate::resources::*;

const INITIAL_SCALE_FACTOR: f32 = 50.;

pub fn initialize_resources(
    mut commands: &mut Commands,
    map: &map::Map,
    initial_position: Position,
    tiles_texture_handle: &Handle<TextureAtlas>,
) {
    commands.insert_resource(ScaleFactor(INITIAL_SCALE_FACTOR));
    commands.insert_resource(MousePosition(Vec2::new(0., 0.)));
    commands.insert_resource(ClearColor(Color::rgb(0., 0., 0.)));
    commands.insert_resource(Follow(false));
    commands.insert_resource(Floor(initial_position.z));
    commands.insert_resource(Tiles::new());
    commands.insert_resource(Enemies::new());
    commands.insert_resource(Healths::new());
    commands.insert_resource(map.clone());
    create_camera(&mut commands, initial_position);
    commands.insert_resource(SpriteTexture(tiles_texture_handle.clone()));
}

fn create_camera(commands: &mut Commands, initial_position: Position) {
    let mut camera_2d_bundle = Camera2dBundle::default();
    camera_2d_bundle.transform = camera_2d_bundle.transform.with_translation(
        camera_2d_bundle.transform.translation
            + Vec3::new(
                initial_position.x as f32 * INITIAL_SCALE_FACTOR,
                initial_position.z as f32 * INITIAL_SCALE_FACTOR,
                0.,
            ),
    );
    commands.spawn_bundle(camera_2d_bundle).insert(Camera);
}

pub fn get_tiles_texture_handle(
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) -> Handle<TextureAtlas> {
    let tiles_texture_handle = asset_server.load("tiles.png");
    let tiles_texture_atlas =
        TextureAtlas::from_grid(tiles_texture_handle, Vec2::new(32., 32.), 64, 48);
    texture_atlases.add(tiles_texture_atlas)
}

pub fn setup_play(
    mut commands: Commands,
    test_map: Res<map::Map>,
    asset_server: Res<AssetServer>,
    scale_factor: Res<ScaleFactor>,
    mut floor: ResMut<Floor>,
    mut camera: Query<&mut Transform, With<Camera>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut tiles: ResMut<Tiles>,
    mut enemies: ResMut<Enemies>,
    mut healths: ResMut<Healths>,
) {
    let initial_position = test_map.room.initial_position;

    let tiles_texture_handle = get_tiles_texture_handle(&asset_server, &mut texture_atlases);

    let room = test_map.room.clone();

    if let Some(mut transform) = camera.iter_mut().next() {
        transform.translation = Vec3::new(
            room.initial_position.x as f32 * scale_factor.0,
            room.initial_position.x as f32 * scale_factor.0,
            transform.translation.z,
        )
    } else {
        panic!("no camera");
    }

    floor.0 = room.initial_position.z;

    for (Position { x, y, z }, tile) in (&room.tiles).into_iter() {
        let entity = commands
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
                    is_visible: *z == initial_position.z,
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
        tiles.insert(
            Position {
                x: *x,
                y: *y,
                z: *z,
            },
            CachedTile {
                entity,
                passable: tile.passable,
            },
        );
    }

    for (Position { x, y, z }, enemy) in (&room.enemies).into_iter() {
        let enemy_id = commands
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
                    is_visible: *z == initial_position.z,
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
            .insert(Health(enemy.health as i64))
            .insert(OriginalHealth(enemy.health as i64))
            .insert(Strength(enemy.strength as i64))
            .insert(Enemy)
            .insert(MovementPath {
                age: 20,
                path: None,
            })
            .insert(SpriteIndex(enemy.sprite_index as usize))
            .insert(ZLevel(0.01))
            .id();

        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0., 1., 0.),
                    custom_size: Some(Vec2::new(
                        INITIAL_SCALE_FACTOR as f32 / 2.,
                        INITIAL_SCALE_FACTOR as f32 / 8.,
                    )),
                    ..default()
                },
                transform: Transform::from_xyz(
                    (*x as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    (*y as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    0.05,
                ),
                ..default()
            })
            .insert(HealthBar(enemy_id));
        enemies.insert(
            Position {
                x: *x,
                y: *y,
                z: *z,
            },
            enemy_id,
        );
    }

    for (Position { x, y, z }, health) in (&room.healths).into_iter() {
        let health_id = commands
            .spawn_bundle(SpriteSheetBundle {
                transform: Transform::from_xyz(
                    (*x as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    (*y as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    0.01,
                ),
                texture_atlas: tiles_texture_handle.clone(),
                sprite: TextureAtlasSprite {
                    index: health.sprite_index as usize,
                    custom_size: Some(Vec2::new(INITIAL_SCALE_FACTOR, INITIAL_SCALE_FACTOR)),
                    ..default()
                },
                visibility: Visibility {
                    is_visible: *z == initial_position.z,
                },
                ..default()
            })
            .insert(Position {
                x: *x,
                y: *y,
                z: *z,
            })
            .insert(Passable(true))
            .insert(Health(health.health as i64))
            .insert(HealthGain)
            .insert(MovementPath {
                age: 20,
                path: None,
            })
            .insert(SpriteIndex(health.sprite_index as usize))
            .insert(ZLevel(0.005))
            .id();
        healths.insert(
            Position {
                x: *x,
                y: *y,
                z: *z,
            },
            CachedHealth {
                entity: health_id,
                health: health.health as i64,
            },
        );
    }

    let player_id = commands
        .spawn_bundle(SpriteSheetBundle {
            transform: Transform::from_xyz(
                (room.initial_position.z as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                (room.initial_position.z as f32 - 0.5) * INITIAL_SCALE_FACTOR,
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
        .insert(room.initial_position.clone())
        .insert(Player)
        .insert(Health(test_map.player_health as i64))
        .insert(OriginalHealth(test_map.player_health as i64))
        .insert(Strength(test_map.player_strength as i64))
        .insert(Passable(false))
        .insert(SpriteIndex(test_map.player_sprite as usize))
        .insert(ZLevel(0.02))
        .id();

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0., 1., 0.),
                custom_size: Some(Vec2::new(
                    INITIAL_SCALE_FACTOR as f32 / 2.,
                    INITIAL_SCALE_FACTOR as f32 / 8.,
                )),
                ..default()
            },
            transform: Transform::from_xyz(
                (initial_position.x as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                (initial_position.y as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                0.05,
            ),
            ..default()
        })
        .insert(HealthBar(player_id));
}
