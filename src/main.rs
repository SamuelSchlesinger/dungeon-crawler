mod components;
mod events;
mod map;
mod maps;
mod resources;
mod systems;
mod utils;

use bevy::{prelude::*, time::FixedTimestep};
use systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::steps_per_second(60.))
                .with_system(follow.before(animate_sprites))
                .with_system(animate_sprites)
                .with_system(display_health),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::steps_per_second(2.))
                .with_system(walk_enemies),
        )
        .add_system(move_camera)
        .add_system(move_player)
        .add_system(set_follow)
        .add_system(set_visibility)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::steps_per_second(10.))
                .with_system(combat),
        )
        .add_system(track_mouse_movement)
        .run();
}
