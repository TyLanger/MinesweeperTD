use bevy::{prelude::*, render::texture::ImageSettings};
use std::collections::HashSet;

use crate::{
    castle::{ExpandAreaEvent, NumberFilledEvent, TerritoryInfo},
    tower::TowerPlacedEvent,
    GameState, MouseWorldPos,
};

// palette from https://lospec.com/palette-list/endesga-32
pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid { tiles: Vec::new() })
            .insert_resource(ImageSettings::default_nearest())
            .insert_resource(NumberTextures::default())
            .add_event::<ClearSelectionsEvent>();

        app.add_system_set(SystemSet::on_enter(GameState::Loading).with_system(setup_atlas))
            .add_system_set(
                SystemSet::on_enter(GameState::MainMenu).with_system(setup_grid), //.after(setup_atlas)
            )
            // .add_system_set(
            //     // after setup_atlas
            //     SystemSet::on_enter(GameState::Playing).with_system(setup),
            // )
            // update
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(expand_floor)
                    .with_system(clear_interaction)
                    .with_system(interaction.after(clear_interaction))
                    .with_system(tile_interaction.after(interaction))
                    .with_system(clear_selection.after(tile_interaction))
                    .with_system(update_numbers)
                    .with_system(decrement_numbers),
            );
        // exit
        // .add_system_set(SystemSet::on_exit(GameState::MainMenu).with_system(cleanup_menu));

        // app.insert_resource(Grid { tiles: Vec::new() })
        //     .insert_resource(ImageSettings::default_nearest())
        //     .insert_resource(NumberTextures::default())
        //     .add_event::<ClearSelectionsEvent>()
        //     .add_startup_system(setup_atlas.before(setup))
        //     .add_startup_system(setup)
        //     .add_system(expand_floor)
        //     .add_system(clear_interaction.before(interaction))
        //     .add_system(interaction)
        //     .add_system(tile_interaction.after(interaction))
        //     .add_system(clear_selection.after(tile_interaction))
        //     .add_system(update_numbers)
        //     .add_system(decrement_numbers);
    }
}

const GRID_WIDTH: usize = 21;
const GRID_HEIGHT: usize = 21;
const TILE_SIZE: f32 = 30.0;

// Events
pub struct ClearSelectionsEvent;

#[derive(Default)]
struct NumberTextures {
    handle: Handle<TextureAtlas>,
}

#[derive(Component)]
struct NumberSprite;

#[derive(Component)]
pub struct Tile {
    colour: Color,
    selection: Color,
    hover: Color,
    floor: Color,
    pub tile_state: TileState,
    number: usize,
    pub x: usize,
    pub y: usize,
}

impl Tile {
    fn new(colour: Color, floor: Color, x: usize, y: usize) -> Self {
        Tile {
            colour,
            selection: Color::MIDNIGHT_BLUE,
            hover: Color::ALICE_BLUE,
            tile_state: TileState::Wall,
            floor,
            number: 0,
            x,
            y,
        }
    }

    pub fn try_spawn_tower(&mut self) -> Result<(), PlaceError> {
        // if self.tile_state == TileState::Wall {
        //     self.tile_state = TileState::Tower;
        //     return Ok(());
        // }
        // Err(())
        match self.tile_state {
            TileState::Wall => {
                self.tile_state = TileState::Tower;
                Ok(())
            }
            TileState::Tower => Err(PlaceError::TowerAlready),
            _ => Err(PlaceError::Floor),
        }
    }

    pub fn get_colour(&self) -> Color {
        match self.tile_state {
            TileState::Wall => self.colour,
            TileState::Floor => self.floor,
            TileState::Tower => self.floor,
            _ => Color::ANTIQUE_WHITE,
        }
    }
}

#[derive(Debug)]
pub enum PlaceError {
    TowerAlready,
    Floor,
    //NotEdge,
}

#[allow(dead_code)]
#[derive(PartialEq)]
pub enum TileState {
    Wall,
    Floor,
    Tower,
    Number,
    Castle,
}

#[derive(Copy, Clone)]
pub struct TileInfo {
    pub entity: Entity,
    //tile: Tile,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Coords {
    pub x: i32,
    pub y: i32,
}

impl Coords {
    fn new(x: i32, y: i32) -> Self {
        Coords { x, y }
    }

