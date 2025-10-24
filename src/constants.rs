use bevy::prelude::*;

/// Constants for the game grid
pub const GRID_SIZE_X: i32 = 10;
pub const GRID_SIZE_Y: i32 = 20;
pub const BLOCK_SIZE: f32 = 25.0;

/// Constants for the Scoreboard UI
pub const SCOREBOARD_FONT_SIZE: f32 = 25.0;
pub const SCOREBOARD_TEXT_PADDING: Val = Val::Px(50.0);
pub const SCOREBOARD_LINE_TEXT_PADDING: Val = Val::Px(50.0 + SCOREBOARD_FONT_SIZE);