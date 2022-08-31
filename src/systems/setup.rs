use bevy::prelude::*;

use crate::components::*;
use crate::maps;
use crate::resources::*;

const INITIAL_SCALE_FACTOR: f32 = 50.;

fn initialize_resources(commands: &mut Commands) {
    commands.insert_resource(ScaleFactor(INITIAL_SCALE_FACTOR));
    commands.insert_resource(MousePosition(Vec2::new(0., 0.)));
    commands.insert_resource(ClearColor(Color::rgb(0., 0., 0.)));
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
    let test_map = maps::avoidance();

    commands.insert_resource(Follow(false));

    let initial_position = test_map.room.initial_position;

    create_camera(&mut commands, initial_position);

    initialize_resources(&mut commands);

    commands.insert_resource(Floor(initial_position.z));

    commands.insert_resource(test_map.victory_condition);

    let tiles_texture_handle = get_tiles_texture_handle(&asset_server, &mut texture_atlases);

    commands.insert_resource(SpriteTexture(tiles_texture_handle.clone()));

    let room = test_map.room.clone();
    let mut tiles = Tiles::new();

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

    commands.insert_resource(tiles);

    let mut enemies = Enemies::new();

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
            .insert(Health(enemy.health as i32))
            .insert(Strength(enemy.strength as i32))
            .insert(Enemy)
            .insert(MovementPath {
                age: 20,
                path: None,
            })
            .insert(SpriteIndex(enemy.sprite_index as usize))
            .insert(ZLevel(0.01))
            .id();

        commands
            .spawn_bundle(
                TextBundle::from_section(
                    format!("health: {}", enemy.health),
                    TextStyle {
                        font: asset_server.load("fonts/FreeMono.ttf"),
                        font_size: 22.0,
                        color: Color::GREEN,
                    },
                )
                .with_text_alignment(TextAlignment::BOTTOM_RIGHT)
                .with_style(Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        bottom: Val::Px(*x as f32 - 10.),
                        left: Val::Px(*y as f32 + INITIAL_SCALE_FACTOR),
                        ..default()
                    },
                    ..default()
                }),
            )
            .insert(TextOverEntity(enemy_id));
        enemies.insert(
            Position {
                x: *x,
                y: *y,
                z: *z,
            },
            enemy_id,
        );
    }

    commands.insert_resource(enemies);

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
        .insert(Health(test_map.player_health as i32))
        .insert(Strength(test_map.player_strength as i32))
        .insert(Passable(false))
        .insert(SpriteIndex(test_map.player_sprite as usize))
        .insert(ZLevel(0.02))
        .id();

    commands
        .spawn_bundle(
            TextBundle::from_section(
                format!("health: {}", test_map.player_health),
                TextStyle {
                    font: asset_server.load("fonts/FreeMono.ttf"),
                    font_size: 22.0,
                    color: Color::GREEN,
                },
            )
            .with_text_alignment(TextAlignment::BOTTOM_RIGHT)
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(10.),
                    left: Val::Px(10.),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(TextOverEntity(player_id));
}
