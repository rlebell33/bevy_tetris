use bevy::prelude::*;

/// Represents the position of a block on the game grid.
/// This is different from the world transform.
#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

/// Represents the color of a block.
#[derive(Component)]
pub struct Color(pub bevy::prelude::Color);

/// Represents the different shapes a tetromino can have.
#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Shape {
    I,
    O,
    T,
    L,
    J,
    S,
    Z,
}

/// A "marker" component to identify the active tetromino.
/// Its presence on an entity signals that it is part of the currently falling piece.
#[derive(Component)]
pub struct Tetromino;

/// A marker component to define the rotation center of a tetromino.
#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub struct RotationCenter(pub GridPosition);

/// A component to mark the entities that display the score and lines.
#[derive(Component)]
pub enum Scoreboard {
    Score,
    Lines,
    Level,
}

/// A component to identify all entities on the title screen
#[derive(Component)]
pub struct TitleScreen;

/// A component to identify all entities on the pause screen
#[derive(Component)]
pub struct PauseMenu;

/// A component to identify game over overlay entities
#[derive(Component)]
pub struct GameOverOverlay;

/// Marker for blocks that are part of the next piece preview
#[derive(Component)]
pub struct PreviewBlock;