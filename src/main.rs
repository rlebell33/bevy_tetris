use bevy::prelude::*;
use rand::seq::{IndexedRandom, SliceRandom}; // Required for picking a random shape

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Playing,
    Paused,
    GameOver,
}

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

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

#[derive(Component)]
pub struct Board;

#[derive(Component)]
pub struct Tetromino;

#[derive(Resource, Deref, DerefMut)]
struct FallTimer(Timer);

// A marker component to define the rotation center of a tetromino.
#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub struct RotationCenter(pub GridPosition);

const GRID_SIZE_X: i32 = 10;
const GRID_SIZE_Y: i32 = 20;
const BLOCK_SIZE: f32 = 25.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        // Insert a resource to control the fall speed.
        .insert_resource(FallTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_systems(Startup, (setup_camera, setup_grid, spawn_tetromino))
        .add_systems(Update, game_logic.run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_input)
        .add_systems(
            Update,
            (gravity_system, update_transforms).run_if(in_state(GameState::Playing)),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
    println!("Camera set up successfully");
}

fn setup_grid(mut commands: Commands) {
    for x in 0..GRID_SIZE_X {
        for y in 0..GRID_SIZE_Y {
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.2, 0.2, 0.2),
                    custom_size: Some(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                    ..default()
                },
                Transform::from_xyz(
                    (x as f32 - (GRID_SIZE_X as f32 / 2.0) + 0.5) * BLOCK_SIZE,
                    (y as f32 - (GRID_SIZE_Y as f32 / 2.0) + 0.5) * BLOCK_SIZE,
                    0.0,
                ),
                Board,
            ));
        }
    }
    println!("Grid set up successfully!");
}

fn game_logic() {}

fn handle_input(
    input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut tetromino_query: Query<&mut GridPosition, With<Tetromino>>,
    grid_query: Query<&GridPosition, Without<Tetromino>>,
) {
    if input.just_pressed(KeyCode::Space) {
        println!("Space was pressed!");
    }

    if input.just_pressed(KeyCode::KeyP) {
        if *current_state.get() == GameState::Playing {
            next_state.set(GameState::Paused);
            println!("Game Paused");
        } else {
            next_state.set(GameState::Playing);
            println!("Game Resumed");
        }
    }

    // Only process movement input if the game is playing
    if *current_state.get() == GameState::Playing {
        if input.just_pressed(KeyCode::ArrowLeft) {
            let mut can_move = true;
            for mut position in tetromino_query.iter_mut() {
                let new_pos = GridPosition {
                    x: position.x - 1,
                    y: position.y,
                };
                if check_collision(new_pos, &grid_query) {
                    can_move = false;
                    break;
                }
            }

            if can_move {
                for mut position in tetromino_query.iter_mut() {
                    position.x -= 1;
                }
            }
        }
        if input.just_pressed(KeyCode::ArrowRight) {
            let mut can_move = true;
            for mut position in tetromino_query.iter_mut() {
                let new_pos = GridPosition {
                    x: position.x + 1,
                    y: position.y,
                };
                if check_collision(new_pos, &grid_query) {
                    can_move = false;
                    break;
                }
            }
            if can_move {
                for mut position in tetromino_query.iter_mut() {
                    position.x += 1;
                }
            }
        }
        if input.just_pressed(KeyCode::ArrowDown) {
            let mut can_move = true;
            for mut position in tetromino_query.iter_mut() {
                let new_pos = GridPosition {
                    x: position.x,
                    y: position.y - 1,
                };
                if check_collision(new_pos, &grid_query) {
                    can_move = false;
                    break;
                }
            }
            if can_move {
                for mut position in tetromino_query.iter_mut() {
                    position.y -= 1;
                }
            }
        }
    }
}

// A system to make the tetrominoes fall automatically.
fn gravity_system(
    time: Res<Time>,
    mut fall_timer: ResMut<FallTimer>,
    mut tetromino_query: Query<(Entity, &mut GridPosition), With<Tetromino>>,
    grid_query: Query<&GridPosition, Without<Tetromino>>,
    mut commands: Commands,
) {
    fall_timer.tick(time.delta());

    if fall_timer.finished() {
        let mut can_move = true;
        for (entity, position) in tetromino_query.iter() {
            let new_pos = GridPosition {
                x: position.x,
                y: position.y - 1,
            };
            if check_collision(new_pos, &grid_query) {
                can_move = false;
                break;
            } 
        }

        if can_move {
            for (entity, mut position) in tetromino_query.iter_mut() {
                position.y -= 1;
            }
        } else {
            println!("Piece landed!");
            for (entity, _) in tetromino_query.iter() {
                commands.entity(entity).remove::<Tetromino>();
            }
            spawn_tetromino(commands);
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

// Checks for collisions with the game board boundaries or other pieces.
fn check_collision(
    new_pos: GridPosition,
    grid_query: &Query<&GridPosition, Without<Tetromino>>,
) -> bool {
    // Check for collisions with the floor or walls
    if new_pos.x < 0 || new_pos.x >= GRID_SIZE_X || new_pos.y < 0 {
        return true;
    }

    // Check for collisions with other static blocks
    for static_pos in grid_query.iter() {
        if static_pos.x == new_pos.x && static_pos.y == new_pos.y {
            return true;
        }
    }

    false
}

// A system to spawn a new tetromino.
fn spawn_tetromino(mut commands: Commands) {
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
            GridPosition { x: 0, y: 0 },
            GridPosition { x: 0, y: 1 },
            GridPosition { x: 0, y: 2 },
            GridPosition { x: 0, y: 3 },
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
    let initial_y_offset = GRID_SIZE_Y as i32 / 2;
    let initial_x_offset = GRID_SIZE_X as i32 / 2 - 1;

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
}
