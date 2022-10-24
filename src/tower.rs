use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    grid::{interaction, ClearSelectionsEvent, Grid, Selection, Tile, TileState},
    ui::ButtonPressEvent,
    MouseWorldPos,
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
            .insert_resource(TowerServer { towers: Vec::new() })
            .add_startup_system(setup_towers)
            .add_system(spawn_tower.before(interaction))
            .add_system(tower_shoot)
            .add_system(move_bullets);
    }
}

pub struct TowerPlacedEvent {
    pub x: usize,
    pub y: usize,
    // type of tower?
}

#[derive(Component, Clone)]
struct Tower {
    range: f32,
    cost: u32,
    bullet: Bullet,
    gun: Gun,
}

#[derive(Component, Clone)]
struct Bullet {
    impact_type: ImpactType,
    damage: u32,
    movement: Movement,
}

impl Bullet {
    fn update_target(&mut self, target: Target) -> Self {
        self.movement.target = target;
        self.clone()
    }
}

#[derive(Clone)]
enum ImpactType {
    // number of pierces
    Pierce(usize),
    // radius of the explosion
    Explosion(f32),
}

#[derive(Clone)]
struct Movement {
    target: Target,
    speed: f32,
    // turn_radius: f32 or turn_angle??
}

#[derive(Default, Clone)]
enum Target {
    #[default]
    None,
    Point(Option<Vec3>),
    Follow(Option<Entity>),
    Direction(Option<Vec3>),
}

#[derive(Clone)]
struct Gun {
    clip_size: u32,
    // timer or float?
    time_between_shots: f32,
    timer_between: Timer,
    reload_time: f32,
    reload_timer: Timer,
    // state?
    // shooting, reloading, rest?
    multi_type: MultiShotType,
}

impl Gun {
    fn new(
        clip_size: u32,
        time_between_shots: f32,
        reload_time: f32,
        multi_type: MultiShotType,
    ) -> Self {
        Gun {
            clip_size,
            time_between_shots,
            timer_between: Timer::from_seconds(time_between_shots, true),
            reload_time,
            reload_timer: Timer::from_seconds(reload_time, false),
            multi_type,
        }
    }
}

// ShotType
// burst vs spread
#[derive(Default, Clone)]
enum MultiShotType {
    #[default]
    Spread,
    Burst,
}

struct TowerServer {
    towers: Vec<Tower>,
}

// set up the different towers you can spawn here.
fn setup_towers(mut tower_server: ResMut<TowerServer>) {
    let basic_tower = Tower {
        range: 60.0,
        cost: 10,
        bullet: Bullet {
            impact_type: ImpactType::Pierce(0),
            damage: 1,
            movement: Movement {
                // should I even set a default?
                // when the bullet is spawned, it should change this
                // target: Target::Point(Vec3::ZERO),
                target: Target::Direction(None),
                speed: 100.0,
            },
        },
        gun: Gun {
            clip_size: 1,
            time_between_shots: 0.1,
            timer_between: Timer::from_seconds(0.1, true),
            reload_time: 1.5,
            reload_timer: Timer::from_seconds(1.5, false),
            multi_type: MultiShotType::Burst,
        },
    };
    tower_server.towers.push(basic_tower);

    let shotgun_tower = Tower {
        range: 60.0,
        cost: 10,
        bullet: Bullet {
            impact_type: ImpactType::Pierce(0),
            damage: 2,
            movement: Movement {
                target: Target::Direction(None),
                speed: 100.0,
            },
        },
        gun: Gun {
            clip_size: 2,
            time_between_shots: 0.3,
            timer_between: Timer::from_seconds(0.3, true),
            reload_time: 1.5,
            reload_timer: Timer::from_seconds(1.5, false),
            multi_type: MultiShotType::Spread,
        },
    };
    tower_server.towers.push(shotgun_tower);

    let bomb_tower = Tower {
        range: 100.0,
        cost: 20,
        bullet: Bullet {
            impact_type: ImpactType::Explosion(20.0),
            damage: 3,
            movement: Movement {
                target: Target::Point(None),
                speed: 100.0,
            },
        },
        gun: Gun::new(1, 0.5, 2.0, MultiShotType::Burst),
    };
    tower_server.towers.push(bomb_tower);

    let swarm_tower = Tower {
        range: 100.0,
        cost: 20,
        bullet: Bullet {
            impact_type: ImpactType::Pierce(3),
            damage: 2,
            movement: Movement {
                target: Target::Point(None),
                speed: 100.0,
            },
        },
        gun: Gun::new(4, 0.0, 2.0, MultiShotType::Spread),
    };
    tower_server.towers.push(swarm_tower);
}

