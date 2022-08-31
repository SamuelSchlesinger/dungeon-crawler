#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameState {
    Playing,
    Victory,
    Defeat,
    Menu,
}
