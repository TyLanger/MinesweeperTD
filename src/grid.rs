use bevy::prelude::*;

use crate::MouseWorldPos;

// palette from https://lospec.com/palette-list/endesga-32
pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid { tiles: Vec::new() })
            .add_event::<ClearSelectionsEvent>()
            .add_startup_system(setup)
            .add_system(clear_interaction.before(interaction))
            .add_system(interaction)
            .add_system(tile_interaction.after(interaction))
            .add_system(clear_selection.after(tile_interaction));
    }
}

const GRID_WIDTH: usize = 18;
const GRID_HEIGHT: usize = 14;
const TILE_SIZE: f32 = 30.0;

// Events
pub struct ClearSelectionsEvent;

#[derive(Component)]
struct Tile {
    colour: Color,
    selection: Color,
    hover: Color,
}

impl Tile {
    fn new(colour: Color) -> Self {
        Tile {
            colour,
            selection: Color::MIDNIGHT_BLUE,
            hover: Color::ALICE_BLUE,
        }
    }
}

pub struct Grid {
    pub tiles: Vec<Entity>,
}

impl Grid {
    pub fn get_vec2(&self, pos: Vec2) -> Option<Entity> {
        let x = ((GRID_WIDTH as f32 * 0.5 * TILE_SIZE) + TILE_SIZE * 0.5 + pos.x) / TILE_SIZE;
        let y = ((GRID_HEIGHT as f32 * 0.5 * TILE_SIZE) + TILE_SIZE * 0.5 + pos.y) / TILE_SIZE;

        if x < 0.0 || y < 0.0 {
            return None;
        }

        self.get_xy(x as usize, y as usize)
    }

    fn get_xy(&self, x: usize, y: usize) -> Option<Entity> {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            return None;
        }

        // in order (0, 0), (0, 1), (0, 2)
        let index = x * GRID_HEIGHT + y;
        let tile = self.tiles.get(index);
        tile.copied()
        // I guess this replaces this:
        // match tile {
        //     Some(&t) => Some(t),
        //     None => None,
        // }
    }
}

fn setup(mut commands: Commands, mut grid: ResMut<Grid>) {
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
                Color::rgb_u8(0x63, 0xc7, 0x4d)
            };

            let pos = offset + Vec3::new(i as f32 * TILE_SIZE, j as f32 * TILE_SIZE, 0.0);

            let tile = commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_translation(pos),
                    ..default()
                })
                .insert(Tile::new(color))
                .insert(Interaction::None)
                .id();

            grid.tiles.push(tile);
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
fn interaction(
    mut q_tile: Query<&mut Interaction, With<Tile>>,
    grid: Res<Grid>,
    mouse: Res<MouseWorldPos>,
    mouse_click: Res<Input<MouseButton>>,
) {
    let hovered = grid.get_vec2(mouse.0);
    if let Some(ent) = hovered {
        let target = q_tile.get_mut(ent);
        if let Ok(mut interaction) = target {
            match *interaction {
                Interaction::Clicked => {
                    // do nothing
                }
                Interaction::Hovered => {
                    // this can't run
                    if mouse_click.just_pressed(MouseButton::Left) {
                        *interaction = Interaction::Clicked;
                    }
                }
                Interaction::None => {
                    *interaction = Interaction::Hovered;
                    if mouse_click.just_pressed(MouseButton::Left) {
                        *interaction = Interaction::Clicked;
                    }
                }
            }
        }
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
                sprite.color = tile.colour;
            }
        }
    }
}

fn clear_selection(
    mut commands: Commands,
    q_selection: Query<Entity, With<Selection>>,
    keyboard: Res<Input<KeyCode>>,
    ev_clear: EventReader<ClearSelectionsEvent>,
) {
    if keyboard.just_pressed(KeyCode::Space) || !ev_clear.is_empty() {
        ev_clear.clear();
        for entity in q_selection.iter() {
            commands.entity(entity).remove::<Selection>();
        }
    }
}
