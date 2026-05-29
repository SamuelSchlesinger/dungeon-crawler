use bevy::prelude::*;

use crate::components::*;
use crate::map;
use crate::resources::*;
use crate::tuning;
use crate::utils::grid_to_world_center;

const INITIAL_SCALE_FACTOR: f32 = 50.;

/// A `Once` timer of the given duration that begins already finished, so a
/// cooldown gated on `timer.finished()` is ready to fire immediately.
fn finished_timer(secs: f32) -> Timer {
    let mut t = Timer::from_seconds(secs, TimerMode::Once);
    t.tick(std::time::Duration::from_secs_f32(secs));
    t
}

/// Builds an atlas sprite sized to one tile. Real-time actors (player/enemies)
/// are excluded from `animate_sprites` (which used to size sprites), so they set
/// `custom_size` at spawn here instead.
fn actor_sprite(
    image: &Handle<Image>,
    layout: &Handle<TextureAtlasLayout>,
    index: usize,
) -> Sprite {
    let mut sprite = Sprite::from_atlas_image(
        image.clone(),
        TextureAtlas {
            layout: layout.clone(),
            index,
        },
    );
    sprite.custom_size = Some(Vec2::splat(INITIAL_SCALE_FACTOR));
    sprite
}

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
    // Camera follows the player by default in the action build (F toggles off).
    commands.insert_resource(Follow(true));
    commands.insert_resource(Floor(initial_position.z));
    commands.insert_resource(Tiles::new());
    commands.insert_resource(Enemies::new());
    commands.insert_resource(Healths::new());
    commands.insert_resource(WeaponDrops::new());
    commands.insert_resource(Revealed::new());
    commands.insert_resource(VisibleTiles::new());
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

#[allow(clippy::too_many_arguments)]
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
    mut weapon_drops: ResMut<WeaponDrops>,
    mut revealed: ResMut<Revealed>,
    mut visible_tiles: ResMut<VisibleTiles>,
    statistics: Option<Res<Statistics>>,
    carry_over: Option<Res<CarryOver>>,
) {
    // Reset position-indexed resources so a freshly entered map never inherits
    // stale entries from a previous floor or run.
    *tiles = Tiles::new();
    *enemies = Enemies::new();
    *healths = Healths::new();
    *weapon_drops = WeaponDrops::new();
    *revealed = Revealed::new();
    *visible_tiles = VisibleTiles::new();

    let initial_position = test_map.room.initial_position;

    // Run depth used for enemy scaling. For procedural floors every tile is on
    // z=0, so we scale by the number of floors cleared this run rather than the
    // z-plane (which `Floor` tracks for visibility).
    let run_depth = statistics.as_ref().map_or(0, |s| s.floors_completed);

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
        // Randomize enemy type for variety, scale stats by run depth
        let enemy_type = EnemyType::random();
        let (health, strength) = enemy_type.get_stats(run_depth);
        let sprite_idx = enemy_type.sprite_index();

        let mut enemy_entity = commands.spawn((
            actor_sprite(&tiles_texture_image, &tiles_texture_layout, sprite_idx),
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

        // Add remaining components, including the real-time action components
        // (continuous position, knockback, and the attack state machine).
        enemy_entity.insert((
            enemy_type,
            AIBehavior::for_enemy_type(enemy_type),
            MovementPath { path: None },
            SpriteIndex(sprite_idx),
            ZLevel(0.01),
            WorldPos(grid_to_world_center(*x, *y, INITIAL_SCALE_FACTOR)),
            Facing::default(),
            Knockback::default(),
            EnemyAttack {
                telegraph: Timer::from_seconds(tuning::ENEMY_TELEGRAPH, TimerMode::Once),
                cooldown: Timer::from_seconds(tuning::ENEMY_ATTACK_COOLDOWN, TimerMode::Once),
                winding_up: false,
            },
            // Stagger initial re-path so a whole pack doesn't path in lockstep.
            RepathTimer(Timer::from_seconds(
                tuning::ENEMY_REPATH_INTERVAL * rand::random::<f32>(),
                TimerMode::Once,
            )),
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
                MovementPath { path: None },
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

    // Carry the player's current health/strength forward between floors when a
    // run is in progress; otherwise use the map's starting values.
    let (player_health, player_strength) = match carry_over.as_ref() {
        Some(c) => (c.health, c.strength),
        None => (test_map.player_health as i64, test_map.player_strength as i64),
    };

    let mut player_entity = commands.spawn((
        actor_sprite(
            &tiles_texture_image,
            &tiles_texture_layout,
            test_map.player_sprite as usize,
        ),
        Transform::from_xyz(
            (room.initial_position.x as f32 - 0.5) * INITIAL_SCALE_FACTOR,
            (room.initial_position.y as f32 - 0.5) * INITIAL_SCALE_FACTOR,
            0.02,
        ),
        Visibility::Visible,
        room.initial_position.clone(),
        Player,
        Health(player_health),
        OriginalHealth(test_map.player_health as i64),
        Strength(player_strength),
        Passable(false),
        SpriteIndex(test_map.player_sprite as usize),
        ZLevel(0.02),
    ));
    // Real-time action components for the player (split into a second insert to
    // stay under Bevy's per-tuple component limit).
    player_entity.insert((
        WorldPos(grid_to_world_center(
            room.initial_position.x,
            room.initial_position.y,
            INITIAL_SCALE_FACTOR,
        )),
        Facing::default(),
        Knockback::default(),
        AttackCooldown(finished_timer(tuning::ATTACK_COOLDOWN)),
        Dash {
            // Start with the burst/i-frame timers already finished so the player
            // is NOT spuriously invulnerable on spawn, and may dash immediately.
            active: finished_timer(tuning::DASH_DURATION),
            iframes: finished_timer(tuning::DASH_IFRAMES),
            cooldown: finished_timer(tuning::DASH_COOLDOWN),
            dir: Vec2::ZERO,
            dashing: false,
        },
    ));
    let player_id = player_entity.id();

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

    // Ensure a Statistics resource exists; do NOT increment floors_completed
    // here. Floor-clear counting is owned by the NextFloor transition so it
    // stays consistent across multi-floor runs.
    if statistics.is_none() {
        commands.insert_resource(Statistics::new());
    }

    // CarryOver is single-use per floor transition; consume it so a fresh run
    // started from the menu does not inherit stale stats.
    if carry_over.is_some() {
        commands.remove_resource::<CarryOver>();
    }
}
