use bevy::prelude::*;
use crate::components::Shape;

/// A resource to control the speed at which tetrominoes fall.
#[derive(Resource, Deref, DerefMut)]
pub struct FallTimer(pub Timer);

/// A resource to track the player's score.
#[derive(Resource)]
pub struct Score(pub u32);

/// A resource to track the number of lines cleared.
#[derive(Resource)]
pub struct LinesCleared(pub u32);

/// A resource to track the current level.
#[derive(Resource)]
pub struct Level(pub u32);

/// Resource to hold the shape of the next piece to spawn
#[derive(Resource, Clone, Copy)]
pub struct NextPiece(pub Shape);