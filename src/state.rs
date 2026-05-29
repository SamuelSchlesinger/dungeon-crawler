use bevy::prelude::States;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default, States)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    /// Entered when a floor's victory condition is met (and the run cap has not
    /// been reached). The game is paused while the player picks one of three
    /// boons / spends gold (see `boon_select`). Picking advances to NextFloor.
    BoonSelect,
    /// Transient state entered after a boon is chosen. The OnEnter(NextFloor)
    /// system generates a fresh procedural floor and returns to Playing.
    NextFloor,
    Victory,
    Defeat,
}
