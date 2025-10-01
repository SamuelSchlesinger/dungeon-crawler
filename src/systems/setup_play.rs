use bevy::prelude::*;

use crate::components::*;
use crate::map;
use crate::resources::*;

const INITIAL_SCALE_FACTOR: f32 = 50.;

pub fn initialize_resources(
    mut commands: &mut Commands,
    map: &map::Map,
    initial_position: Position,
    tiles_texture_handle: &(Handle<Image>, Handle<TextureAtlasLayout>),
    existing_statistics: Option<Statistics>,
) {
    commands.insert_resource(ScaleFactor(INITIAL_SCALE_FACTOR));
    commands.insert_resource(MousePosition(Vec2::new(0., 0.)));
    commands.insert_resource(ClearColor(Color::srgb(0., 0., 0.)));
    commands.insert_resource(Follow(false));
    commands.insert_resource(Floor(initial_position.z));
    commands.insert_resource(Tiles::new());
    commands.insert_resource(Enemies::new());
    commands.insert_resource(Healths::new());
    commands.insert_resource(map.clone());
    create_camera(&mut commands, initial_position);
    commands.insert_resource(SpriteTexture(tiles_texture_handle.clone()));

    // Initialize or restore statistics
    if let Some(mut stats) = existing_statistics {
        stats.floors_completed += 1;
        commands.insert_resource(stats);
    } else {
        commands.insert_resource(Statistics::new());
    }
}

fn create_camera(commands: &mut Commands, initial_position: Position) {
    commands.spawn((
        Camera2d,
        Transform::from_translation(
            Vec3::new(
                initial_position.x as f32 * INITIAL_SCALE_FACTOR,
                initial_position.y as f32 * INITIAL_SCALE_FACTOR,
                0.,
            ),
        ),
        CameraMarker,
    ));
}

pub fn get_tiles_texture_handle(
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
) -> (Handle<Image>, Handle<TextureAtlasLayout>) {
    let tiles_texture_handle = asset_server.load("tiles.png");
    let tiles_texture_atlas =
        TextureAtlasLayout::from_grid(UVec2::new(32, 32), 64, 48, None, None);
    let atlas_layout_handle = texture_atlases.add(tiles_texture_atlas);
    (tiles_texture_handle, atlas_layout_handle)
}

