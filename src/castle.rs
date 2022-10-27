use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    enemy::Enemy,
    grid::{Grid, Tile},
};

pub struct CastlePlugin;

impl Plugin for CastlePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerritoryInfo::new())
            .add_event::<NumberFilledEvent>()
            .add_event::<ExpandAreaEvent>()
            .add_system(startup)
            .add_system(spawn_castle)
            .add_system(number_filled)
            .add_system(enemy_collision);
    }
}

// resource
pub struct TerritoryInfo {
    pub radius: i32,
    pub x: usize,
    pub y: usize,
    pub bombs_percent: f32,
    pub battlements_style: bool,
}

impl TerritoryInfo {
    fn new() -> Self {
        TerritoryInfo {
            radius: 1,
            x: 10,
            y: 10,
            bombs_percent: 0.5,
            battlements_style: true,
        }
    }
}

// events
pub struct NumberFilledEvent;
pub struct ExpandAreaEvent;

#[derive(Component)]
pub struct Castle {
    pub health: u32,
    // colour
    pub money: u32,
}

impl Castle {
    pub fn take_damage(&mut self, damage: u32) {
        if damage >= self.health {
            self.health = 0;
            // you lose
        } else {
            self.health -= damage;
        }
        // send event
        // let writer: EventWriter<UpdateCastleStatsEvent> = EventWriter<UpdateCastleStatsEvent>::new();
    }
}

fn startup(mut ev_expand: EventWriter<ExpandAreaEvent>, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::F) {
        ev_expand.send(ExpandAreaEvent);
    }
}

fn spawn_castle(
    mut commands: Commands,
    q_tiles: Query<Entity, With<Tile>>,
    grid: Res<Grid>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::C) {
        if let Some(info) = grid.get_xy(10, 10) {
            if let Ok(ent) = q_tiles.get(info.entity) {
                let child = commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: Color::GOLD,
                            custom_size: Some(Vec2::splat(25.0)),
                            ..default()
                        },
                        transform: Transform::from_xyz(0.0, 0.0, 0.3),
                        ..default()
                    })
                    .insert(Castle {
                        health: 100,
                        money: 100,
                    })
                    .insert(Collider::cuboid(12.5, 12.5))
                    .insert(Sensor)
                    .id();
                commands.entity(ent).add_child(child);
            }
        }
    }
}

fn number_filled(
    ev_number_filled: EventReader<NumberFilledEvent>,
    mut ev_expand: EventWriter<ExpandAreaEvent>,
    mut info: ResMut<TerritoryInfo>,
) {
    if !ev_number_filled.is_empty() {
        ev_number_filled.clear();
        println!("Number filled. Expand reach");
        info.radius += 2;
        // 5x ring has 16 possibilities
        // 9x ring has 8*4 = 32
        // 13x ring has 12*4 = 48 possibilities
        // not guaranteed to be on the map
        // should this be a percent?
        // or free tiles?
        // info.bombs += 2;
        // percent
        info.bombs_percent += 0.1;
        ev_expand.send(ExpandAreaEvent);
    }
}

fn enemy_collision(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut q_castle: Query<(Entity, &mut Castle)>,
    q_enemies: Query<(Entity, &Enemy)>,
) {
    for (castle_ent, mut castle) in q_castle.iter_mut() {
        // let intersections = rapier_context.intersections_with(castle_ent);
        for (enemy_ent, _enemy) in q_enemies.iter() {
            if rapier_context.intersection_pair(castle_ent, enemy_ent) == Some(true) {
                commands.entity(enemy_ent).despawn_recursive();
                castle.take_damage(1);
            }
        }

        // is _inter relevant? It was always true when I tested other intersections
        // so when is it false?
        // for (a, b, _inter) in intersections {
        //     let enemy_ent = if a == castle_ent { b } else { a };
        //     if let Ok((enemy_ent, _enemy)) = q_enemies.get(enemy_ent) {
        //         commands.entity(enemy_ent).despawn_recursive();
        //         castle.take_damage(1);
        //     }
        // }
    }
}