    fn get_neighbour_coords(self) -> Vec<Coords> {
        let a = Coords::new(self.x, self.y + 1);
        let b = Coords::new(self.x + 1, self.y + 1);
        let c = Coords::new(self.x + 1, self.y);
        let d = Coords::new(self.x + 1, self.y - 1);
        let e = Coords::new(self.x, self.y - 1);
        let f = Coords::new(self.x - 1, self.y - 1);
        let g = Coords::new(self.x - 1, self.y);
        let h = Coords::new(self.x - 1, self.y + 1);

        vec![a, b, c, d, e, f, g, h]
    }

    fn get_ring_coords(self, radius: i32) -> Vec<Coords> {
        let mut v = Vec::new();
        for i in -radius..=radius {
            for j in -radius..=radius {
                if i == -radius || i == radius || j == -radius || j == radius {
                    let x = self.x + i;
                    let y = self.y + j;
                    v.push(Coords::new(x, y));
                }
            }
        }
        v
    }
}

pub struct Grid {
    pub tiles: Vec<TileInfo>,
}

impl Grid {
    pub fn get_vec2(&self, pos: Vec2) -> Option<TileInfo> {
        let x = (((GRID_WIDTH - 1) as f32 * 0.5 * TILE_SIZE) + TILE_SIZE * 0.5 + pos.x) / TILE_SIZE;
        let y = ((GRID_HEIGHT as f32 * 0.5 * TILE_SIZE) + TILE_SIZE * 0.5 + pos.y) / TILE_SIZE;

        if x < 0.0 || y < 0.0 {
            return None;
        }

        self.get_xy(x as usize, y as usize)
    }

    pub fn get_xy(&self, x: usize, y: usize) -> Option<TileInfo> {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            return None;
        }

        // in order (0, 0), (0, 1), (0, 2)
        let index = x * GRID_HEIGHT + y;
        let tile = self.tiles.get(index);
        tile.copied()
        // tile.map(|&t| t)
        // I guess this replaces this:
        // match tile {
        //     Some(&t) => Some(t),
        //     None => None,
        // }
    }

    fn get_coords(&self, coords: Coords) -> Option<TileInfo> {
        self.get_xy(coords.x as usize, coords.y as usize)
    }

    fn get_neighbours(&self, x: usize, y: usize) -> Vec<Option<TileInfo>> {
        let a = self.get_xy(x, y + 1);
        let b = self.get_xy(x + 1, y + 1);
        let c = self.get_xy(x + 1, y);
        let mut d = None;
        let mut e = None;
        let mut f = None;
        let mut g = None;
        let mut h = None;
        if y > 0 {
            d = self.get_xy(x + 1, y - 1);
            e = self.get_xy(x, y - 1);
        }
        if x > 0 && y > 0 {
            f = self.get_xy(x - 1, y - 1);
        }
        if x > 0 {
            g = self.get_xy(x - 1, y);
            h = self.get_xy(x - 1, y + 1);
        }

        vec![a, b, c, d, e, f, g, h]
    }

    #[allow(dead_code)]
    fn get_5x5_ring(&self, x: usize, y: usize) -> Vec<Option<TileInfo>> {
        // (x+2, y), (x-2, y)
        // (x, y+2), (x, y-2)
        // 16 tiles
        // let a = self.get_xy(x-2, y+2);
        let mut v = Vec::new();
        for i in -2..=2 {
            for j in -2..=2 {
                // only do the edges
                if i == -2 || i == 2 || j == -2 || j == 2 {
                    let i2 = x as i32 + i;
                    let j2 = y as i32 + j;

                    let a = if i2 >= 0 && j2 >= 0 {
                        self.get_xy(i2 as usize, j2 as usize)
                    } else {
                        None
                    };

                    v.push(a);
                }
            }
        }
        v
    }

    pub fn get_ring(&self, x: usize, y: usize, radius: i32) -> Vec<Option<TileInfo>> {
        // radius of 0 is self
        // 1 is 3x3
        // 2 is 5x5, etc..
        let mut v = Vec::new();
        for i in -radius..=radius {
            for j in -radius..=radius {
                // only do the edges
                if i == -radius || i == radius || j == -radius || j == radius {
                    let i2 = x as i32 + i;
                    let j2 = y as i32 + j;

                    let a = if i2 >= 0 && j2 >= 0 {
                        self.get_xy(i2 as usize, j2 as usize)
                    } else {
                        None
                    };

                    v.push(a);
                }
            }
        }
        v
    }
}

