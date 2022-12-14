use crate::{
    castle::Castle,
    enemy::Enemy,
    grid::{clear_selection, ClearSelectionsEvent, Grid, Selection, Tile, TileState},
    loading::SpriteAssets,
    ui::{update_buttons, ButtonPressEvent},
    utility, GameState,
};
use bevy::utils::Duration;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle, utils::FloatOrd};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TowerPlacedEvent>()
            .insert_resource(TowerServer { towers: Vec::new() });
        app.add_system_set(SystemSet::on_exit(GameState::Loading).with_system(setup_towers));
        // the timing on spawning a tower is too complicated
        // need to press the button to send the event
        // then run this to process the event
        // before clear_selection runs and removes the Selection component
        // it removes the Selection because clicking away from a tile clears selection
        // which includes clicking on a button.
        app.add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(spawn_tower.after(update_buttons).before(clear_selection))
                .with_system(update_tower_position.before(tower_tick))
                .with_system(tower_tick)
                .with_system(move_bullets)
                .with_system(bullet_collision.after(move_bullets))
                .with_system(bullet_tick.after(bullet_collision))
                .with_system(bomb_tick)
                .with_system(explosion_damage)
                .with_system(update_tower_area_indicator)
                .with_system(swarm_init)
                .with_system(swarm_tick),
        );
        // app.add_system(spawn_tower.after(update_buttons).before(clear_selection))
        //     .add_system(update_tower_position.before(tower_tick))
        //     .add_system(tower_tick)
        //     .add_system(move_bullets)
        //     .add_system(bullet_collision.after(move_bullets))
        //     .add_system(bullet_tick.after(bullet_collision))
        //     .add_system(bomb_tick)
        //     .add_system(explosion_damage)
        //     .add_system(update_tower_area_indicator)
        //     .add_system(swarm_init)
        //     .add_system(swarm_tick);
    }
}

pub struct TowerPlacedEvent {
    pub x: usize,
    pub y: usize,
    // type of tower?
}

#[derive(Component, Clone)]
pub struct Tower {
    pub range: f32,
    pub visuals: TowerVisuals,
    pub cost: u32,
    bullet: Bullet,
    gun: Gun,
    position: Option<Vec2>,
}

