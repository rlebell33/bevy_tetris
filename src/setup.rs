use bevy::prelude::*;

use crate::constants::{BLOCK_SIZE, GRID_SIZE_X, GRID_SIZE_Y};

/// A startup system to spawn a 2D camera and the UI text.
pub fn setup_camera(mut commands: Commands) {
    // Spawn the camera.
    commands.spawn((
        Camera2d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        bevy::core_pipeline::bloom::Bloom::default(),
    ));
    println!("Camera set up successfully!");
}

/// A startup system to set up background audio.
pub fn setup_audio(asset_server: Res<AssetServer>, mut commands: Commands) {
    let asset_path = format!("embedded://sounds/162764.ogg");

    commands.spawn((
        AudioPlayer::new(asset_server.load(asset_path)),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            ..default()
        },
    ));
}

/// A startup system to spawn the empty grid squares.
pub fn setup_grid(mut commands: Commands) {
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
                )
                // add a border to each block
                .with_scale(Vec3::splat(0.95)),
            ));
        }
    }
    println!("Grid set up successfully!");
}