fn setup_atlas(
    asset_server: Res<AssetServer>,
    mut texture_atlasses: ResMut<Assets<TextureAtlas>>,
    mut numbers: ResMut<NumberTextures>,
) {
    let texture_handle = asset_server.load("sprites/number_strip.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::splat(16.0), 10, 1);
    let texture_atlas_handle = texture_atlasses.add(texture_atlas);
    numbers.handle = texture_atlas_handle;
}

fn setup_grid(mut commands: Commands, mut grid: ResMut<Grid>, numbers: Res<NumberTextures>) {
    let offset = Vec3::new(
        // 21 * 0.5 = 10.5 * 30.0
        -0.5 * ((GRID_WIDTH - 1) as f32) * TILE_SIZE,
        -0.5 * ((GRID_HEIGHT) as f32) * TILE_SIZE,
        0.0,
    );

    for i in 0..GRID_WIDTH {
        for j in 0..GRID_HEIGHT {
            let even = (i + j) % 2 == 0;
            let color = if even {
                //Color::GREEN
                // #b4dc25
                //Color::rgb_u8(0xb4, 0xdc, 0x25)

                // #3e8948
                Color::rgb_u8(0x3e, 0x89, 0x48)
            } else {
                //Color::SEA_GREEN
                // #26a630
                //Color::rgb_u8(0x26, 0xa6, 0x30)
                // #fbd439
                //Color::rgb_u8(0xfb, 0xd4, 0x39)
                // #fbffce
                //Color::rgb_u8(0xfb, 0xff, 0xce)
                // #265c42
                //Color::rgb_u8(0x26, 0x5c, 0x42)
                // #63c74d
                // Color::rgb_u8(0x63, 0xc7, 0x4d)

                // #265c42
                Color::rgb_u8(0x26, 0x5c, 0x42)
            };

            // #e4a672
            // #b86f50
            // #e8b796
            let floor = if even {
                //Color::rgb_u8(0xb8, 0x6f, 0x50)
                Color::rgb_u8(0xe4, 0xa6, 0x72)
            } else {
                //Color::rgb_u8(0xe4, 0xa6, 0x72)
                Color::rgb_u8(0xe8, 0xb7, 0x96)
            };

            let pos = offset + Vec3::new(i as f32 * TILE_SIZE, j as f32 * TILE_SIZE, 0.0);

            let tile = Tile::new(color, floor, i, j);
            let tile_ent = commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_translation(pos),
                    ..default()
                })
                .insert(tile)
                .insert(Interaction::None)
                .with_children(|tile| {
                    tile.spawn_bundle(SpriteSheetBundle {
                        texture_atlas: numbers.handle.clone(),
                        ..default()
                    })
                    .insert(NumberSprite);
                })
                .id();

            grid.tiles.push(TileInfo {
                entity: tile_ent,
                //tile: tile,
            });
        }
    }
}

// with<tile> stops it from messing with UI stuff I might have
pub fn clear_interaction(mut q_tile: Query<(&mut Interaction, Option<&Selection>), With<Tile>>) {
    for (mut interaction, selection) in q_tile.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                if selection.is_none() {
                    *interaction = Interaction::None;
                }
            }
            Interaction::Hovered => {
                *interaction = Interaction::None;
            }
            Interaction::None => {
                // do nothing
            }
        }
    }
}

// with<tile> stops it from messing with UI stuff I might have
pub fn interaction(
    mut q_tile: Query<&mut Interaction, With<Tile>>,
    grid: Res<Grid>,
    mouse: Res<MouseWorldPos>,
    mouse_click: Res<Input<MouseButton>>,
    mut ev_clear: EventWriter<ClearSelectionsEvent>,
) {
    let hovered = grid.get_vec2(mouse.0);
    let left_clicked = mouse_click.just_pressed(MouseButton::Left);
    let right_clicked = mouse_click.just_pressed(MouseButton::Right);
    if let Some(info) = hovered {
        let target = q_tile.get_mut(info.entity);
        if let Ok(mut interaction) = target {
            match *interaction {
                Interaction::Clicked => {
                    // do nothing
                }
                Interaction::Hovered => {
                    // this can't run
                    // clear_interaction runs before this and clears hovered to none
                    if left_clicked {
                        *interaction = Interaction::Clicked;
                    }
                }
                Interaction::None => {
                    *interaction = Interaction::Hovered;
                    if left_clicked {
                        *interaction = Interaction::Clicked;
                    }
                }
            }
        }
    } else if left_clicked {
        // sometimes won't spawn a tower
        // clicking a button sends a tower build event
        // it also triggers this (clicking while not hovering)
        // both call clear_selection
        // need to build before clear_selevtion
        // bug was confusion between clear_interaction and clear_selection
        // I wanted clear_selection, the one that removes the Selection component
        // but also messes up if it goes tower, interaction, clear_selection
        // needs to go clear, this, tower

        // update_buttons, spawn_tower, clear
        // is the correct order
        ev_clear.send(ClearSelectionsEvent);
    }

    if right_clicked {
        ev_clear.send(ClearSelectionsEvent);
    }
}

