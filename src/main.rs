mod components;
mod events;
mod map;
mod maps;
mod resources;
mod state;
mod systems;
mod utils;

use bevy::prelude::*;
use state::GameState;
use systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .insert_resource(Time::<Fixed>::from_hz(30.0))
        .add_systems(Startup, setup)
        .add_systems(Update, menu.run_if(in_state(GameState::Menu)))
        .add_systems(OnEnter(GameState::Playing), setup_play)
        .add_systems(
            FixedUpdate,
            (
                follow,
                display_health,
                animate_sprites,
                walk_enemies,
                combat,
                cleanup_dead_enemies,
                cleanup_collected_health,
                victory,
                defeat,
            ).run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                move_camera,
                move_player,
                set_follow,
                health,
                set_visibility,
                track_mouse_movement,
            ).run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::Victory), on_victory)
        .add_systems(OnEnter(GameState::Defeat), on_defeat)
        .run();
}
