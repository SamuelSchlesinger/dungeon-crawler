mod components;
mod events;
mod map;
mod maps;
mod resources;
mod state;
mod systems;
mod utils;

use bevy::{prelude::*, time::FixedTimestep};
use state::GameState;
use systems::*;

fn main() {
    println!("working directory: {:?}", std::env::current_dir());
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_state(GameState::Playing)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::steps_per_second(30.))
                .with_system(follow.before(animate_sprites))
                .with_system(animate_sprites)
                .with_system(display_health),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::steps_per_second(100.))
                .with_system(walk_enemies),
        )
        .add_system(move_camera)
        .add_system(move_player)
        .add_system(set_follow)
        .add_system(set_visibility)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::steps_per_second(3.))
                .with_system(combat),
        )
        .add_system(track_mouse_movement)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::steps_per_second(1.))
                .with_system(victory),
        )
        .add_system_set(SystemSet::on_update(GameState::Victory).with_system(on_victory))
        .run();
}
