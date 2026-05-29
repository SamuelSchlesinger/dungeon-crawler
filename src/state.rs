use bevy::prelude::States;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default, States)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    /// Transient state entered when a floor's victory condition is met but the
    /// run length cap has not been reached. The OnEnter(NextFloor) system
    /// generates a fresh procedural floor and returns to Playing.
    NextFloor,
    Victory,
    Defeat,
}
