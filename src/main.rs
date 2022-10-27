// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use minesweeper_td::GamePlugin;

fn main() {
    App::new()
        // #63c74d light green
        // #f77622 orange
        // #193c3e v dark green
        .insert_resource(ClearColor(Color::rgb_u8(0x19, 0x3c, 0x3e)))
        // .insert_resource(ClearColor(Color::rgb_u8(0xf7, 0x76, 0x22)))
        // .insert_resource(ClearColor(Color::rgb_u8(0x63, 0xc7, 0x4d)))
        .insert_resource(WindowDescriptor {
            width: 1280.0,
            height: 720.0,
            title: "Minesweeper TD".to_string(),
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(ShapePlugin)
        .add_plugin(GamePlugin)
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
