use bevy::prelude::States;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default, States)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Victory,
    Defeat,
}