#[derive(Component)]
pub struct Selection;

fn tile_interaction(
    mut commands: Commands,
    mut q_interaction: Query<(Entity, &Interaction, &mut Sprite, &Tile)>,
) {
    for (entity, interaction, mut sprite, tile) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                sprite.color = tile.selection;
                commands.entity(entity).insert(Selection);
            }
            Interaction::Hovered => {
                sprite.color = tile.hover;
            }
            Interaction::None => {
                sprite.color = tile.get_colour();
            }
        }
    }
}

pub fn clear_selection(
    mut commands: Commands,
    q_selection: Query<Entity, With<Selection>>,
    keyboard: Res<Input<KeyCode>>,
    ev_clear: EventReader<ClearSelectionsEvent>,
) {
    if keyboard.just_pressed(KeyCode::Space) || !ev_clear.is_empty() {
        ev_clear.clear();
        //println!("Clear selection");
        for entity in q_selection.iter() {
            commands.entity(entity).remove::<Selection>();
        }
    }
}

fn expand_floor(
    mut q_tiles: Query<(&mut Tile, &mut Sprite)>,
    grid: Res<Grid>,
    territory_info: Res<TerritoryInfo>,
    ev_expand: EventReader<ExpandAreaEvent>,
) {
    if !ev_expand.is_empty() {
        ev_expand.clear();
        let x = territory_info.x;
        let y = territory_info.y;
        let radius = territory_info.radius;
        let center_coords = Coords::new(x as i32, y as i32);

        if radius == 1 {
            // set center to floor
            let center = grid.get_coords(center_coords);
            if let Some(info) = center {
                if let Ok((mut tile, mut sprite)) = q_tiles.get_mut(info.entity) {
                    tile.tile_state = TileState::Floor;
                    sprite.color = tile.get_colour();
                    tile.number = 0;
                }
            }
        }
        // radius goes up += 2
        if radius > 2 && !territory_info.battlements_style {
            // Brute force
            // for (mut tile, mut _sprite) in q_tiles.iter_mut() {
            //     if tile.tile_state == TileState::Floor {
            //         tile.number = 0;
            //     }
            // }
            // clear current numbers
            let neighbours = grid.get_ring(x, y, radius - 2);
            for info in neighbours.iter().flatten() {
                // if let Some(info) = ent {
                if let Ok((mut tile, mut _sprite)) = q_tiles.get_mut(info.entity) {
                    if tile.tile_state == TileState::Floor {
                        tile.number = 0;
                    }
                }
                // }
            }
            // clear old walls
            let neighbours = grid.get_ring(x, y, radius - 1);
            for info in neighbours.iter().flatten() {
                // if let Some(info) = ent {
                if let Ok((mut tile, mut sprite)) = q_tiles.get_mut(info.entity) {
                    if tile.tile_state == TileState::Wall {
                        tile.tile_state = TileState::Floor;
                        sprite.color = tile.get_colour();
                    }
                }
                // }
            }
        }
        // battlements
        if territory_info.battlements_style {
            // radius starts at 1 for the 3x3
            let mut wall_set: HashSet<Coords> = HashSet::new();
            let mut floor_set: HashSet<Coords> = HashSet::new();
            let inner = center_coords.get_ring_coords(radius);
            let outer = center_coords.get_ring_coords(radius + 1);
            if radius > 2 {
                // clear old
                let mut old_inner = center_coords.get_ring_coords(radius - 2);
                let mut old_outer = center_coords.get_ring_coords(radius - 1);

                // only inner will have a number, but not worth separating for that
                old_inner.append(&mut old_outer);

                // skip towers
                // set walls to floor
                // number to 0
                // color adjustment
                for c in old_inner {
                    if let Some(info) = grid.get_coords(c) {
                        if let Ok((mut tile, mut sprite)) = q_tiles.get_mut(info.entity) {
                            if tile.tile_state != TileState::Tower {
                                tile.number = 0;
                                tile.tile_state = TileState::Floor;
                                sprite.color = tile.get_colour();
                            }
                        }
                    }
                }
            }

            for i in inner {
                if (i.x + i.y) % 2 == 0 {
                    floor_set.insert(i);
                } else {
                    wall_set.insert(i);
                }
            }
            for o in outer {
                wall_set.insert(o);
            }

            let mut random_set = HashSet::new();
            let number_total = wall_set.len() as f32 * territory_info.bombs_percent;

            for (i, c) in wall_set.drain().enumerate() {
                if i > number_total.floor() as usize {
                    break;
                }
                random_set.insert(c);
            }

            // calculate numbers
            for floor in floor_set {
                let mut number = 0;
                let n_coords = floor.get_ring_coords(1);
                for c in n_coords {
                    // is it a bomb?
                    if random_set.contains(&c) {
                        number += 1;
                    }
                }
                // set the number
                // and set to floor
                // and change colour
                if let Some(info) = grid.get_coords(floor) {
                    if let Ok((mut tile, mut sprite)) = q_tiles.get_mut(info.entity) {
                        tile.number = number;
                        tile.tile_state = TileState::Floor;
                        sprite.color = tile.get_colour();
                    }
                }
            }
        } else {
            // set all to floor
            let neighbours = grid.get_ring(x, y, radius);
            for info in neighbours.iter().flatten() {
                // if let Some(info) = ent {
                if let Ok((mut tile, mut sprite)) = q_tiles.get_mut(info.entity) {
                    if tile.tile_state != TileState::Tower {
                        tile.tile_state = TileState::Floor;
                        sprite.color = tile.get_colour();
                    }
                }
                // }
            }

            let mut wall_set: HashSet<Coords> = HashSet::new();
            // get a set of all the walls in neighbours of the neighbours
            let neighbours = center_coords.get_ring_coords(radius);
            for &c in neighbours.iter() {
                if c.x > 0 && c.y > 0 {
                    if let Some(_info) = grid.get_coords(c) {
                        let wall_coords = c.get_neighbour_coords();
                        for &wc in wall_coords.iter() {
                            if let Some(info2) = grid.get_coords(wc) {
                                if let Ok((tile, _sprite)) = q_tiles.get(info2.entity) {
                                    if tile.tile_state == TileState::Wall {
                                        wall_set.insert(Coords::new(tile.x as i32, tile.y as i32));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // could this just be the 5x5? instead of neighbours of neighbours

            let mut random_set = HashSet::new();
            let number_total = wall_set.len() as f32 * territory_info.bombs_percent;
            // pick random coords to have bombs
            //wall_set.remove(value)
            for (i, c) in wall_set.drain().enumerate() {
                if i >= number_total.floor() as usize {
                    break;
                }
                random_set.insert(c);
                println!("Bomb at {:?}", c);
            }
            // coordinates of the bombs
            // for each bomb, check its neighbours
            // if the neighbour is a floor, increment its count
            for c in random_set.drain() {
                let n = c.get_neighbour_coords();
                for c in n {
                    if let Some(info) = grid.get_coords(c) {
                        if let Ok((mut tile, _sprite)) = q_tiles.get_mut(info.entity) {
                            if tile.tile_state == TileState::Floor {
                                let ct = Coords::new(tile.x as i32, tile.y as i32);
                                if ct == c {
                                    tile.number += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn update_numbers(
    q_tiles: Query<&Tile>,
    mut q_tile_numbers: Query<
        (&mut TextureAtlasSprite, &Handle<TextureAtlas>, &Parent),
        With<NumberSprite>,
    >,
) {
    for (mut sprite, _handle, parent) in q_tile_numbers.iter_mut() {
        let tile = q_tiles.get(parent.get()).unwrap();
        sprite.index = tile.number % 9;
    }
}

fn decrement_numbers(
    mut ev_tower_spawned: EventReader<TowerPlacedEvent>,
    mut q_tiles: Query<&mut Tile>,
    grid: Res<Grid>,
    mut ev_number_filled: EventWriter<NumberFilledEvent>,
) {
    for ev in ev_tower_spawned.iter() {
        // check for number around the tower
        let neighbours = grid.get_neighbours(ev.x, ev.y);
        for n in neighbours.iter().flatten() {
            // if let Some(n) = n {
            if let Ok(mut tile) = q_tiles.get_mut(n.entity) {
                if tile.tile_state == TileState::Floor && tile.number > 0 {
                    tile.number -= 1;

                    // only count as filled if the tower being placed
                    // caused this to go to 0
                    // if it started at 0, it's fine
                    if tile.number == 0 {
                        println!(
                            "Number filled at tower: {}, {} Tile: {}, {}",
                            ev.x, ev.y, tile.x, tile.y
                        );
                        ev_number_filled.send(NumberFilledEvent);
                    }
                }
            }
            // }
        }
    }
}
