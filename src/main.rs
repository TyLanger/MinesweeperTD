// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{prelude::*, render::camera::RenderTarget};
use bevy_rapier2d::prelude::*;

mod castle;
mod director;
mod enemy;
mod grid;
mod tower;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(grid::GridPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(tower::TowerPlugin)
        .add_plugin(castle::CastlePlugin)
        .add_plugin(director::DirectorPlugin)
        .add_plugin(enemy::EnemyPlugin)
        .insert_resource(MouseWorldPos(Vec2::ONE * 10000.0))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .add_startup_system(setup)
        .add_system(update_mouse_position)
        .run();
}

pub struct MouseWorldPos(Vec2);

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn update_mouse_position(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut mouse_pos: ResMut<MouseWorldPos>,
) {
    //let (camera, camera_transform) = q_camera.single();
    for (camera, camera_transform) in q_camera.iter() {
        let win = if let RenderTarget::Window(id) = camera.target {
            windows.get(id).unwrap()
        } else {
            windows.get_primary().unwrap()
        };

        if let Some(screen_pos) = win.cursor_position() {
            let window_size = Vec2::new(win.width() as f32, win.height() as f32);

            // convert screen position [0..resolution] to ndc [-1..1] (gpu coords)
            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

            // matrix for undoing the projection and camera transform
            let ndc_to_world =
                camera_transform.compute_matrix() * camera.projection_matrix().inverse();

            // use it to convert ndc to world-space coordinates
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

            // reduce it to a 2D value
            let world_pos: Vec2 = world_pos.truncate();

            mouse_pos.0 = world_pos;
        }
    }
}
