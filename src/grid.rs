use bevy::{prelude::*, render::texture::ImageSettings};
use std::collections::HashSet;

use crate::MouseWorldPos;

// palette from https://lospec.com/palette-list/endesga-32
pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid { tiles: Vec::new() })
            .insert_resource(ImageSettings::default_nearest())
            .insert_resource(NumberTextures::default())
            .add_event::<ClearSelectionsEvent>()
            .add_startup_system(setup_atlas.before(setup))
            .add_startup_system(setup)
            .add_system(make_floor)
            .add_system(clear_interaction.before(interaction))
            .add_system(interaction)
            .add_system(tile_interaction.after(interaction))
            .add_system(clear_selection.after(tile_interaction))
            .add_system(update_numbers);
    }
}

const GRID_WIDTH: usize = 18;
const GRID_HEIGHT: usize = 14;
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
    tile_state: TileState,
    number: usize,
    x: usize,
    y: usize,
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
            _ => Color::ANTIQUE_WHITE,
        }
    }
}

#[derive(Debug)]
pub enum PlaceError {
    TowerAlready,
    Floor,
}

#[allow(dead_code)]
#[derive(PartialEq)]
enum TileState {
    Wall,
    Floor,
    Tower,
    Number,
    Castle,
}

#[derive(Copy, Clone)]
pub struct TileInfo {
    entity: Entity,
    //tile: Tile,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct Coords {
    x: i32,
    y: i32,
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
}

pub struct Grid {
    pub tiles: Vec<TileInfo>,
}

impl Grid {
    pub fn get_vec2(&self, pos: Vec2) -> Option<TileInfo> {
        let x = ((GRID_WIDTH as f32 * 0.5 * TILE_SIZE) + TILE_SIZE * 0.5 + pos.x) / TILE_SIZE;
        let y = ((GRID_HEIGHT as f32 * 0.5 * TILE_SIZE) + TILE_SIZE * 0.5 + pos.y) / TILE_SIZE;

        if x < 0.0 || y < 0.0 {
            return None;
        }

        self.get_xy(x as usize, y as usize)
    }

    fn get_xy(&self, x: usize, y: usize) -> Option<TileInfo> {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            return None;
        }

        // in order (0, 0), (0, 1), (0, 2)
        let index = x * GRID_HEIGHT + y;
        let tile = self.tiles.get(index);
        //tile.copied()
        tile.map(|&t| t)
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

fn setup(mut commands: Commands, mut grid: ResMut<Grid>, numbers: Res<NumberTextures>) {
    let offset = Vec3::new(
        -0.5 * (GRID_WIDTH as f32) * TILE_SIZE,
        -0.5 * (GRID_HEIGHT as f32) * TILE_SIZE,
        0.0,
    );

    for i in 0..GRID_WIDTH {
        for j in 0..GRID_HEIGHT {
            let even = (i + j) % 2 == 0;
            let color = if even {
                //Color::GREEN
                // #b4dc25
                //Color::rgb_u8(0xb4, 0xdc, 0x25)

                // #265c42
                Color::rgb_u8(0x26, 0x5c, 0x42)
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

                // #3e8948
                Color::rgb_u8(0x3e, 0x89, 0x48)
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
fn clear_interaction(mut q_tile: Query<(&mut Interaction, Option<&Selection>), With<Tile>>) {
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
    } else if right_clicked {
        // this breaks for some reason if using left click
        // println!("Clear interaction");
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

fn make_floor(
    mut q_tiles: Query<(&mut Tile, &mut Sprite)>,
    grid: Res<Grid>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F) {
        let start = grid.get_xy(4, 5);
        if let Some(info) = start {
            if let Ok((mut tile, mut sprite)) = q_tiles.get_mut(info.entity) {
                tile.tile_state = TileState::Floor;
                sprite.color = tile.get_colour();
            }
        }
        let start = grid.get_xy(5, 5);
        if let Some(info) = start {
            if let Ok((mut tile, mut sprite)) = q_tiles.get_mut(info.entity) {
                tile.tile_state = TileState::Floor;
                sprite.color = tile.get_colour();
            }
        }

        let x = 9;
        let y = 10;
        // center
        let start = grid.get_xy(x, y);
        if let Some(info) = start {
            if let Ok((mut tile, mut sprite)) = q_tiles.get_mut(info.entity) {
                tile.tile_state = TileState::Floor;
                sprite.color = tile.get_colour();
                tile.number = 1;
            }
        }
        // neighbours
        let neighbours = grid.get_neighbours(x, y);
        for (i, ent) in neighbours.iter().enumerate() {
            if let Some(info) = ent {
                if let Ok((mut tile, mut sprite)) = q_tiles.get_mut(info.entity) {
                    tile.tile_state = TileState::Floor;
                    sprite.color = tile.get_colour();
                    // change this
                    tile.number = i + 5;
                }
            }
        }

        let x = 10;
        let y = 4;
        let coords = Coords::new(x, y);
        let center = grid.get_coords(coords);
        if let Some(info) = center {
            if let Ok((mut tile, mut sprite)) = q_tiles.get_mut(info.entity) {
                tile.tile_state = TileState::Floor;
                sprite.color = tile.get_colour();
                tile.number = 0;
            }
        }
        // set all to floor
        let neighbours = grid.get_neighbours(x as usize, y as usize);
        for ent in neighbours.iter() {
            if let Some(info) = ent {
                if let Ok((mut tile, mut sprite)) = q_tiles.get_mut(info.entity) {
                    tile.tile_state = TileState::Floor;
                    sprite.color = tile.get_colour();
                }
            }
        }

        let mut wall_set: HashSet<Coords> = HashSet::new();
        // now check for the numbers
        let neighbours = coords.get_neighbour_coords();
        for &c in neighbours.iter() {
            if c.x > 0 && c.y > 0 {
                if let Some(_info) = grid.get_coords(c) {
                    // if let Ok((mut tile, mut sprite)) = q_tiles.get_mut(info.entity) {
                    //     tile.tile_state = TileState::Floor;
                    //     sprite.color = tile.get_colour();
                    // }

                    //let mut wall_count = 0;
                    let wall_coords = c.get_neighbour_coords();
                    for &wc in wall_coords.iter() {
                        if let Some(info2) = grid.get_coords(wc) {
                            if let Ok((tile, _sprite)) = q_tiles.get(info2.entity) {
                                if tile.tile_state == TileState::Wall {
                                    //wall_count += 1;
                                    wall_set.insert(Coords::new(tile.x as i32, tile.y as i32));
                                }
                            }
                        }
                    }

                    //println!("wall set count: {:?}", wall_set.len());

                    // let mut random = wall_count;
                    // if number_total > random {

                    // } else {
                    //     random = number_total;
                    // }
                    // number_total -= usize::min(random, number_total);
                    // if let Ok((mut tile, _sprite)) = q_tiles.get_mut(info.entity) {
                    //     tile.number = random;
                    // }
                }
            }
        }
        // println!("final wall set count: {:?}", wall_set.len());

        let mut random_set = HashSet::new();
        let number_total = 8;

        for (i, c) in wall_set.drain().enumerate() {
            random_set.insert(c);
            println!("Bomb at {:?}", c);
            if i > number_total {
                break;
            }
        }
        // coordinates of the bombs
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

        // set numbers
        // let neighbours = grid.get_neighbours(x as usize, y as usize);
        // for ent in neighbours.iter() {
        //     if let Some(info) = ent {
        //         if let Ok((mut tile, mut sprite)) = q_tiles.get_mut(info.entity) {
        //             tile.number
        //         }
        //     }
        // }
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
