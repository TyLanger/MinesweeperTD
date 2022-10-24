use bevy::prelude::*;

pub struct CastlePlugin;

impl Plugin for CastlePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerritoryInfo::new())
            .add_event::<NumberFilledEvent>()
            .add_event::<ExpandAreaEvent>()
            .add_system(startup)
            .add_system(number_filled);
    }
}

// resource
pub struct TerritoryInfo {
    pub radius: i32,
    pub x: usize,
    pub y: usize,
    pub bombs_percent: f32,
}

impl TerritoryInfo {
    fn new() -> Self {
        TerritoryInfo {
            radius: 1,
            x: 10,
            y: 10,
            bombs_percent: 0.5,
        }
    }
}

// events
pub struct NumberFilledEvent;
pub struct ExpandAreaEvent;

fn startup(mut ev_expand: EventWriter<ExpandAreaEvent>, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::F) {
        ev_expand.send(ExpandAreaEvent);
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
