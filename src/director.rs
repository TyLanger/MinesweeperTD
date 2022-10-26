use bevy::prelude::*;

use crate::enemy::spawn_enemy;

pub struct DirectorPlugin;

impl Plugin for DirectorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnInfo::new()).add_system(spawn_tick);
    }
}

struct SpawnInfo {
    wave_timer: Timer,
    batch_size: u32,
}

impl SpawnInfo {
    fn new() -> Self {
        SpawnInfo {
            wave_timer: Timer::from_seconds(10.0, true),
            batch_size: 10,
        }
    }
}

fn spawn_tick(mut commands: Commands, mut spawn_info: ResMut<SpawnInfo>, time: Res<Time>) {
    if spawn_info.wave_timer.tick(time.delta()).just_finished() {
        spawn_enemy(&mut commands, spawn_info.batch_size);
    }
}
