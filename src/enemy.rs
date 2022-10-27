use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    castle::Castle,
    tower::{Movement, Target},
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movement).add_system(tick_enemy);
    }
}

#[derive(Component)]
pub struct Enemy {
    pub health: u32,
}

impl Enemy {
    fn new() -> Self {
        Enemy { health: 5 }
    }

    pub fn take_damage(&mut self, damage: u32) {
        if damage > self.health {
            self.health = 0;
        } else {
            self.health -= damage;
        }
    }
}

pub fn spawn_enemy(commands: &mut Commands, num: u32) {
    for i in 0..num {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::CRIMSON,
                    custom_size: Some(Vec2::splat(20.0)),
                    ..default()
                },
                transform: Transform::from_xyz(400.0, 25.0 * i as f32, 0.2),
                ..default()
            })
            .insert(Enemy::new())
            .insert(Movement {
                target: Target::Point(Some(Vec3::ZERO)),
                speed: 50.0,
            })
            .insert(Collider::cuboid(10.0, 10.0))
            .insert(Sensor)
            .insert(RigidBody::Dynamic);
    }
}

fn movement(mut q_enemies: Query<(&mut Transform, &Movement, &Enemy)>, time: Res<Time>) {
    for (mut trans, movement, _enemy) in q_enemies.iter_mut() {
        match movement.target {
            Target::None => todo!(),
            Target::Point(p) => {
                if let Some(p) = p {
                    let dir = p - trans.translation;
                    trans.translation +=
                        dir.normalize_or_zero() * time.delta_seconds() * movement.speed;
                }
            }
            Target::Follow(_) => todo!(),
            Target::Direction(_) => todo!(),
        }
    }
}

fn tick_enemy(
    mut commands: Commands,
    q_enemies: Query<(Entity, &Enemy)>,
    mut q_castle: Query<&mut Castle>,
) {
    for (entity, enemy) in q_enemies.iter() {
        if enemy.health == 0 {
            commands.entity(entity).despawn_recursive();
            for mut castle in q_castle.iter_mut() {
                // enemies die and drop 1 money
                castle.money += 1;
            }
        }
    }
}
