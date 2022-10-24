use bevy::prelude::*;

use crate::{
    grid::{interaction, ClearSelectionsEvent, Grid, Selection, Tile, TileState},
    ui::ButtonPressEvent,
};

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        // this before just doesn't work?
        // Clear interaction
        // Build tower 1
        // Clear button

        // good
        // Build tower 2
        // Clear button false
        // Clear interaction

        // bad
        // Clear interaction
        // Build tower 0
        // Clear button true
        app.add_event::<TowerPlacedEvent>()
            .add_system(spawn_tower.before(interaction));
    }
}

pub struct TowerPlacedEvent {
    pub x: usize,
    pub y: usize,
    // type of tower?
}

fn spawn_tower(
    mut commands: Commands,
    mut ev_button_press: EventReader<ButtonPressEvent>,
    mut q_selection: Query<(Entity, &mut Tile), With<Selection>>,
    grid: Res<Grid>,
    q_tiles: Query<&Tile, Without<Selection>>,
    mut ev_clear_selection: EventWriter<ClearSelectionsEvent>,
    mut ev_tower_placed: EventWriter<TowerPlacedEvent>,
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
                color = Color::BLUE;
            }
            3 => {
                println!("Build tower 3");
                color = Color::ORANGE;
            }
            _ => {}
        }
        //println!("Clear button {:}", q_selection.is_empty());

        ev_clear_selection.send(ClearSelectionsEvent);
        for (ent, mut tile) in q_selection.iter_mut() {
            //commands.entity().add_child(child)

            let mut floor_nearby = false;
            for neighbour in grid.get_ring(tile.x, tile.y, 1) {
                if let Some(info) = neighbour {
                    if let Ok(tile) = q_tiles.get(info.entity) {
                        if tile.tile_state == TileState::Floor {
                            floor_nearby = true;
                            break;
                        }
                    }
                }
            }
            if !floor_nearby {
                println!("Tower failed. No floor nearby {}, {}", tile.x, tile.y);
                continue;
            }
            let result = tile.try_spawn_tower();
            match result {
                Ok(_) => {
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
                    println!("Placed a tower at {},{}", tile.x, tile.y);
                    ev_tower_placed.send(TowerPlacedEvent {
                        x: tile.x,
                        y: tile.y,
                    });
                }
                Err(e) => {
                    println!("Failed to spawn. {:?}", e);
                }
            }
        }
    }
}
