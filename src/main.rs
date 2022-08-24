mod combat;
mod components;
mod map;
mod resources;
mod systems;

use bevy::{prelude::*, time::FixedTimestep};
use systems::*;

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
