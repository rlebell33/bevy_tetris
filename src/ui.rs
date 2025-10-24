use bevy::prelude::*;

use crate::{
    components::{GameOverOverlay, PauseMenu, PreviewBlock, Scoreboard, TitleScreen},
    constants::{
        BLOCK_SIZE, GRID_SIZE_X, GRID_SIZE_Y, SCOREBOARD_FONT_SIZE, SCOREBOARD_LINE_TEXT_PADDING,
        SCOREBOARD_TEXT_PADDING,
    },
    resources::{Level, LinesCleared, NextPiece, Score},
    tetromino::{get_tetromino_blocks, get_tetromino_color},
};

/// A system to set up the title screen UI.
pub fn setup_title_screen(mut commands: Commands) {
    // A separate camera for the UI to prevent it from moving with the game camera
    commands.spawn((
        Camera2d::default(),
        Camera {
            hdr: true,
            // Renders the title screen UI on top of the main camera
            order: 1,
            ..default()
        },
        bevy::core_pipeline::bloom::Bloom::default(),
        TitleScreen,
    ));

    // Title text
    commands.spawn((
        Text::new("Tetris Remake"),
        TextFont {
            font_size: 50.0,
            ..default()
        },
        TextColor(bevy::prelude::Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(200.0),
            left: Val::Percent(50.0),
            // offset by half the text width to truly center it
            margin: UiRect {
                left: Val::Px(-150.0), // Approximate half the width of the text
                ..default()
            },
            ..default()
        },
        TitleScreen,
    ));

    // Instructions
    commands.spawn((
        Text::new("Press SPACE to start"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(bevy::prelude::Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(300.0),
            left: Val::Percent(50.0),
            // offset by half the text width to truly center it
            margin: UiRect {
                left: Val::Px(-70.0), // Approximate half the width of the text
                ..default()
            },
            ..default()
        },
        TitleScreen,
    ));

    commands.spawn((
        Text::new("P to pause | R to reset"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(bevy::prelude::Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(350.0),
            left: Val::Percent(50.0),
            // offset by half the text width to truly center it
            margin: UiRect {
                left: Val::Px(-85.0), // Approximate half the width of the text
                ..default()
            },
            ..default()
        },
        TitleScreen,
    ));
    println!("Title screen set up successfully!");
}

/// A system to despawn the title screen entities.
pub fn despawn_title_screen(mut commands: Commands, query: Query<Entity, With<TitleScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// A system to set up the pause menu.
pub fn setup_pause_menu(mut commands: Commands) {
    // Spawn a transparent background that covers the whole screen
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(bevy::prelude::Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseMenu,
        ))
        .with_children(|parent| {
            // "PAUSED" text
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(bevy::prelude::Color::WHITE),
            ));
        });
}

/// A system to despawn the pause menu.
pub fn despawn_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenu>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// A system to set up the game over screen.
pub fn setup_game_over_screen(mut commands: Commands) {
    // Spawn a transparent background that covers the whole screen
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(bevy::prelude::Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GameOverOverlay,
        ))
        .with_children(|parent| {
            // "GAME OVER" text
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(bevy::prelude::Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(200.0),
                    left: Val::Percent(50.0),
                    // offset by half the text width to truly center it
                    margin: UiRect {
                        left: Val::Px(-163.0), // Approximate half the width of the text
                        ..default()
                    },
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("Press R to restart"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(bevy::prelude::Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(275.0),
                    left: Val::Percent(50.0),
                    // offset by half the text width to truly center it
                    margin: UiRect {
                        left: Val::Px(-107.0), // Approximate half the width of the text
                        ..default()
                    },
                    ..default()
                },
            ));
        });
}

