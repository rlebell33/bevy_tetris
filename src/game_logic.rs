use bevy::prelude::*;
use std::collections::HashMap;

use crate::{
    components::{GridPosition, Tetromino},
    constants::{GRID_SIZE_X, GRID_SIZE_Y},
    game_state::GameState,
    resources::{FallTimer, Level, LinesCleared, Score},
};

/// Checks for collisions with the game board boundaries or other pieces.
pub fn check_collision(new_pos: GridPosition, static_blocks: &[GridPosition]) -> bool {
    // Check for collisions with the floor or walls
    if new_pos.x < 0 || new_pos.x >= GRID_SIZE_X || new_pos.y < 0 {
        return true;
    }
    // Check for collisions with other static blocks
    for static_pos in static_blocks.iter() {
        if static_pos.x == new_pos.x && static_pos.y == new_pos.y {
            return true;
        }
    }
    false
}

/// A system to make the tetrominoes fall automatically.
pub fn gravity_system(
    mut commands: Commands,
    time: Res<Time>,
    mut fall_timer: ResMut<FallTimer>,
    mut tetromino_query: Query<(Entity, &mut GridPosition), With<Tetromino>>,
    grid_query: Query<&GridPosition, Without<Tetromino>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    fall_timer.tick(time.delta());
    if fall_timer.finished() {
        // Collect the positions of all static blocks once for collision checks
        let static_blocks: Vec<GridPosition> = grid_query.iter().cloned().collect();
        let mut can_move = true;
        for (_entity, position) in tetromino_query.iter() {
            let new_pos = GridPosition {
                x: position.x,
                y: position.y - 1,
            };
            if check_collision(new_pos, &static_blocks) {
                can_move = false;
                break;
            }
        }

        if can_move {
            for (_entity, mut position) in tetromino_query.iter_mut() {
                position.y -= 1;
            }
        } else {
            println!("Piece landed!");
            // Remove the Tetromino component from the landed pieces
            for (entity, _) in tetromino_query.iter() {
                commands.entity(entity).remove::<Tetromino>();
            }
            next_state.set(GameState::Spawning);
        }
    }
}

/// This system keeps the visual transforms in sync with the logical grid positions.
pub fn update_transforms(mut query: Query<(&GridPosition, &mut Transform)>) {
    for (grid_position, mut transform) in query.iter_mut() {
        transform.translation.x =
            (grid_position.x as f32 - (GRID_SIZE_X as f32 / 2.0)) * crate::constants::BLOCK_SIZE + (crate::constants::BLOCK_SIZE / 2.0);
        transform.translation.y =
            (grid_position.y as f32 - (GRID_SIZE_Y as f32 / 2.0)) * crate::constants::BLOCK_SIZE + (crate::constants::BLOCK_SIZE / 2.0);
        transform.translation.z = 1.0; // Ensure tetrominoes are rendered above the grid
    }
}

/// A system that updates the fall speed based on the current level.
pub fn update_fall_speed(level: Res<Level>, mut fall_timer: ResMut<FallTimer>) {
    let speed_multiplier = 0.9_f32.powf((level.0 - 1) as f32);
    fall_timer.set_duration(std::time::Duration::from_secs_f32(1.0 * speed_multiplier));
}

/// A system that checks for and clears full rows, and shifts blocks down.
pub fn clear_lines(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut lines_cleared: ResMut<LinesCleared>,
    mut level: ResMut<Level>,
    mut grid_query: Query<(Entity, &mut GridPosition), Without<Tetromino>>,
) {
    // Group all static blocks by their Y coordinate.
    let mut rows: HashMap<i32, Vec<Entity>> = HashMap::new();
    for (entity, position) in grid_query.iter() {
        rows.entry(position.y).or_insert_with(Vec::new).push(entity);
    }

    let mut cleared_rows = 0;
    // Iterate from the bottom up to check for full rows.
    for y in 0..GRID_SIZE_Y {
        if let Some(entities) = rows.get(&y) {
            if entities.len() == GRID_SIZE_X as usize {
                cleared_rows += 1;
                // Despawn all entities in the cleared row.
                for entity in entities {
                    commands.entity(*entity).despawn();
                }
            } else if cleared_rows > 0 {
                // If this row is not full, and we've cleared rows below it,
                // move all blocks in this row down.
                for entity in entities {
                    if let Ok((_, mut position)) = grid_query.get_mut(*entity) {
                        position.y -= cleared_rows;
                    }
                }
            }
        }
    }

    // Update the score based on the number of lines cleared and the current level
    if cleared_rows > 0 {
        println!("Cleared {} lines!", cleared_rows);
        let points = match cleared_rows {
            1 => 40,
            2 => 100,
            3 => 300,
            4 => 1200,
            _ => 0,
        };
        score.0 += points * (level.0 + 1);
        lines_cleared.0 += cleared_rows as u32;

        // Check if the level needs to be increased
        if lines_cleared.0 / 5 > (level.0 - 1) {
            level.0 += 1;
            println!("Level up! Current Level: {}", level.0);
        }

        println!("Current Score: {}", score.0);
    }
}