fn spawn_tower(
    mut commands: Commands,
    mut ev_button_press: EventReader<ButtonPressEvent>,
    mut q_selection: Query<(Entity, &mut Tile), With<Selection>>,
    grid: Res<Grid>,
    q_tiles: Query<&Tile, Without<Selection>>,
    mut ev_clear_selection: EventWriter<ClearSelectionsEvent>,
    mut ev_tower_placed: EventWriter<TowerPlacedEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tower_server: Res<TowerServer>,
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
                        .insert(tower_server.towers.get(ev.button_number).unwrap().clone())
                        .with_children(|tower| {
                            tower.spawn_bundle(MaterialMesh2dBundle {
                                // #0099db
                                // 30 is an arbitrary range
                                // these overlap with one another
                                // if you get enough, it becomes solid
                                mesh: meshes.add(shape::Circle::new(30.0).into()).into(),
                                material: materials.add(ColorMaterial::from(Color::rgba_u8(
                                    0x00, 0x99, 0xdb, 0x35,
                                ))),
                                // set visibility to true when you click on it?
                                visibility: Visibility { is_visible: false },
                                ..default()
                            });
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

// towers are a child. So their transform is 0,0
// need to use global to get their real pos (their parent's pos)
fn tower_shoot(
    mut commands: Commands,
    q_towers: Query<(&GlobalTransform, &Tower)>,
    keyboard: Res<Input<KeyCode>>,
    mouse: Res<MouseWorldPos>,
) {
    if keyboard.just_pressed(KeyCode::T) {
        for (trans, tower) in q_towers.iter() {
            let mut target = Target::None;
            match tower.bullet.movement.target {
                Target::None => todo!(),
                Target::Point(_) => {
                    target = Target::Point(Some(mouse.0.extend(0.0)));
                }
                Target::Follow(_) => todo!(),
                Target::Direction(_) => {
                    target = Target::Direction(Some(mouse.0.extend(0.0) - trans.translation()));
                }
            }
            //println!("Spawn bullet at {:?}", trans.translation());
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::BLACK,
                        custom_size: Some(Vec2::new(5.0, 5.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(
                        trans.translation() + Vec3::new(0.0, 0.0, 0.1),
                    ),
                    ..default() // does this clone twice?
                })
                .insert(tower.bullet.clone().update_target(target));
        }
    }
}

fn move_bullets(mut q_bullets: Query<(&mut Transform, &Bullet)>, time: Res<Time>) {
    for (mut trans, bullet) in q_bullets.iter_mut() {
        match bullet.movement.target {
            Target::None => todo!(),
            Target::Point(p) => {
                if let Some(p) = p {
                    // dir goes to 0 at the point
                    let dir = p - trans.translation;
                    trans.translation +=
                        dir.normalize_or_zero() * time.delta_seconds() * bullet.movement.speed;
                }
            }
            Target::Follow(_) => todo!(),
            Target::Direction(d) => {
                if let Some(d) = d {
                    trans.translation +=
                        d.normalize_or_zero() * time.delta_seconds() * bullet.movement.speed;
                }
            }
        }
    }
}