impl Tower {
    fn shoot(&mut self, commands: &mut Commands, target: Target) {
        self.gun.state = ShootState::BetweenShots;

        match &self.gun.multi_type {
            MultiShotType::Spread(spread) => {
                // convert shoot types to direction
                let front_dir = match target {
                    Target::None => Vec2::ZERO - self.position.unwrap(),
                    Target::Point(point) => point.unwrap().truncate() - self.position.unwrap(),
                    // Target::Follow(_) => Vec2::ZERO - self.position.unwrap(),
                    Target::Direction(dir) => dir.unwrap().truncate(),
                };

                let num = spread.num_shots;
                let angle = spread.spread_angle_deg;
                //let front_dir = mouse.0.extend(0.0) - trans.translation();
                let spread_angle_rad = angle * 0.0174533;
                let spread_half_angle = spread_angle_rad / 2.0;
                // let angle_growth = spread_angle / (num as f32);
                // let rotation = Vec2::new(spread_half_angle.cos(), spread_half_angle.sin());
                // let left_dir = rotation.rotate(front_dir.truncate());
                // let spread_target = Target::Direction(Some(left_dir.extend(0.0)));

                let left_dir = Vec2::from_angle(-spread_half_angle).rotate(front_dir);
                let right_dir = Vec2::from_angle(spread_half_angle).rotate(front_dir);
                for i in 0..num {
                    // 1
                    // 0.5
                    // 2
                    // 0/1, 1/1
                    // 3
                    // 0/2, 1/2, 2/2
                    // 4
                    // 0/3, 1/3, 2/3, 3/3
                    let t = if num > 1 {
                        i as f32 / (num - 1) as f32
                    } else {
                        // if only 1 shot, shoot straight instead of to the left
                        0.5
                    };

                    //let t = i as f32 / f32::max(1.0, (num - 1) as f32);
                    let dir = left_dir.lerp(right_dir, t);
                    // force into direction mode
                    let spread_target = Target::Direction(Some(dir.extend(0.0)));
                    commands
                        .spawn_bundle(SpriteBundle {
                            texture: self.visuals.texture.clone(),
                            sprite: Sprite {
                                // color: Color::BLACK,
                                custom_size: Some(Vec2::splat(16.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(
                                self.position.unwrap().extend(0.2),
                            ),
                            ..default() // does this clone twice?
                        })
                        .insert(self.bullet.clone().update_target(spread_target))
                        .insert(Collider::ball(5.0))
                        .insert(RigidBody::Dynamic)
                        .insert(Sensor);
                }
            }
            MultiShotType::Burst(_num) => {
                // one fire event makes multiple bullets over time
                // how?
                commands
                    .spawn_bundle(SpriteBundle {
                        texture: self.visuals.texture.clone(),

                        sprite: Sprite {
                            // color: Color::BLACK,
                            custom_size: Some(Vec2::splat(16.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(
                            self.position.unwrap().extend(0.2),
                            // trans.translation() + Vec3::new(0.0, 0.0, 0.1),
                        ),
                        ..default() // does this clone twice?
                    })
                    .insert(self.bullet.clone().update_target(target))
                    .insert(Collider::ball(5.0))
                    .insert(RigidBody::Dynamic)
                    .insert(Sensor);
            }
            MultiShotType::Bomb => {
                let dir = match target {
                    Target::None => todo!(),
                    Target::Point(p) => {
                        let p = p.unwrap();
                        let dir_to_center = Vec3::ZERO - p;
                        // enemies moving towards center
                        // movement is 50
                        // bomb takes 1s to travel
                        // in 1s, they will be 50.0 closer to the center so aim there
                        let prediction = p + dir_to_center.normalize_or_zero() * 50.0;
                        prediction.truncate() - self.position.unwrap()
                    }
                    // Target::Follow(_) => todo!(),
                    Target::Direction(d) => d.unwrap().truncate(),
                };
                let mag = dir.length();

                let start_dir = dir.lerp(Vec2::Y * mag * 5.0, 0.7);
                let end_dir = dir - start_dir;

                let start_pos = self.position.unwrap();

                commands
                    .spawn_bundle(SpriteBundle {
                        texture: self.visuals.texture.clone(),

                        sprite: Sprite {
                            // color: Color::BLACK,
                            custom_size: Some(Vec2::splat(16.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(
                            self.position.unwrap().extend(0.2),
                            // trans.translation() + Vec3::new(0.0, 0.0, 0.1),
                        ),
                        ..default() // does this clone twice?
                    })
                    .insert(BombComponent {
                        start_pos,
                        start_dir,
                        end_dir,
                        timer: Timer::from_seconds(1.0, false),
                    });
            }
            MultiShotType::Swarm(s) => {
                let mut s = s.clone();
                s.target = target;
                s.bullet = Some(self.bullet.clone());
                commands
                    .spawn_bundle(SpatialBundle {
                        transform: Transform::from_translation(self.position.unwrap().extend(0.0)),
                        ..default()
                    })
                    .insert(s)
                    .insert(Collider::ball(self.range))
                    .insert(Sensor);
            }
        }
    }

    fn set_position(&mut self, position: Vec2) -> &Self {
        self.position = Some(position);
        self
    }

    fn get_target(&self) -> Target {
        self.bullet.movement.target
    }
}

#[derive(Clone)]
pub struct TowerVisuals {
    pub texture: Handle<Image>,
    pub name: String,
    pub color: Color,
    pub cost: u32,
}

#[derive(Component, Clone)]
struct Bullet {
    // impact_type: ImpactType,
    damage: u32,
    movement: Movement,
    lifetime: Timer,
}

impl Bullet {
    fn new(damage: u32, movement: Movement) -> Self {
        Bullet {
            // impact_type,
            damage,
            movement,
            lifetime: Timer::from_seconds(2.0, false),
        }
    }

    fn update_target(&mut self, target: Target) -> Self {
        self.movement.target = target;
        self.clone()
    }
}

// #[derive(Clone)]
// enum ImpactType {
//     // number of pierces
//     Pierce(usize),
//     // radius of the explosion
//     Explosion(f32),
// }

#[derive(Clone, Component)]
pub struct Movement {
    pub target: Target,
    pub speed: f32,
    // turn_radius: f32 or turn_angle??
}

#[derive(Default, Clone, Copy)]
pub enum Target {
    #[default]
    None,
    Point(Option<Vec3>),
    // Follow(Option<Entity>),
    Direction(Option<Vec3>),
}

#[derive(Clone)]
struct Gun {
    // clip_size: u32,
    // // timer or float?
    // time_between_shots: f32,
    timer_between: Timer,
    // reload_time: f32,
    // reload_timer: Timer,
    // state?
    // shooting, reloading, rest?
    multi_type: MultiShotType,
    state: ShootState,
}

impl Gun {
    fn new(
        // clip_size: u32,
        time_between_shots: f32,
        // reload_time: f32,
        multi_type: MultiShotType,
    ) -> Self {
        // don't make a 0.0s timer. Will crash
        Gun {
            // clip_size,
            // time_between_shots,
            timer_between: Timer::from_seconds(time_between_shots, true),
            // reload_time,
            // reload_timer: Timer::from_seconds(reload_time, false),
            multi_type,
            state: ShootState::Ready,
        }
    }

    #[allow(clippy::collapsible_if)]
    fn tick(&mut self, delta: Duration) -> &Self {
        // ignore collapsable if
        // tick does work I don't want done if not in the right state
        // #![allow(clippy::too_many_arguments, clippy::type_complexity)]

        if self.state == ShootState::BetweenShots {
            if self.timer_between.tick(delta).just_finished() {
                self.state = ShootState::Ready;
            }
        }
        // match self.state {
        //     ShootState::BetweenShots => {
        //         if self.timer_between.tick(delta).just_finished() {
        //             self.state = ShootState::Ready;
        //         }
        //     }
        //     _ => {}
        // }
        self
    }
}

// ShotType
// burst vs spread
#[derive(Clone)]
enum MultiShotType {
    Spread(Spread),
    Burst(u32),
    Bomb,
    Swarm(Swarm),
}

#[derive(Clone, Component)]
pub struct Swarm {
    // num: u32,
    timer: Timer,
    bullet: Option<Bullet>,
    target: Target,
    entities: Vec<Entity>,
}

#[derive(Component)]
struct SwarmComponent {
    start_pos: Vec3,
    end_pos: Vec3,
}

#[derive(Component)]
struct BombComponent {
    start_pos: Vec2,
    start_dir: Vec2,
    end_dir: Vec2,
    timer: Timer,
}

#[derive(Clone, Default, PartialEq)]
enum ShootState {
    #[default]
    Ready,
    BetweenShots,
}

impl Default for MultiShotType {
    fn default() -> Self {
        MultiShotType::Spread(Spread {
            num_shots: 2,
            spread_angle_deg: 20.0,
        })
    }
}

#[derive(Clone, Copy)]
struct Spread {
    num_shots: u32,
    spread_angle_deg: f32,
}

impl Default for Spread {
    fn default() -> Self {
        Spread {
            num_shots: 2,
            spread_angle_deg: 20.0,
        }
    }
}

pub struct TowerServer {
    pub towers: Vec<Tower>,
}

// set up the different towers you can spawn here.
pub fn setup_towers(mut tower_server: ResMut<TowerServer>, textures: Res<SpriteAssets>) {
    let basic_tower = Tower {
        range: 80.0,
        cost: 10,
        visuals: TowerVisuals {
            texture: textures.pistol.clone(),
            name: "Basic".to_string(),
            color: Color::GREEN,
            cost: 10,
        },
        bullet: Bullet::new(
            // ImpactType::Pierce(0),
            1,
            Movement {
                // should I even set a default?
                // when the bullet is spawned, it should change this
                // target: Target::Point(Vec3::ZERO),
                target: Target::Direction(None),
                speed: 100.0,
            },
        ),
        gun: Gun {
            // clip_size: 1,
            // time_between_shots: 0.3,
            timer_between: Timer::from_seconds(0.3, true),
            // reload_time: 1.5,
            // reload_timer: Timer::from_seconds(1.5, false),
            multi_type: MultiShotType::Burst(1),
            state: ShootState::Ready,
        },
        position: None,
    };
    tower_server.towers.push(basic_tower);

    let shotgun_tower = Tower {
        range: 80.0,
        cost: 15,
        visuals: TowerVisuals {
            texture: textures.shotgun.clone(),
            name: "Shotgun".to_string(),
            color: Color::RED,
            cost: 10,
        },
        bullet: Bullet::new(
            // ImpactType::Pierce(0),
            1,
            Movement {
                target: Target::Direction(None),
                speed: 100.0,
            },
        ),
        gun: Gun {
            // clip_size: 2,
            // time_between_shots: 0.5,
            timer_between: Timer::from_seconds(0.5, true),
            // reload_time: 1.5,
            // reload_timer: Timer::from_seconds(1.5, false),
            multi_type: MultiShotType::Spread(Spread {
                num_shots: 3,
                spread_angle_deg: 15.0,
            }),
            state: ShootState::Ready,
        },
        position: None,
    };
    tower_server.towers.push(shotgun_tower);

    let bomb_tower = Tower {
        range: 100.0,
        cost: 30,
        visuals: TowerVisuals {
            texture: textures.bomb.clone(),
            name: "Bomb".to_string(),
            color: Color::ORANGE,
            cost: 20,
        },
        bullet: Bullet::new(
            // ImpactType::Explosion(20.0),
            3,
            Movement {
                target: Target::Point(None),
                speed: 100.0,
            },
        ),
        gun: Gun::new(1.5, MultiShotType::Bomb),
        position: None,
    };
    tower_server.towers.push(bomb_tower);

    let swarm_tower = Tower {
        range: 150.0,
        cost: 20,
        visuals: TowerVisuals {
            texture: textures.magic.clone(),
            name: "Swarm".to_string(),
            color: Color::PURPLE,
            cost: 20,
        },
        bullet: Bullet::new(
            // ImpactType::Pierce(3),
            2,
            Movement {
                target: Target::Point(None),
                speed: 100.0,
            },
        ),
        gun: Gun::new(
            // 4,
            1.0,
            // 2.0,
            MultiShotType::Swarm(Swarm {
                // num: 4,
                timer: Timer::from_seconds(0.8, false),
                bullet: None,
                target: Target::Point(None),
                entities: Vec::new(),
            }),
            // MultiShotType::Spread(Spread {
            //     num_shots: 4,
            //     spread_angle_deg: 20.0,
            // }),
        ),
        position: None,
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
    tower_server: Res<TowerServer>,
    mut q_castle: Query<&mut Castle>,
) {
    for mut castle in q_castle.iter_mut() {
        for ev in ev_button_press.iter() {
            ev_clear_selection.send(ClearSelectionsEvent);
            let tower = tower_server.towers.get(ev.button_number).unwrap();
            //for tower in tower_server.towers.iter() {
            for (ent, mut tile) in q_selection.iter_mut() {
                let mut floor_nearby = false;
                for neighbour in grid.get_ring(tile.x, tile.y, 1).into_iter().flatten() {
                    // if let Some(info) = neighbour {
                    if let Ok(tile) = q_tiles.get(neighbour.entity) {
                        if tile.tile_state == TileState::Floor {
                            floor_nearby = true;
                            break;
                        }
                    }
                    // }
                }
                if !floor_nearby {
                    println!("Tower failed. No floor nearby {}, {}", tile.x, tile.y);
                    continue;
                }

                if tower.cost > castle.money {
                    println!("Tower failed. Too expensive");
                    continue;
                }

                let result = tile.try_spawn_tower();
                match result {
                    Ok(_) => {
                        // money -= tower.cost;
                        castle.money -= tower.cost;
                        let child = commands
                            .spawn_bundle(SpriteBundle {
                                texture: tower.visuals.texture.clone(),
                                // sprite: Sprite {
                                //     color: tower.visuals.color,
                                //     custom_size: Some(Vec2::new(15.0, 15.0)),
                                //     ..default()
                                // },
                                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                                ..default()
                            })
                            .insert(tower.clone())
                            .insert(Collider::ball(tower.range))
                            .insert(Sensor)
                            .with_children(|parent| {
                                let shape = shapes::Circle {
                                    radius: tower.range,
                                    center: Vec2::ZERO,
                                };
                                let mut bundle = GeometryBuilder::build_as(
                                    &shape,
                                    DrawMode::Stroke(StrokeMode::new(
                                        Color::rgba_u8(0x00, 0x99, 0xdb, 0x77),
                                        3.0,
                                    )),
                                    Transform::from_xyz(0.0, 0.0, 0.5),
                                );
                                // not visible by default
                                bundle.visibility.is_visible = false;
                                parent.spawn_bundle(bundle).insert(TowerRangeIndicator);
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
                        println!("Failed to spawn, {:?}", e);
                    }
                }
            }
            //}
        }
    }
}

fn update_tower_position(mut q_towers: Query<(&mut Tower, &GlobalTransform), Added<Tower>>) {
    for (mut tower, trans) in q_towers.iter_mut() {
        tower.set_position(trans.translation().truncate());
    }
}

#[derive(Component)]
struct TowerRangeIndicator;

fn update_tower_area_indicator(
    q_tiles: Query<&Selection, With<Tile>>,
    q_towers: Query<&Parent, With<Tower>>,
    mut q_indicators: Query<(&mut Visibility, &Parent), With<TowerRangeIndicator>>,
) {
    // parent structure is 3 deep
    // Tile.Tower.Indicator
    // set visible if it has a selection
    for (mut vis, parent) in q_indicators.iter_mut() {
        if let Ok(parent) = q_towers.get(parent.get()) {
            if q_tiles.contains(parent.get()) {
                vis.is_visible = true;
            } else {
                vis.is_visible = false;
            }
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
            // Target::Follow(_) => todo!(),
            Target::Direction(d) => {
                if let Some(d) = d {
                    trans.translation +=
                        d.normalize_or_zero() * time.delta_seconds() * bullet.movement.speed;
                }
            }
        }
        trans.rotate_local_z(-3.0 * time.delta_seconds());
    }
}

fn bullet_tick(
    mut commands: Commands,
    mut q_bullets: Query<(Entity, &mut Bullet)>,
    time: Res<Time>,
) {
    for (entity, mut bullet) in q_bullets.iter_mut() {
        if bullet.lifetime.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn bullet_collision(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    q_bullets: Query<(Entity, &Bullet)>,
    mut q_enemies: Query<(Entity, &mut Enemy)>,
) {
    for (bullet_ent, bullet) in q_bullets.iter() {
        let collisions = rapier_context.intersections_with(bullet_ent);
        // let enemies_hit = collisions
        //     .map(|(a, b, inter)| if a == bullet_ent {b} else {a})
        //     .filter(|e| q_enemies.contains(*e))
        //     .map(|e| {
        //         let (entity, mut enemy) = q_enemies.get_mut(e).unwrap();
        //         enemy.take_damage(bullet.damage);
        //         // (entity, enemy)
        //     });

        // for ent in enemies_hit {
        //     if let Ok((e_ent, mut enemy)) = q_enemies.get_mut(ent) {
        //         enemy.take_damage(bullet.damage);
        //     }
        // }
        let mut destroy_bullet = false;
        for (a, b, _) in collisions {
            let enemy_ent = if a == bullet_ent { b } else { a };

            if let Ok((_e_ent, mut enemy)) = q_enemies.get_mut(enemy_ent) {
                enemy.take_damage(bullet.damage);
                destroy_bullet = true;
            }
        }
        if destroy_bullet {
            commands.entity(bullet_ent).despawn_recursive();
        }
    }
}

fn bomb_tick(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut q_bombs: Query<(Entity, &mut Transform, &mut BombComponent)>,
    time: Res<Time>,
) {
    for (ent, mut trans, mut bomb) in q_bombs.iter_mut() {
        if bomb.timer.tick(time.delta()).just_finished() {
            commands
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(30.0).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::ORANGE)),
                    transform: Transform::from_translation(trans.translation),
                    ..default()
                })
                .insert(Collider::ball(30.0))
                .insert(Sensor)
                .insert(Explosion::new());
            commands.entity(ent).despawn_recursive();
        }

        let t = bomb.timer.percent();

        let start_lerp = Vec2::lerp(Vec2::ZERO, bomb.start_dir, t);
        let end_lerp = Vec2::lerp(Vec2::ZERO, bomb.end_dir, t);

        let pos = Vec2::lerp(start_lerp, start_lerp + end_lerp, t);
        trans.translation = (bomb.start_pos + pos).extend(0.3);
        trans.rotate_local_z(-3.0 * time.delta_seconds());
    }
}

#[derive(Component)]
struct Explosion {
    damage: u32,
    danger_timer: Timer,
    lifetime_timer: Timer,
}

impl Explosion {
    fn new() -> Self {
        Explosion {
            damage: 4,
            danger_timer: Timer::from_seconds(0.02, false),
            lifetime_timer: Timer::from_seconds(0.3, false),
        }
    }
}

fn explosion_damage(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut q_bombs: Query<(Entity, &mut Explosion)>,
    mut q_enemies: Query<(Entity, &mut Enemy)>,
    time: Res<Time>,
) {
    for (bomb_ent, mut bomb) in q_bombs.iter_mut() {
        for (enemy_ent, mut enemy) in q_enemies.iter_mut() {
            if rapier_context.intersection_pair(bomb_ent, enemy_ent) == Some(true) {
                enemy.take_damage(bomb.damage);
            }
        }
        // remove the art after 0.3s
        if bomb.lifetime_timer.tick(time.delta()).just_finished() {
            commands.entity(bomb_ent).despawn_recursive();
        }
        // remove the danger after 0.15s
        // that's way op. It does damage every frame then
        // after 0.02s. 1 frame is 0.016 so that should give 2 frames?
        if bomb.danger_timer.tick(time.delta()).just_finished() {
            commands.entity(bomb_ent).remove::<Collider>();
        }
    }
}

fn swarm_init(
    mut commands: Commands,
    mut q_swarm: Query<(&Transform, &mut Swarm), Added<Swarm>>,
    textures: Res<SpriteAssets>,
) {
    for (trans, mut swarm) in q_swarm.iter_mut() {
        let t = swarm.target;
        let p = match t {
            Target::None => todo!(),
            Target::Point(p) => p.unwrap(),
            // Target::Follow(_) => todo!(),
            Target::Direction(_) => todo!(),
        };

        let spawn_dir = (p - trans.translation).truncate();

        let spread_angle_rad = 40.0 * 0.0174533;
        let spread_half_angle = spread_angle_rad / 2.0;

        let left_dir = Vec2::from_angle(-spread_half_angle).rotate(spawn_dir);
        let right_dir = Vec2::from_angle(spread_half_angle).rotate(spawn_dir);

        let far_left_dir = Vec2::from_angle(-2.0 * spread_half_angle).rotate(spawn_dir);
        let far_right_dir = Vec2::from_angle(2.0 * spread_half_angle).rotate(spawn_dir);

        let v = vec![far_left_dir, left_dir, right_dir, far_right_dir];

        for dir in v {
            let end_pos = trans.translation + (dir.normalize_or_zero() * 50.0).extend(0.0);

            let e = commands
                .spawn_bundle(SpriteBundle {
                    texture: textures.magic.clone(),

                    sprite: Sprite {
                        // color: Color::PURPLE,
                        custom_size: Some(Vec2::splat(16.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(
                        trans.translation + Vec3::new(0.0, 0.0, 0.2), //.position.unwrap().extend(0.2),
                                                                      // trans.translation() + Vec3::new(0.0, 0.0, 0.1),
                    ),
                    ..default() // does this clone twice?
                })
                .insert(SwarmComponent {
                    start_pos: trans.translation,
                    end_pos,
                })
                .id();

            swarm.entities.push(e);

            // .insert(swarm.bullet.clone().update_target(target))
            //             .insert(Collider::ball(5.0))
            //             .insert(RigidBody::Dynamic)
            //             .insert(Sensor).id();
        }
    }
}

fn swarm_tick(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    q_enemies: Query<(Entity, &Transform), (With<Enemy>, Without<Swarm>, Without<SwarmComponent>)>,
    mut q_swarm: Query<(Entity, &Transform, &mut Swarm), (Without<SwarmComponent>, Without<Enemy>)>,
    mut q_swarm_objs: Query<(&mut Transform, &SwarmComponent), (Without<Enemy>, Without<Swarm>)>,
    time: Res<Time>,
) {
    for (entity, trans, mut swarm) in q_swarm.iter_mut() {
        if swarm.timer.tick(time.delta()).just_finished() {
            // do normal bullet things
            // spawn a normal bullet?
            // add bullet component and remove swarm?
            // need to add collision

            let collisions = rapier_context.intersections_with(entity);
            let close_pair = collisions
                // figure out which is self and which is enemy
                .map(|(a, b, _inter)| if a == entity { b } else { a })
                // filter out enemies from other entities (like bullets)
                .filter(|b| q_enemies.contains(*b))
                // turn 1 entity into (entity, vec3)
                // which is what we want returned
                .map(|b| {
                    let (e, pos) = q_enemies.get(b).unwrap();
                    (e, pos.translation)
                })
                // find the smallest distance
                // many -> one
                .min_by_key(|(_e, pos)| {
                    // let (_ent, trans) = q_enemies.get(*b).unwrap();
                    FloatOrd(trans.translation.distance_squared(*pos))
                });

            // send the target to each swarm spawn
            for &e in swarm.entities.iter() {
                if let Ok((t, s)) = q_swarm_objs.get(e) {
                    // if a target exists, all orbs fire at that point
                    // else just fly off into space
                    let target;
                    if let Some((_e, pos)) = close_pair {
                        let dir_to_center = Vec3::ZERO - pos;
                        // enemies moving towards center
                        // movement is 50
                        // bomb takes 1s to travel
                        // in 1s, they will be 50.0 closer to the center so aim there
                        let prediction = pos + dir_to_center.normalize_or_zero() * 25.0;

                        // bullets all have speed 100.0
                        // enemies all have speed 50.0
                        let inter = utility::get_intercept(
                            t.translation.truncate(),
                            100.0,
                            pos.truncate(),
                            dir_to_center.truncate().normalize_or_zero(),
                            50.0,
                        );
                        if let Some(inter) = inter {
                            target = Target::Direction(Some(inter.heading.extend(0.0)));
                        } else {
                            target = Target::Direction(Some(
                                (prediction - t.translation).normalize_or_zero(),
                            ));
                        }
                    } else {
                        target =
                            Target::Direction(Some((s.end_pos - s.start_pos).normalize_or_zero()));
                    }

                    commands.entity(e).remove::<SwarmComponent>();
                    commands
                        .entity(e)
                        .insert(swarm.bullet.clone().unwrap().update_target(target))
                        .insert(Collider::ball(5.0))
                        .insert(RigidBody::Dynamic)
                        .insert(Sensor);
                }
            }

            // destroy the helper now that the orbs have their own direction
            commands.entity(entity).despawn_recursive();
        } else {
            let t = swarm.timer.percent();

            for &e in swarm.entities.iter() {
                if let Ok((mut ent_trans, swarm_c)) = q_swarm_objs.get_mut(e) {
                    ent_trans.translation = Vec3::lerp(swarm_c.start_pos, swarm_c.end_pos, t);
                }
            }

            // trans.translation = Vec2::lerp(swarm.start_pos, swarm.end_pos, t).extend(0.3);
        }
    }
}

fn tower_tick(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut q_towers: Query<(Entity, &mut Tower, &GlobalTransform)>,
    q_enemies: Query<(Entity, &GlobalTransform), With<Enemy>>,
    time: Res<Time>,
) {
    for (entity, mut tower, tower_trans) in q_towers.iter_mut() {
        if tower.gun.tick(time.delta()).state == ShootState::Ready {
            let collisions = rapier_context.intersections_with(entity);
            // map version
            let close_pair = collisions
                // figure out which is self and which is enemy
                .map(|(a, b, _inter)| if a == entity { b } else { a })
                // filter out enemies from other entities (like bullets)
                .filter(|b| q_enemies.contains(*b))
                // turn 1 entity into (entity, vec3)
                // which is what we want returned
                .map(|b| {
                    let (e, pos) = q_enemies.get(b).unwrap();
                    (e, pos.translation())
                })
                // find the smallest distance
                // many -> one
                .min_by_key(|(_e, pos)| {
                    // let (_ent, trans) = q_enemies.get(*b).unwrap();
                    FloatOrd(tower_trans.translation().distance_squared(*pos))
                });
            // .map(|b| {
            //     let (e, pos) = q_enemies.get(b).unwrap();
            //     (e, pos.translation())
            // });

            // doesn't work for old enemies
            // if a tower is spawned after enemies, it won't shoot.
            // maybe the oldest enemy is a and new is b
            // how do I check for that?
            // entity == a -> use a
            // how do I do that with the map?

            // might be able to simplify map.filter.map
            // they're doing a lot of the same stuff

            // let collisions = rapier_context.intersections_with(entity);
            // // loop version
            // let mut closest_pos = None;
            // let mut closest_ent = None;
            // let mut closest_dist_sq = 100000.0;
            // for (_ent_a, ent_b, _some_bool) in collisions {
            //     // a is self, b is all the enemies
            //     // bool is inter
            //     if let Ok((e_ent, e_trans)) = q_enemies.get(ent_b) {
            //         let dist_sq = tower_trans
            //             .translation()
            //             .distance_squared(e_trans.translation());
            //         if dist_sq < closest_dist_sq {
            //             closest_dist_sq = dist_sq;
            //             closest_pos = Some(e_trans.translation());
            //             closest_ent = Some(e_ent);
            //         }
            //     }
            // }

            // if close_pair.is_some() {
            //     // they're equivalent so I must've done something right
            //     // if close_pair.is_some() {
            //     //     println!("Both some");
            //     //     println!(
            //     //         "loop: pos: {:?}, ent: {:?}. Map: pos: {:?}, ent: {:?}",
            //     //         closest_pos.unwrap(),
            //     //         closest_ent.unwrap(),
            //     //         close_pair.unwrap().1,
            //     //         close_pair.unwrap().0,
            //     //     );
            //     //     if closest_pos.unwrap() == close_pair.unwrap().1 {
            //     //         println!("Both positions same");
            //     //     }
            //     //     if closest_ent.unwrap() == close_pair.unwrap().0 {
            //     //         println!("Both entities same");
            //     //     }
            //     // }
            //     let (_closest_ent, closest_pos) = close_pair.unwrap();

            if let Some((_closest_ent, closest_pos)) = close_pair {
                match tower.get_target() {
                    Target::None => {}
                    Target::Point(_) => {
                        let target = Target::Point(Some(closest_pos));
                        tower.shoot(&mut commands, target);
                    }
                    // Target::Follow(_) => {
                    //     let target = Target::Follow(Some(closest_ent));
                    //     tower.shoot(&mut commands, target);
                    // }
                    Target::Direction(_) => {
                        // enemies moving towards center
                        // movement is 50
                        // bomb takes 1s to travel
                        // in 1s, they will be 50.0 closer to the center so aim there
                        // other bullets are faster. lead by less
                        // if I really wanted good aim, would need to scale it by distance to target
                        let dir_to_center = Vec3::ZERO - closest_pos;

                        // bullets all have speed 100.0
                        // enemies all have speed 50.0
                        let inter = utility::get_intercept(
                            tower_trans.translation().truncate(),
                            100.0,
                            closest_pos.truncate(),
                            dir_to_center.truncate().normalize_or_zero(),
                            50.0,
                        );
                        if let Some(inter) = inter {
                            // accurate predictions
                            let target = Target::Direction(Some(inter.heading.extend(0.0)));
                            tower.shoot(&mut commands, target);
                        } else {
                            let prediction = closest_pos + dir_to_center.normalize_or_zero() * 30.0;

                            let dir = prediction - tower_trans.translation();

                            //let dir = closest_pos - tower_trans.translation();
                            let target = Target::Direction(Some(dir));
                            tower.shoot(&mut commands, target);
                        }
                    }
                }
            }

            // shoot something
            // for (enemy_ent, enemy_trans) in q_enemies.iter() {

            //     // check for enemies in collision
            //     let hit = rapier_context.intersection_pair(entity, enemy_ent);
            //     if let Some(true) = hit {
            //         // bullet.movement.target is a bad spot to have this
            //         // very hard to find.
            //         // match tower.bullet.movement.target {
            //         match tower.get_target() {
            //             Target::None => {},
            //             Target::Point(_) => {
            //                 let target = Target::Point(Some(enemy_trans.translation()));
            //                 tower.shoot(&mut commands, target);
            //             },
            //             Target::Follow(_) => {
            //                 let target = Target::Follow(Some(enemy_ent));
            //                 tower.shoot(&mut commands, target);
            //             },
            //             Target::Direction(_) => {
            //                 let dir = enemy_trans.translation() - tower_trans.translation();
            //                 let target = Target::Direction(Some(dir));
            //                 tower.shoot(&mut commands, target);
            //             },
            //         }
            //     }
            // }
        }
    }
}