pub fn setup_play(
    mut commands: Commands,
    test_map: Res<map::Map>,
    asset_server: Res<AssetServer>,
    scale_factor: Res<ScaleFactor>,
    mut floor: ResMut<Floor>,
    mut camera: Query<&mut Transform, With<CameraMarker>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut tiles: ResMut<Tiles>,
    mut enemies: ResMut<Enemies>,
    mut healths: ResMut<Healths>,
    statistics: Option<Res<Statistics>>,
) {
    let initial_position = test_map.room.initial_position;

    let (tiles_texture_image, tiles_texture_layout) = get_tiles_texture_handle(&asset_server, &mut texture_atlases);

    let room = test_map.room.clone();

    if let Some(mut transform) = camera.iter_mut().next() {
        transform.translation = Vec3::new(
            room.initial_position.x as f32 * scale_factor.0,
            room.initial_position.y as f32 * scale_factor.0,
            transform.translation.z,
        )
    } else {
        panic!("no camera");
    }

    floor.0 = room.initial_position.z;

    for (Position { x, y, z }, tile) in (&room.tiles).into_iter() {
        let entity = commands
            .spawn((
                Sprite::from_atlas_image(
                    tiles_texture_image.clone(),
                    TextureAtlas {
                        layout: tiles_texture_layout.clone(),
                        index: tile.sprite_index as usize,
                    },
                ),
                Transform::from_xyz(
                    (*x as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    (*y as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    0.,
                ),
                if *z == initial_position.z {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
                Position {
                    x: *x,
                    y: *y,
                    z: *z,
                },
                Passable(tile.passable),
                Tile,
                SpriteIndex(tile.sprite_index as usize),
                ZLevel(0.),
            ))
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
        // Randomize enemy type for variety, scale stats by floor
        let enemy_type = EnemyType::random();
        let (health, strength) = enemy_type.get_stats(floor.0.abs());
        let sprite_idx = enemy_type.sprite_index();

        let mut enemy_entity = commands.spawn((
            Sprite::from_atlas_image(
                tiles_texture_image.clone(),
                TextureAtlas {
                    layout: tiles_texture_layout.clone(),
                    index: sprite_idx,
                },
            ),
            Transform::from_xyz(
                (*x as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                (*y as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                0.01,
            ),
            if *z == initial_position.z {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Position {
                x: *x,
                y: *y,
                z: *z,
            },
            Passable(false),
            WakeZone(enemy.wake_zone.clone()),
            Awake(false),
            Health(health),
            OriginalHealth(health),
            Strength(strength),
            Enemy,
        ));

        // Add remaining components
        enemy_entity.insert((
            enemy_type,
            AIBehavior::for_enemy_type(enemy_type),
            MovementPath {
                age: 20,
                path: None,
            },
            SpriteIndex(sprite_idx),
            ZLevel(0.01),
        ));

        let enemy_id = enemy_entity.id();

        commands
            .spawn((
                Sprite {
                    color: Color::srgb(0., 1., 0.),
                    custom_size: Some(Vec2::new(
                        INITIAL_SCALE_FACTOR as f32 / 2.,
                        INITIAL_SCALE_FACTOR as f32 / 8.,
                    )),
                    ..default()
                },
                Transform::from_xyz(
                    (*x as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    (*y as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    0.05,
                ),
                HealthBar(enemy_id),
            ));
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
            .spawn((
                Sprite::from_atlas_image(
                    tiles_texture_image.clone(),
                    TextureAtlas {
                        layout: tiles_texture_layout.clone(),
                        index: health.sprite_index as usize,
                    },
                ),
                Transform::from_xyz(
                    (*x as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    (*y as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                    0.01,
                ),
                if *z == initial_position.z {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
                Position {
                    x: *x,
                    y: *y,
                    z: *z,
                },
                Passable(true),
                Health(health.health as i64),
                HealthGain,
                MovementPath {
                    age: 20,
                    path: None,
                },
                SpriteIndex(health.sprite_index as usize),
                ZLevel(0.005),
            ))
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
        .spawn((
            Sprite::from_atlas_image(
                tiles_texture_image.clone(),
                TextureAtlas {
                    layout: tiles_texture_layout.clone(),
                    index: test_map.player_sprite as usize,
                },
            ),
            Transform::from_xyz(
                (room.initial_position.x as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                (room.initial_position.y as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                0.02,
            ),
            Visibility::Visible,
            room.initial_position.clone(),
            Player,
            Health(test_map.player_health as i64),
            OriginalHealth(test_map.player_health as i64),
            Strength(test_map.player_strength as i64),
            Passable(false),
            SpriteIndex(test_map.player_sprite as usize),
            ZLevel(0.02),
        ))
        .id();

    commands
        .spawn((
            Sprite {
                color: Color::srgb(0., 1., 0.),
                custom_size: Some(Vec2::new(
                    INITIAL_SCALE_FACTOR as f32 / 2.,
                    INITIAL_SCALE_FACTOR as f32 / 8.,
                )),
                ..default()
            },
            Transform::from_xyz(
                (initial_position.x as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                (initial_position.y as f32 - 0.5) * INITIAL_SCALE_FACTOR,
                0.05,
            ),
            HealthBar(player_id),
        ));

    // Initialize or update statistics
    if let Some(stats) = statistics {
        let mut new_stats = stats.clone();
        new_stats.floors_completed += 1;
        commands.insert_resource(new_stats);
    } else {
        commands.insert_resource(Statistics::new());
    }
}
