use bevy::prelude::*;

use crate::{
    components::{GridPosition, RotationCenter, Tetromino},
    game_logic::check_collision,
    game_state::GameState,
    resources::{Level, LinesCleared, Score},
};

/// A system to handle user input for moving and rotating pieces.
/// Bevy provides a `Res<ButtonInput<KeyCode>>` to check for key presses.
pub fn handle_input(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut tetromino_query: Query<(Entity, &mut GridPosition, Option<&RotationCenter>), With<Tetromino>>,
    grid_query: Query<&GridPosition, Without<Tetromino>>,
    grid_entities: Query<Entity, With<GridPosition>>,
) {
    // Start the game from the title screen
    if *current_state.get() == GameState::Title && input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Spawning);
        println!("Game started!");
        return;
    }

    // Toggle between Playing and Paused states
    if input.just_pressed(KeyCode::KeyP) {
        if *current_state.get() == GameState::Playing {
            next_state.set(GameState::Paused);
            println!("Game Paused");
        } else if *current_state.get() == GameState::Paused {
            next_state.set(GameState::Playing);
            println!("Game Resumed");
        }
        return;
    }

    // Reset the game when 'R' is pressed
    if input.just_pressed(KeyCode::KeyR)
        && (*current_state.get() == GameState::Playing
            || *current_state.get() == GameState::Paused
            || *current_state.get() == GameState::GameOver)
    {
        println!("Resetting Game");
        for entity in grid_entities.iter() {
            commands.entity(entity).despawn();
        }
        commands.insert_resource(Score(0));
        commands.insert_resource(LinesCleared(0));
        commands.insert_resource(Level(1));
        next_state.set(GameState::Title);
        return;
    }

    // Only process movement input if the game is playing
    if *current_state.get() == GameState::Playing {
        // Collect the positions of all static blocks once for collision checks
        let static_blocks: Vec<GridPosition> = grid_query.iter().cloned().collect();
        
        // Handle rotation first, as it can block movement
        if input.just_pressed(KeyCode::ArrowUp) {
            handle_rotation(&mut tetromino_query, &static_blocks);
        }
        
        if input.just_pressed(KeyCode::ArrowLeft) {
            handle_horizontal_movement(&mut tetromino_query, &static_blocks, -1);
        }
        
        if input.just_pressed(KeyCode::ArrowRight) {
            handle_horizontal_movement(&mut tetromino_query, &static_blocks, 1);
        }
        
        if input.just_pressed(KeyCode::ArrowDown) {
            handle_vertical_movement(&mut tetromino_query, &static_blocks, -1);
        }
        
        // Hard drop logic for the Space key
        if input.just_pressed(KeyCode::Space) {
            handle_hard_drop(&mut commands, &mut tetromino_query, &static_blocks, &mut next_state);
        }
    }
}

/// Handles tetromino rotation
fn handle_rotation(
    tetromino_query: &mut Query<(Entity, &mut GridPosition, Option<&RotationCenter>), With<Tetromino>>,
    static_blocks: &[GridPosition],
) {
    let mut can_rotate = true;
    let mut new_positions = Vec::new();

    // Find the rotation center's current grid position
    let rotation_center_pos = tetromino_query
        .iter()
        .find_map(|(_, pos, center)| {
            if center.is_some() {
                Some(*pos)
            } else {
                None
            }
        })
        .unwrap_or(GridPosition { x: 0, y: 0 });

    for (entity, position, _) in tetromino_query.iter() {
        // Calculate position relative to the rotation center
        let relative_x = position.x - rotation_center_pos.x;
        let relative_y = position.y - rotation_center_pos.y;

        // Rotate 90 degrees clockwise: (x, y) -> (y, -x)
        let rotated_x = relative_y;
        let rotated_y = -relative_x;

        let new_pos = GridPosition {
            x: rotated_x + rotation_center_pos.x,
            y: rotated_y + rotation_center_pos.y,
        };

        if check_collision(new_pos, static_blocks) {
            can_rotate = false;
            break;
        }
        new_positions.push((entity, new_pos));
    }

    if can_rotate {
        for (entity, new_pos) in new_positions {
            let mut position = tetromino_query.get_mut(entity).unwrap().1;
            *position = new_pos;
        }
    }
}

/// Handles horizontal movement (left/right)
fn handle_horizontal_movement(
    tetromino_query: &mut Query<(Entity, &mut GridPosition, Option<&RotationCenter>), With<Tetromino>>,
    static_blocks: &[GridPosition],
    direction: i32,
) {
    let mut can_move = true;
    for (_entity, position, _) in tetromino_query.iter() {
        let new_pos = GridPosition {
            x: position.x + direction,
            y: position.y,
        };
        if check_collision(new_pos, static_blocks) {
            can_move = false;
            break;
        }
    }
    if can_move {
        for (_entity, mut position, _) in tetromino_query.iter_mut() {
            position.x += direction;
        }
    }
}

/// Handles vertical movement (down)
fn handle_vertical_movement(
    tetromino_query: &mut Query<(Entity, &mut GridPosition, Option<&RotationCenter>), With<Tetromino>>,
    static_blocks: &[GridPosition],
    direction: i32,
) {
    let mut can_move = true;
    for (_entity, position, _) in tetromino_query.iter() {
        let new_pos = GridPosition {
            x: position.x,
            y: position.y + direction,
        };
        if check_collision(new_pos, static_blocks) {
            can_move = false;
            break;
        }
    }
    if can_move {
        for (_entity, mut position, _) in tetromino_query.iter_mut() {
            position.y += direction;
        }
    }
}

/// Handles hard drop (space key)
fn handle_hard_drop(
    commands: &mut Commands,
    tetromino_query: &mut Query<(Entity, &mut GridPosition, Option<&RotationCenter>), With<Tetromino>>,
    static_blocks: &[GridPosition],
    next_state: &mut ResMut<NextState<GameState>>,
) {
    let mut can_move = true;
    while can_move {
        let mut temp_positions: Vec<GridPosition> = Vec::new();
        for (_entity, position, _) in tetromino_query.iter() {
            let new_pos = GridPosition {
                x: position.x,
                y: position.y - 1,
            };
            if check_collision(new_pos, static_blocks) {
                can_move = false;
                break;
            }
            temp_positions.push(new_pos);
        }

        if can_move {
            for (_entity, mut position, _) in tetromino_query.iter_mut() {
                position.y -= 1;
            }
        } else {
            for (entity, _, _) in tetromino_query.iter() {
                commands.entity(entity).remove::<Tetromino>();
            }
            next_state.set(GameState::Spawning);
        }
    }
}