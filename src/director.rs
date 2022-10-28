use bevy::prelude::*;
use rand::prelude::*;

use crate::{castle::ExpandAreaEvent, enemy::spawn_enemy, loading::SpriteAssets, GameState};

pub struct DirectorPlugin;

impl Plugin for DirectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateDirectorUiEvent>()
            .insert_resource(SpawnInfo::new())
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(spawn_tick)
                    .with_system(upgrade_director)
                    .with_system(update_dirctor_ui),
            );
        // app.insert_resource(SpawnInfo::new()).add_system(spawn_tick);
    }
}

pub struct SpawnInfo {
    duration: f32,
    pub wave_timer: Timer,
    pub batch_size: u32,
    pub difficulty: u32,
    pub enemy_health: u32,
    pub next_strat: SpawnStrat,
    pub positions: Vec<Vec2>,
}

impl SpawnInfo {
    fn new() -> Self {
        SpawnInfo {
            duration: 10.0,
            wave_timer: Timer::from_seconds(10.0, true),
            batch_size: 5,
            difficulty: 0,
            enemy_health: 5,
            next_strat: SpawnStrat::Spread,
            positions: get_spread_positions(10),
        }
    }

    pub fn get_time(&self) -> f32 {
        self.duration - self.wave_timer.elapsed_secs()
    }
}

#[derive(Debug)]
pub enum SpawnStrat {
    Burst,
    Spread,
    Line,
    Pincer,
}

fn spawn_tick(
    mut commands: Commands,
    mut spawn_info: ResMut<SpawnInfo>,
    time: Res<Time>,
    mut ev_update: EventWriter<UpdateDirectorUiEvent>,
    textures: Res<SpriteAssets>,
) {
    // spawn_info.time_elapsed += time.delta_seconds();
    if spawn_info.wave_timer.tick(time.delta()).just_finished() {
        // spawn using old positions
        let points = &spawn_info.positions;
        println!("points len: {:?}", points.len());
        for p in points {
            spawn_enemy(&mut commands, p.extend(0.4), spawn_info.enemy_health, &textures);
        }

        // gen next positions and SpawnStrat
        // gives time to place the info on screen
        let mut rng = rand::thread_rng();
        let spawn_r = rng.gen_range(0..4);
        // increase every spawn
        spawn_info.batch_size += spawn_info.difficulty;
        let num = spawn_info.batch_size;
        spawn_info.enemy_health = 5 + num / 10;
        println!("health is {:}", spawn_info.enemy_health);
        // println!("Spawn {} enemies", num);
        match spawn_r {
            0 => {
                // println!("Next is Burst");
                spawn_info.next_strat = SpawnStrat::Burst;
                spawn_info.positions = get_burst_positions(num);
            }
            1 => {
                // println!("Next is Spread");
                spawn_info.next_strat = SpawnStrat::Spread;
                spawn_info.positions = get_spread_positions(num);
            }
            2 => {
                // println!("Next is Line");
                spawn_info.next_strat = SpawnStrat::Line;
                spawn_info.positions = get_line_positions(num);
            }
            _ => {
                // println!("Next is Pincer");
                spawn_info.next_strat = SpawnStrat::Pincer;
                spawn_info.positions = get_pincer_positions(num);
            }
        };
        ev_update.send(UpdateDirectorUiEvent);
    }
}

fn upgrade_director(
    mut ev_expand: EventReader<ExpandAreaEvent>,
    mut spawn_info: ResMut<SpawnInfo>,
) {
    for _ev in ev_expand.iter() {
        // will run 6 times.
        // runs once when you press play
        // once it runs the 6th time, that's the boss round. Survive and you win.
        println!("Enemies are harder!");
        spawn_info.difficulty += 1;
        spawn_info.duration = 10.0 - (spawn_info.difficulty / 2) as f32;
        spawn_info.wave_timer = Timer::from_seconds(spawn_info.duration, true);
        if spawn_info.difficulty >= 6 {
            println!("Difficulty {}. You win!", spawn_info.difficulty);
        }
    }
}

struct UpdateDirectorUiEvent;

fn update_dirctor_ui(ev_update: EventReader<UpdateDirectorUiEvent>, spawn_info: Res<SpawnInfo>) {
    if !ev_update.is_empty() {
        ev_update.clear();
        println!("Spawn Info changed. batch_size: {}", spawn_info.batch_size);
    }
}

fn get_spread_positions(num: u32) -> Vec<Vec2> {
    let mut v = Vec::new();
    let mut rng = rand::thread_rng();
    for _ in 0..num {
        let spawn_pos = Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0))
            .normalize_or_zero()
            * 500.;
        v.push(spawn_pos);
    }
    v
}

fn get_burst_positions(num: u32) -> Vec<Vec2> {
    let mut v = Vec::new();

    let mut rng = rand::thread_rng();
    let spawn_pos =
        Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0)).normalize_or_zero() * 500.;

    for _ in 0..num {
        let offset = Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0)) * 50.0;
        v.push(spawn_pos + offset);
    }
    v
}

fn get_pincer_positions(num: u32) -> Vec<Vec2> {
    let mut v = Vec::new();
    let mut rng = rand::thread_rng();

    let spawn_pos =
        Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0)).normalize_or_zero() * 500.;
    // opposite side
    let other_spawn = -spawn_pos; // Vec2::new(-spawn_pos.x, -spawn_pos.y);
    let half_num = num / 2;
    let other_num = num - half_num;
    for _ in 0..half_num {
        let offset = Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0)) * 50.0;
        v.push(spawn_pos + offset);
    }
    for _ in 0..other_num {
        let offset = Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0)) * 50.0;
        v.push(other_spawn + offset);
    }
    v
}

fn get_line_positions(num: u32) -> Vec<Vec2> {
    let mut v = Vec::new();
    let mut rng = rand::thread_rng();

    let spawn_pos =
        Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0)).normalize_or_zero() * 500.;
    let dir = (spawn_pos - Vec2::ZERO).normalize_or_zero();
    for i in 0..num {
        let offset = dir * 30.0 * i as f32;
        v.push(spawn_pos + offset);
    }

    v
}
