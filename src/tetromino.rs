use bevy::prelude::*;
use rand::seq::IndexedRandom;

use crate::{
    components::{GridPosition, RotationCenter, Shape, Tetromino},
    constants::{BLOCK_SIZE, GRID_SIZE_X, GRID_SIZE_Y},
    game_logic::check_collision,
    game_state::GameState,
    resources::NextPiece,
};

/// Returns the block positions for a given tetromino shape, relative to the piece's origin
pub fn get_tetromino_blocks(shape: Shape) -> Vec<GridPosition> {
    match shape {
        Shape::I => vec![
            GridPosition { x: -1, y: 0 },
            GridPosition { x: 0, y: 0 },
            GridPosition { x: 1, y: 0 },
            GridPosition { x: 2, y: 0 },
        ],
        Shape::O => vec![
            GridPosition { x: 0, y: 0 },
            GridPosition { x: 0, y: 1 },
            GridPosition { x: 1, y: 0 },
            GridPosition { x: 1, y: 1 },
        ],
        Shape::T => vec![
            GridPosition { x: 0, y: 0 },
            GridPosition { x: -1, y: 0 },
            GridPosition { x: 1, y: 0 },
            GridPosition { x: 0, y: 1 },
        ],
        Shape::L => vec![
            GridPosition { x: 0, y: 0 },
            GridPosition { x: -1, y: 0 },
            GridPosition { x: 1, y: 0 },
            GridPosition { x: 1, y: 1 },
        ],
        Shape::J => vec![
            GridPosition { x: 0, y: 0 },
            GridPosition { x: -1, y: 0 },
            GridPosition { x: 1, y: 0 },
            GridPosition { x: -1, y: 1 },
        ],
        Shape::S => vec![
            GridPosition { x: 0, y: 0 },
            GridPosition { x: 1, y: 0 },
            GridPosition { x: 0, y: 1 },
            GridPosition { x: -1, y: 1 },
        ],
        Shape::Z => vec![
            GridPosition { x: 0, y: 0 },
            GridPosition { x: -1, y: 0 },
            GridPosition { x: 0, y: 1 },
            GridPosition { x: 1, y: 1 },
        ],
    }
}

/// Returns the color for a given tetromino shape
pub fn get_tetromino_color(shape: Shape) -> bevy::prelude::Color {
    match shape {
        Shape::I => bevy::prelude::Color::srgba(0.0, 2.0, 2.0, 0.8), // Cyan
        Shape::O => bevy::prelude::Color::srgba(2.0, 2.0, 0.0, 0.8), // Yellow
        Shape::T => bevy::prelude::Color::srgba(1.5, 0.0, 1.5, 0.8), // Purple
        Shape::L => bevy::prelude::Color::srgba(2.0, 1.65, 0.0, 0.8), // Orange
        Shape::J => bevy::prelude::Color::srgba(0.0, 0.0, 2.0, 0.8), // Blue
        Shape::S => bevy::prelude::Color::srgba(0.0, 2.0, 0.0, 0.8), // Green
        Shape::Z => bevy::prelude::Color::srgba(2.0, 0.0, 0.0, 0.8), // Red
    }
}

/// Returns the index of the rotation center block for a given shape
pub fn get_rotation_center_index(shape: Shape) -> Option<usize> {
    match shape {
        Shape::I => Some(1),
        Shape::O => None,
        Shape::T => Some(0),
        Shape::L => Some(0),
        Shape::J => Some(0),
        Shape::S => Some(0),
        Shape::Z => Some(0),
    }
}

/// Spawns a new tetromino and transitions the state.
pub fn spawn_tetromino(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    grid_query: Query<&GridPosition, Without<Tetromino>>,
    mut next_piece: ResMut<NextPiece>,
) {
    // 1. Determine the shape to spawn (It's the NextPiece from the previous cycle)
    let current_shape_to_spawn = next_piece.0;

    // 2. Generate the shape for the *next* spawn (and store it)
    let shapes = [
        Shape::I,
        Shape::O,
        Shape::T,
        Shape::L,
        Shape::J,
        Shape::S,
        Shape::Z,
    ];
    // Pick a random shape from the list
    let new_next_shape = shapes.choose(&mut rand::rng()).unwrap().clone();
    next_piece.0 = new_next_shape;

    // Get the blocks and color for the current shape
    let blocks = get_tetromino_blocks(current_shape_to_spawn);
    let color = get_tetromino_color(current_shape_to_spawn);
    
    // Set the initial position of the tetromino's origin
    let initial_y_offset = GRID_SIZE_Y as i32 - 1;
    let initial_x_offset = GRID_SIZE_X as i32 / 2 - 1;

    // Check for game over condition
    let static_blocks: Vec<GridPosition> = grid_query.iter().cloned().collect();
    for block_position in &blocks {
        let new_pos = GridPosition {
            x: block_position.x + initial_x_offset,
            y: block_position.y + initial_y_offset,
        };
        if check_collision(new_pos, &static_blocks) {
            println!("Game Over!");
            next_state.set(GameState::GameOver);
            return;
        }
    }

    // Spawn the individual blocks for the new tetromino
    for (i, block_position) in blocks.iter().enumerate() {
        let mut entity_commands = commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                ..default()
            },
            Transform::from_xyz(
                (block_position.x + initial_x_offset) as f32 * BLOCK_SIZE,
                (block_position.y + initial_y_offset) as f32 * BLOCK_SIZE,
                1.0,
            ),
            GridPosition {
                x: block_position.x + initial_x_offset,
                y: block_position.y + initial_y_offset,
            },
            Tetromino,
        ));
        
        // Add the rotation center component to the correct block
        if let Some(center_index) = get_rotation_center_index(current_shape_to_spawn) {
            if i == center_index {
                entity_commands.insert(RotationCenter(GridPosition {
                    x: block_position.x + initial_x_offset,
                    y: block_position.y + initial_y_offset,
                }));
            }
        }
    }
    println!("New tetromino spawned!");
    next_state.set(GameState::Playing);
}