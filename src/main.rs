use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use rand::seq::IndexedRandom;

// Module declarations
mod components;
mod constants;
mod game_logic;
mod game_state;
mod input;
mod resources;
mod setup;
mod tetromino;
mod ui;

// Re-export commonly used items
use components::Shape;
use game_state::GameState;
use resources::{FallTimer, Level, LinesCleared, NextPiece, Score};

fn main() {
    // Determine the very first piece to put into the NextPiece resource
    let first_next_shape = [
        Shape::I,
        Shape::O,
        Shape::T,
        Shape::L,
        Shape::J,
        Shape::S,
        Shape::Z,
    ]
    .choose(&mut rand::rng()) // Use thread_rng for initial randomness
    .unwrap()
    .clone();

    App::new()
        // Add the default Bevy plugins for rendering, window management, input, etc.
        .add_plugins((DefaultPlugins, EmbeddedAssetPlugin::default()))
        // This is where we'll add our game state logic.
        // We're initializing it to the "Playing" state.
        .init_state::<GameState>()
        // Insert Resources
        .insert_resource(FallTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .insert_resource(Score(0))
        .insert_resource(LinesCleared(0))
        .insert_resource(Level(1))
        .insert_resource(NextPiece(first_next_shape)) // Initialize the NextPiece resource

        // Add a startup system to set up the game environment once.
        .add_systems(Startup, setup::setup_camera)
        .add_systems(Startup, setup::setup_audio)
        
        // Add systems for the Title state
        .add_systems(
            OnEnter(GameState::Title),
            (ui::setup_title_screen, ui::despawn_game_board).chain(),
        )
        .add_systems(OnExit(GameState::Title), ui::despawn_title_screen)
        
        // Add systems for the Paused state
        .add_systems(OnEnter(GameState::Paused), ui::setup_pause_menu)
        .add_systems(OnExit(GameState::Paused), ui::despawn_pause_menu)

        // Add systems for the GameOver state
        .add_systems(OnEnter(GameState::GameOver), ui::setup_game_over_screen)
        .add_systems(OnExit(GameState::GameOver), ui::despawn_game_over_screen)

        // Systems for handling user input. This will now run in all states.
        .add_systems(Update, input::handle_input)
        
        // When we enter the Spawning state, we'll clear lines, spawn a new piece, and immediately
        // transition back to Playing.
        .add_systems(
            OnEnter(GameState::Spawning),
            (game_logic::clear_lines, tetromino::spawn_tetromino).chain(),
        )
        .add_systems(
            OnEnter(GameState::Playing),
            (setup::setup_grid, ui::setup_scoreboard, ui::setup_next_piece_preview).chain(),
        )
        // Add a system for the main game logic that runs during the `Playing` state.
        // `update_transforms` will sync grid positions with their visual transforms.
        .add_systems(
            Update,
            (game_logic::gravity_system, game_logic::update_transforms, ui::update_scoreboard, ui::update_next_piece_preview)
                .run_if(in_state(GameState::Playing)),
        )
        // System to update the fall speed when the level changes
        .add_systems(Update, game_logic::update_fall_speed)
        // Run the game!
        .run();
}


