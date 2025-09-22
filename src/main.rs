use bevy::prelude::*;
use rand::seq::{IndexedRandom};
use std::collections::HashMap;

// We'll define our game states here later to control the game flow (e.g., MainMenu, Playing, GameOver).
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Playing,
    Paused,
    Spawning,
    GameOver,
}

// Represents the position of a block on the game grid.
// This is different from the world transform.
#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

// Represents the color of a block.
#[derive(Component)]
pub struct Color(pub bevy::prelude::Color);

// Represents the different shapes a tetromino can have.
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

// A "marker" component to identify the active tetromino.
// Its presence on an entity signals that it is part of the currently falling piece.
#[derive(Component)]
pub struct Tetromino;

// A resource to control the speed at which tetrominoes fall.
#[derive(Resource, Deref, DerefMut)]
struct FallTimer(Timer);

// A marker component to define the rotation center of a tetromino.
#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub struct RotationCenter(pub GridPosition);

// A resource to track the player's score.
#[derive(Resource)]
struct Score(u32);

// A resource to track the number of lines cleared.
#[derive(Resource)]
struct LinesCleared(u32);

// A component to mark the entities that display the score and lines.
#[derive(Component)]
enum Scoreboard {
    Score,
    Lines,
}

// Constants for the game grid
const GRID_SIZE_X: i32 = 10;
const GRID_SIZE_Y: i32 = 20;
const BLOCK_SIZE: f32 = 25.0;

//Constants for the Scoreboard UI
const SCOREBOARD_FONT_SIZE: f32 = 25.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(50.0);
const SCOREBOARD_LINE_TEXT_PADDING: Val = Val::Px(50.0 + SCOREBOARD_FONT_SIZE);

