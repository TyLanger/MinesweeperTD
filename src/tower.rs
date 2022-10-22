use bevy::prelude::*;

use crate::{
    grid::{ClearSelectionsEvent, Selection},
    ui::ButtonPressEvent,
};

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_tower);
    }
}

fn spawn_tower(
    mut commands: Commands,
    mut ev_button_press: EventReader<ButtonPressEvent>,
    q_selection: Query<Entity, With<Selection>>,
    mut ev_clear_selection: EventWriter<ClearSelectionsEvent>,
) {
    for ev in ev_button_press.iter() {
        let mut color = Color::WHITE;
        match ev.button_number {
            0 => {
                println!("Build tower 0");
                color = Color::GREEN;
            }
            1 => {
                println!("Build tower 1");
                color = Color::RED;
            }
            2 => {
                println!("Build tower 2");
                color = Color::CRIMSON;
            }
            3 => {
                println!("Build tower 3");
                color = Color::ORANGE;
            }
            _ => {}
        }
        ev_clear_selection.send(ClearSelectionsEvent);
        for ent in q_selection.iter() {
            //commands.entity().add_child(child)

            let child = commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::new(15.0, 15.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                    ..default()
                })
                .id();
            commands.entity(ent).add_child(child);
        }
    }
}