/// A system to despawn the game over screen.
pub fn despawn_game_over_screen(
    mut commands: Commands,
    query: Query<Entity, With<GameOverOverlay>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// A system to set up the static "NEXT" label and background box for the preview.
pub fn setup_next_piece_preview(mut commands: Commands) {
    // World coordinates for the top-right area, outside the grid
    let preview_center_x = (GRID_SIZE_X as f32 / 2.0 + 3.5) * BLOCK_SIZE;
    let preview_center_y = (GRID_SIZE_Y as f32 / 2.0 - 5.0) * BLOCK_SIZE;
    let preview_width = 6.0 * BLOCK_SIZE;
    let preview_height = 5.0 * BLOCK_SIZE;

    // 1. Static Preview Box (Background)
    commands.spawn((
        Sprite {
            color: bevy::prelude::Color::srgba(0.1, 0.1, 0.1, 0.9), // Dark background box
            custom_size: Some(Vec2::new(preview_width, preview_height)),
            ..default()
        },
        Transform::from_xyz(preview_center_x, preview_center_y, 0.5),
        PreviewBlock,
    ));

    commands.spawn((
        Text::new("Next"),
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
        PreviewBlock,
    ));
}

/// A system to draw the next piece blocks
pub fn update_next_piece_preview(
    mut commands: Commands,
    next_piece: Res<NextPiece>,
    block_query: Query<Entity, With<PreviewBlock>>,
) {
    // World coordinates for centering the blocks in the preview box
    let center_x = (GRID_SIZE_X as f32 / 2.0 + 3.5) * BLOCK_SIZE;
    let center_y = (GRID_SIZE_Y as f32 / 2.0 - 5.0) * BLOCK_SIZE;

    // Only update when the next piece resource has changed
    if next_piece.is_changed() {
        // 1. Despawn old preview blocks
        for entity in block_query.iter() {
            commands.entity(entity).despawn();
        }

        // Get the shape and color of the next piece
        let shape_to_preview = next_piece.0;
        let blocks = get_tetromino_blocks(shape_to_preview);
        let color = get_tetromino_color(shape_to_preview);

        // 3. Spawn the new preview blocks
        for block_position in blocks.iter() {
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                    ..default()
                },
                Transform::from_xyz(
                    center_x + block_position.x as f32 * BLOCK_SIZE,
                    center_y + block_position.y as f32 * BLOCK_SIZE,
                    1.5, // Z is higher than the box background
                ),
                PreviewBlock,
            ));
        }
    }
}

/// A system to set up the scoreboard UI.
pub fn setup_scoreboard(mut commands: Commands) {
    // Spawn the score, lines, and level text in a single container for clean UI
    commands.spawn((
        Text::new("Score: 0"),
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
        Text::new("Lines: 0"),
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

    // Spawn the scoreboard text for the level.
    commands.spawn((
        Text::new("Level: 1"),
        TextFont {
            font_size: SCOREBOARD_FONT_SIZE,
            ..default()
        },
        TextColor(bevy::prelude::Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0 + 2.0 * SCOREBOARD_FONT_SIZE),
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        },
        Scoreboard::Level,
    ));

    println!("UI set up successfully!");
}

/// A system that updates the scoreboard UI.
pub fn update_scoreboard(
    score: Res<Score>,
    lines_cleared: Res<LinesCleared>,
    level: Res<Level>,
    mut query: Query<(&mut Text, &Scoreboard)>,
) {
    for (mut text, scoreboard) in query.iter_mut() {
        match scoreboard {
            Scoreboard::Score => {
                *text = Text::new(format!("Score: {}", score.0));
            }
            Scoreboard::Lines => {
                *text = Text::new(format!("Lines: {}", lines_cleared.0));
            }
            Scoreboard::Level => {
                *text = Text::new(format!("Level: {}", level.0));
            }
        }
    }
}

/// System to despawn game board entities when transitioning back to title
pub fn despawn_game_board(
    mut commands: Commands,
    query1: Query<Entity, With<crate::components::GridPosition>>,
    query2: Query<Entity, With<Scoreboard>>,
    query3: Query<Entity, With<crate::components::Tetromino>>,
    query4: Query<Entity, With<Sprite>>,
    query5: Query<Entity, With<PreviewBlock>>,
) {
    for entity in query1.iter() {
        commands.entity(entity).despawn();
    }
    for entity in query2.iter() {
        commands.entity(entity).despawn();
    }
    for entity in query3.iter() {
        commands.entity(entity).despawn();
    }
    for entity in query4.iter() {
        commands.entity(entity).despawn();
    }
    for entity in query5.iter() {
        commands.entity(entity).despawn();
    }
}