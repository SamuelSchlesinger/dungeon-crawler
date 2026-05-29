use bevy::prelude::*;

use crate::{maps, systems::setup_play::*};

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    // The main menu is now drawn with egui (see `systems::menu`), so `setup`
    // only initializes the gameplay resources/camera. The old overlapping
    // `Text` menu entities have been removed.
    let map = maps::unbeatable();
    let initial_position = map.room.initial_position;
    let tiles_texture_handle = get_tiles_texture_handle(&asset_server, &mut texture_atlases);
    initialize_resources(&mut commands, &map, initial_position, &tiles_texture_handle, None);
}