fn main() {
    App::new()
        // Add the default Bevy plugins for rendering, window management, input, etc.
        .add_plugins(DefaultPlugins)
        
        // This is where we'll add our game state logic.
        // We're initializing it to the "Playing" state.
        .init_state::<GameState>()
        
        // Insert a resource to control the fall speed.
        .insert_resource(FallTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        // Insert a resource to track the score.
        .insert_resource(Score(0))
        // Insert a resource to track the number of lines cleared.
        .insert_resource(LinesCleared(0))
        
        // Add a startup system to set up the game environment once.
        // This system will be responsible for things like setting up the camera and the UI.
        .add_systems(Startup, (setup_ui, setup_grid, spawn_tetromino).chain())
        // Systems for handling user input
        .add_systems(Update, handle_input.run_if(in_state(GameState::Playing)))
        
        // When we enter the Spawning state, we'll clear lines, spawn a new piece, and immediately
        // transition back to Playing.
        .add_systems(OnEnter(GameState::Spawning), (clear_lines, spawn_tetromino).chain())
        // Add a system for the main game logic that runs during the `Playing` state.
        // `update_transforms` will sync grid positions with their visual transforms.
        .add_systems(Update, (gravity_system, update_transforms, update_scoreboard).run_if(in_state(GameState::Playing)))
        
        // Run the game!
        .run();
}

// A startup system to spawn a 2D camera and the UI text.
fn setup_ui(mut commands: Commands) {
    // Spawn the camera.
    commands.spawn(Camera2d::default());
    println!("Camera set up successfully!");
    
    // Spawn the scoreboard text for the score.
    commands.spawn((
        Text::new(
            "Score: 0"
        ),
        TextFont {
            font_size: SCOREBOARD_FONT_SIZE,
            ..default()
        },
        TextColor(bevy::prelude::Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        },
        Scoreboard::Score,
    ));
    
    // Spawn the scoreboard text for the lines cleared.
    commands.spawn((
        Text::new(
            "Lines: 0"
        ),
        TextFont {
            font_size: SCOREBOARD_FONT_SIZE,
            ..default()
        },
        TextColor(bevy::prelude::Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_LINE_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        },
        Scoreboard::Lines,
    ));
    println!("UI set up successfully!");
}

// A startup system to spawn the empty grid squares.
fn setup_grid(mut commands: Commands) {
    for x in 0..GRID_SIZE_X {
        for y in 0..GRID_SIZE_Y {
            commands.spawn((
                Sprite {
                    color: bevy::prelude::Color::srgb(0.2, 0.2, 0.2), // Dark gray color
                    custom_size: Some(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                    ..default()
                },
                Transform::from_xyz(
                    (x as f32 - (GRID_SIZE_X as f32 / 2.0) + 0.5) * BLOCK_SIZE,
                    (y as f32 - (GRID_SIZE_Y as f32 / 2.0) + 0.5) * BLOCK_SIZE,
                    0.0,
                ),
            ));
        }
    }
    println!("Grid set up successfully!");
}

// A system to handle user input for moving and rotating pieces.
// Bevy provides a `Res<ButtonInput<KeyCode>>` to check for key presses.
fn handle_input(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut tetromino_query: Query<(Entity, &mut GridPosition), With<Tetromino>>,
    grid_query: Query<&GridPosition, Without<Tetromino>>,
) {
    // Collect the positions of all static blocks once for collision checks
    let static_blocks: Vec<GridPosition> = grid_query.iter().cloned().collect();
    // Toggle between Playing and Paused states
    if input.just_pressed(KeyCode::KeyP) {
        if *current_state.get() == GameState::Playing {
            next_state.set(GameState::Paused);
            println!("Game Paused");
        } else {
            next_state.set(GameState::Playing);
            println!("Game Resumed");
        }
        return;
    }
    
    // Only process movement input if the game is playing
    if *current_state.get() == GameState::Playing {
        // Handle rotation first, as it can block movement
        if input.just_pressed(KeyCode::ArrowUp) {
            let mut can_rotate = true;
            let mut new_positions = Vec::new();
            
            // For a simple Tetris clone, we can assume the rotation center is the 2nd block
            // of a given tetromino.
            let rotation_center = tetromino_query.iter().nth(1).unwrap().1.clone();
            
            for (entity, position) in tetromino_query.iter() {
                // Calculate position relative to the rotation center
                let relative_x = position.x - rotation_center.x;
                let relative_y = position.y - rotation_center.y;
                
                // Rotate 90 degrees clockwise (x, y) -> (y, -x)
                let rotated_x = relative_y;
                let rotated_y = -relative_x;
                
                let new_pos = GridPosition {
                    x: rotated_x + rotation_center.x,
                    y: rotated_y + rotation_center.y,
                };
                
                if check_collision(new_pos, &static_blocks) {
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
        
        if input.just_pressed(KeyCode::ArrowLeft) {
            let mut can_move = true;
            for (_entity, position) in tetromino_query.iter() {
                let new_pos = GridPosition {
                    x: position.x - 1,
                    y: position.y,
                };
                if check_collision(new_pos, &static_blocks) {
                    can_move = false;
                    break;
                }
            }
            if can_move {
                for (_entity, mut position) in tetromino_query.iter_mut() {
                    position.x -= 1;
                }
            }
        }
        if input.just_pressed(KeyCode::ArrowRight) {
            let mut can_move = true;
            for (_entity, position) in tetromino_query.iter() {
                let new_pos = GridPosition {
                    x: position.x + 1,
                    y: position.y,
                };
                if check_collision(new_pos, &static_blocks) {
                    can_move = false;
                    break;
                }
            }
            if can_move {
                for (_entity, mut position) in tetromino_query.iter_mut() {
                    position.x += 1;
                }
            }
        }
        if input.just_pressed(KeyCode::ArrowDown) {
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
            }
        }
        // Hard drop logic for the Space key
        if input.just_pressed(KeyCode::Space) {
            let mut can_move = true;
            while can_move {
                let mut temp_positions: Vec<GridPosition> = Vec::new();
                for (_entity, position) in tetromino_query.iter() {
                    let new_pos = GridPosition {
                        x: position.x,
                        y: position.y - 1,
                    };
                    if check_collision(new_pos, &static_blocks) {
                        can_move = false;
                        break;
                    }
                    temp_positions.push(new_pos);
                }
                
                if can_move {
                    for (_entity, mut position) in tetromino_query.iter_mut() {
                        position.y -= 1;
                    }
                } else {
                    for (entity, _) in tetromino_query.iter() {
                        commands.entity(entity).remove::<Tetromino>();
                    }
                    next_state.set(GameState::Spawning);
                }
            }
        }
    }
}

// A system to make the tetrominoes fall automatically.
fn gravity_system(
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

// This system keeps the visual transforms in sync with the logical grid positions.
fn update_transforms(mut query: Query<(&GridPosition, &mut Transform)>) {
    for (grid_position, mut transform) in query.iter_mut() {
        transform.translation.x =
            (grid_position.x as f32 - (GRID_SIZE_X as f32 / 2.0)) * BLOCK_SIZE + (BLOCK_SIZE / 2.0);
        transform.translation.y =
            (grid_position.y as f32 - (GRID_SIZE_Y as f32 / 2.0)) * BLOCK_SIZE + (BLOCK_SIZE / 2.0);
        transform.translation.z = 1.0; // Ensure tetrominoes are rendered above the grid
    }
}

// A system that updates the scoreboard UI.
fn update_scoreboard(
    score: Res<Score>,
    lines_cleared: Res<LinesCleared>,
    mut query: Query<(&mut Text, &Scoreboard)>,
) {
    for (mut text, scoreboard) in query.iter_mut() {
        match scoreboard {
            Scoreboard::Score => {
                *text = Text::new(
                    format!("Score: {}", score.0)
                );
            }
            Scoreboard::Lines => {
                *text = Text::new(
                    format!("Lines: {}", lines_cleared.0)
                );
            }
        }
    }
}

// Checks for collisions with the game board boundaries or other pieces.
fn check_collision(
    new_pos: GridPosition,
    static_blocks: &[GridPosition],
) -> bool {
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

// Spawns a new tetromino and transitions the state.
fn spawn_tetromino(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>, grid_query: Query<&GridPosition, Without<Tetromino>>) {
    // A list of the seven tetromino shapes
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
    let random_shape = shapes.choose(&mut rand::rng()).unwrap();
    // Define the blocks for each shape, relative to the piece's origin
    let blocks = match random_shape {
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
    };
    // Define the color for the tetromino based on its shape
    let color = match random_shape {
        Shape::I => bevy::prelude::Color::srgb(0.0, 1.0, 1.0), // Cyan
        Shape::O => bevy::prelude::Color::srgb(1.0, 1.0, 0.0), // Yellow
        Shape::T => bevy::prelude::Color::srgb(0.5, 0.0, 0.5), // Purple
        Shape::L => bevy::prelude::Color::srgb(1.0, 0.65, 0.0), // Orange
        Shape::J => bevy::prelude::Color::srgb(0.0, 0.0, 1.0), // Blue
        Shape::S => bevy::prelude::Color::srgb(0.0, 1.0, 0.0), // Green
        Shape::Z => bevy::prelude::Color::srgb(1.0, 0.0, 0.0), // Red
    };
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
    for block_position in blocks {
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                ..default()
            },
            GridPosition {
                x: block_position.x + initial_x_offset,
                y: block_position.y + initial_y_offset,
            },
            Tetromino,
        ));
    }
    println!("New tetromino spawned!");
    next_state.set(GameState::Playing);
}

/// A system that checks for and clears full rows, and shifts blocks down.
fn clear_lines(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut lines_cleared: ResMut<LinesCleared>,
    mut grid_query: Query<(Entity, &mut GridPosition), Without<Tetromino>>,
) {
    // Group all static blocks by their Y coordinate.
    let mut rows: HashMap<i32, Vec<Entity>> = HashMap::new();
    for (entity, position) in grid_query.iter() {
        rows.entry(position.y)
            .or_insert_with(Vec::new)
            .push(entity);
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
    
    // Update the score and lines cleared based on the number of lines cleared.
    if cleared_rows > 0 {
        println!("Cleared {} lines!", cleared_rows);
        score.0 += cleared_rows as u32;
        lines_cleared.0 += cleared_rows as u32;
        println!("Current Score: {}", score.0);
    }
}
