use bevy::prelude::*;

/// Represents the different states the game can be in.
/// This controls the game flow between title screen, playing, paused, and game over states.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Title,
    Playing,
    Paused,
    Spawning,
    GameOver,
}