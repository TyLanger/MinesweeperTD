use bevy::prelude::*;

use crate::{
    castle::Castle, director::SpawnInfo, loading::FontAssets, tower::TowerServer, GameState,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ButtonPressEvent>()
            .add_event::<ButtonHoverEvent>()
            .add_event::<SwitchPlayEvent>();
        app.add_system_set(SystemSet::on_enter(GameState::MainMenu).with_system(setup_main_menu))
            .add_system_set(
                SystemSet::on_update(GameState::MainMenu)
                    .with_system(click_play_button)
                    .with_system(switch_to_playing),
            )
            .add_system_set(SystemSet::on_exit(GameState::MainMenu).with_system(cleanup_menu));
        //spawn_director_menu
        app.add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(spawn_tower_menu)
                .with_system(spawn_director_menu),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(update_buttons)
                .with_system(update_castle_stats)
                .with_system(update_tower_info_panel)
                .with_system(update_director_panel),
        );
        // .add_startup_system(spawn_tower_menu.after(setup_towers))
        // app.add_event::<ButtonPressEvent>()
        //     .add_event::<SwitchPlayEvent>()
        //     .add_system(update_buttons)
        //     .add_system(update_castle_stats);
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

// Events
pub struct ButtonPressEvent {
    pub button_number: usize,
}

pub struct ButtonHoverEvent {
    pub button_number: usize,
}

// Components
#[derive(Component)]
pub struct ButtonInfo {
    base_text: String,
    hovered_text: String,
    button_number: usize,
}

#[derive(Component)]
pub struct StartButton;

fn setup_main_menu(mut commands: Commands, font_assets: Res<FontAssets>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(120.0), Val::Px(50.0)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .insert(StartButton)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::from_section(
                    "Play".to_string(),
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ),
                ..default()
            });
        });
}

fn click_play_button(
    // mut state: ResMut<State<GameState>>,
    mut q_interaction: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>, With<StartButton>),
    >,
    mut ev_switch: EventWriter<SwitchPlayEvent>,
) {
    for (interaction, mut color) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                // state.set(GameState::Playing).unwrap();
                // switch_to_playing(state);
                ev_switch.send(SwitchPlayEvent);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

struct SwitchPlayEvent;

fn switch_to_playing(ev_switch: EventReader<SwitchPlayEvent>, mut state: ResMut<State<GameState>>) {
    if !ev_switch.is_empty() {
        ev_switch.clear();
        state.set(GameState::Playing).unwrap();
    }
}

fn cleanup_menu(mut commands: Commands, q_button: Query<Entity, With<StartButton>>) {
    for entity in q_button.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn update_buttons(
    mut q_interaction: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<Button>, Without<StartButton>),
    >,
    q_child: Query<&ButtonInfo>,
    mut q_text: Query<&mut Text>,
    mut ev_button_press: EventWriter<ButtonPressEvent>,
    mut ev_button_hover: EventWriter<ButtonHoverEvent>,
) {
    for (interaction, mut color, children) in q_interaction.iter_mut() {
        let mut text = q_text.get_mut(children[0]).unwrap();
        let info = q_child.get(children[0]);
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                ev_button_press.send(ButtonPressEvent {
                    button_number: info.unwrap().button_number,
                });
            }
            Interaction::Hovered => {
                text.sections[0].value = info.unwrap().hovered_text.clone();
                *color = HOVERED_BUTTON.into();
                ev_button_hover.send(ButtonHoverEvent {
                    button_number: info.unwrap().button_number,
                });
            }
            Interaction::None => {
                text.sections[0].value = info.unwrap().base_text.clone();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn spawn_tower_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tower_server: Res<TowerServer>,
    fonts: Res<FontAssets>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(0.0),
                    ..default()
                },
                size: Size::new(Val::Percent(20.0), Val::Percent(100.0)),
                // left-right
                justify_content: JustifyContent::SpaceEvenly,
                // up-down
                align_content: AlignContent::Center,
                //align_items: AlignItems::FlexEnd,
                flex_wrap: FlexWrap::Wrap,

                ..default()
            },
            // #262b44
            color: Color::rgb_u8(0x26, 0x2b, 0x44).into(),
            ..default()
        })
        .with_children(|root| {
            for (i, tower) in tower_server.towers.iter().enumerate() {
                //let button_text = vec!["Green", "Red", "Blue", "Orange"];
                //for i in 0..4 {
                root.spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(40.0), Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        // coloured box around text
                        padding: UiRect {
                            left: Val::Px(2.0),
                            right: Val::Px(2.0),
                            top: Val::Px(2.0),
                            bottom: Val::Px(2.0),
                        },
                        // whitespace around the button
                        margin: UiRect {
                            left: Val::Px(2.0),
                            right: Val::Px(2.0),
                            top: Val::Px(2.0),
                            bottom: Val::Px(2.0),
                            //..default()
                        },
                        ..default()
                    },
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|button_base| {
                    button_base
                        .spawn_bundle(TextBundle::from_section(
                            "Button",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 20.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ))
                        .insert(ButtonInfo {
                            base_text: tower.visuals.name.to_string(),
                            hovered_text: "Build Tower".to_string(),
                            button_number: i,
                        });
                });
            }
        })
        .with_children(|root| {
            root.spawn_bundle(
                TextBundle::from_sections([
                    // TextSection::new(
                    //     "Hovered button info\n",
                    //     TextStyle {
                    //         font: fonts.fira_sans.clone(),
                    //         font_size: 20.0,
                    //         color: Color::rgb(0.9, 0.9, 0.9),
                    //     },
                    // ),
                    TextSection::new(
                        "Basic Tower\n",
                        TextStyle {
                            font: fonts.fira_sans.clone(),
                            font_size: 20.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ),
                    TextSection::new(
                        "Cost: ",
                        TextStyle {
                            font: fonts.fira_sans.clone(),
                            font_size: 20.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ),
                    TextSection::new(
                        "10\n",
                        TextStyle {
                            font: fonts.fira_sans.clone(),
                            font_size: 20.0,
                            // #fee761
                            color: Color::rgb_u8(0xf3, 0xe7, 0x61),
                        },
                    ),
                    TextSection::new(
                        "Range: ",
                        TextStyle {
                            font: fonts.fira_sans.clone(),
                            font_size: 20.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ),
                    TextSection::new(
                        "60\n",
                        TextStyle {
                            font: fonts.fira_sans.clone(),
                            font_size: 20.0,
                            // #0099db
                            color: Color::rgb_u8(0x00, 0x99, 0xdb),
                        },
                    ),
                ])
                .with_style(Style {
                    align_self: AlignSelf::FlexEnd,
                    // whitespace around the button
                    margin: UiRect {
                        // left: Val::Px(2.0),
                        // right: Val::Px(2.0),
                        // top: Val::Px(2.0),
                        bottom: Val::Px(30.0),
                        ..default()
                    },
                    // position_type: PositionType::Absolute,
                    // position: UiRect {
                    //     bottom: Val::Percent(5.0),
                    //     right: Val::Percent(20.5),
                    //     ..default()
                    // },
                    ..default()
                }),
            )
            .insert(TowerInfoPanel);
        });
}

#[derive(Component)]
struct TowerInfoPanel;

fn update_tower_info_panel(
    mut ev_button_hover: EventReader<ButtonHoverEvent>,
    mut q_ui: Query<&mut Text, With<TowerInfoPanel>>,
    tower_server: Res<TowerServer>,
) {
    for ev in ev_button_hover.iter() {
        for mut text in q_ui.iter_mut() {
            let cost = tower_server.towers[ev.button_number].cost;
            let range = tower_server.towers[ev.button_number].range;

            let name = match ev.button_number {
                0 => {
                    // basic tower
                    "Basic Tower\n"
                }
                1 => {
                    // shotgun
                    "Shotgun Tower\n"
                }
                2 => {
                    // bomb
                    "Bomb Tower\n"
                }
                3 => {
                    // swarm
                    "Swarm Tower\n"
                }
                _ => "Error Tower\n",
            };
            // 0 is Basic Tower\n
            // 1 is Cost
            // 2 is 10\n
            // 3 is Range
            // 4 is 60\n
            // text.sections[0]
            text.sections[0].value = name.to_string();
            text.sections[2].value = format!("{:}\n", cost);
            text.sections[4].value = format!("{:}\n", range);
        }
    }
}

fn spawn_director_menu(mut commands: Commands, fonts: Res<FontAssets>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                // position_type: PositionType::Absolute,
                // position: UiRect {
                //     left: Val::Px(0.0),
                //     ..default()
                // },
                size: Size::new(Val::Percent(20.0), Val::Percent(100.0)),
                // left-right
                justify_content: JustifyContent::Center,
                // up-down
                align_content: AlignContent::Center,
                //align_items: AlignItems::FlexEnd,
                flex_wrap: FlexWrap::Wrap,

                ..default()
            },
            // #262b44
            color: Color::rgb_u8(0x26, 0x2b, 0x44).into(),
            ..default()
        })
        .with_children(|root| {
            root.spawn_bundle(
                TextBundle::from_sections([
                    // fields
                    // SpawnInfo {
                    //     wave_timer: todo!(),
                    //     batch_size: todo!(),
                    //     difficulty: todo!(),
                    //     next_strat: todo!(),
                    //     positions: todo!(),
                    // }
                    TextSection::new(
                        "Director\n",
                        TextStyle {
                            font: fonts.fira_sans.clone(),
                            font_size: 20.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ),
                    TextSection::new(
                        "Will spawn ",
                        TextStyle {
                            font: fonts.fira_sans.clone(),
                            font_size: 20.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ),
                    TextSection::new(
                        "15 enemies\n",
                        TextStyle {
                            font: fonts.fira_sans.clone(),
                            font_size: 20.0,
                            // #fee761
                            color: Color::rgb_u8(0xf3, 0xe7, 0x61),
                        },
                    ),
                    TextSection::new(
                        "in a ",
                        TextStyle {
                            font: fonts.fira_sans.clone(),
                            font_size: 20.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ),
                    TextSection::new(
                        "Burst\n",
                        TextStyle {
                            font: fonts.fira_sans.clone(),
                            font_size: 20.0,
                            // #0099db
                            color: Color::rgb_u8(0x00, 0x99, 0xdb),
                        },
                    ),
                    TextSection::new(
                        "in ",
                        TextStyle {
                            font: fonts.fira_sans.clone(),
                            font_size: 20.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ),
                    TextSection::new(
                        "5.5s\n",
                        TextStyle {
                            font: fonts.fira_sans.clone(),
                            font_size: 20.0,
                            // #0099db
                            color: Color::rgb_u8(0x00, 0x99, 0xdb),
                        },
                    ),
                ])
                .with_style(Style {
                    // justify_content: JustifyContent::Center,
                    // align_items: AlignItems::Center,
                    align_self: AlignSelf::Center,
                    // whitespace around the button
                    // margin: UiRect {
                    //     // left: Val::Px(2.0),
                    //     // right: Val::Px(2.0),
                    //     // top: Val::Px(2.0),
                    //     //bottom: Val::Px(30.0),
                    //     ..default()
                    // },
                    // position_type: PositionType::Absolute,
                    // position: UiRect {
                    //     bottom: Val::Percent(5.0),
                    //     right: Val::Percent(20.5),
                    //     ..default()
                    // },
                    ..default()
                }),
            )
            .insert(DirectorInfoPanel);
        });
}

#[derive(Component)]
struct DirectorInfoPanel;

fn update_director_panel(
    mut q_ui: Query<&mut Text, With<DirectorInfoPanel>>,
    spawn_info: Res<SpawnInfo>,
) {
    for mut text in q_ui.iter_mut() {
        // 0 Director
        // 1 Will spawn
        // 2 15 enemies
        // 3 in a
        // 4 Burst
        // 5 in
        // 6 5.5s
        text.sections[2].value = format!("{:} enemies\n", spawn_info.batch_size);
        text.sections[4].value = format!("{:?}\n", spawn_info.next_strat);
        text.sections[6].value = format!("{:.1}s\n", spawn_info.get_time());
    }
}

fn spawn_castle_stats(
    commands: &mut Commands,
    // asset_server: &Res<AssetServer>,
    fonts: &Res<FontAssets>,
    health: u32,
    money: u32,
) {
    // let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font = fonts.fira_sans.clone();
    commands
        .spawn_bundle(
            TextBundle::from_sections([
                TextSection::new(
                    "Castle Health: ",
                    TextStyle {
                        font: font.clone(),
                        font_size: 25.0,
                        color: Color::WHITE,
                    },
                ),
                TextSection::new(
                    format!("{:}\n", health),
                    TextStyle {
                        font: font.clone(),
                        font_size: 25.0,
                        color: Color::GOLD,
                    },
                ),
                TextSection::new(
                    "Gold: ",
                    TextStyle {
                        font: font.clone(),
                        font_size: 25.0,
                        color: Color::WHITE,
                    },
                ),
                TextSection::new(
                    format!("{:}\n", money),
                    TextStyle {
                        font,
                        font_size: 25.0,
                        color: Color::GOLD,
                    },
                ),
            ])
            .with_text_alignment(TextAlignment::TOP_RIGHT)
            .with_style(Style {
                // align_self: AlignSelf::FlexEnd,
                // justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    // left: Val::Percent(30.0),
                    // right: Val::Percent(50.0),
                    // right: Val::Px(5.0),
                    right: Val::Percent(20.5),
                    // left: Val::Percent(81.5),
                    ..default()
                },
                // flex_shrink: 0.0,
                // padding: UiRect {
                //     left: Val::Px(2.0),
                //     right: Val::Px(2.0),
                //     top: Val::Px(2.0),
                //     bottom: Val::Px(2.0),
                // },
                // margin: UiRect {
                //     left: Val::Px(2.0),
                //     right: Val::Px(2.0),
                //     top: Val::Px(2.0),
                //     bottom: Val::Px(2.0),
                //     //..default()
                // },
                ..default()
            }),
        )
        .insert(CastleUi);
}

// pub struct UpdateCastleStatsEvent {
//     health: u32,
//     money: u32,
// }

#[derive(Component)]
struct CastleUi;

fn update_castle_stats(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    fonts: Res<FontAssets>,
    q_castle: Query<&Castle, Changed<Castle>>,
    mut q_castle_ui: Query<&mut Text, With<CastleUi>>,
) {
    for castle in q_castle.iter() {
        //println!("Updating stats");
        if q_castle_ui.is_empty() {
            spawn_castle_stats(&mut commands, &fonts, castle.health, castle.money);
        }
        for mut text in q_castle_ui.iter_mut() {
            text.sections[1].value = format!("{:}\n", castle.health);
            text.sections[3].value = format!("{:}\n", castle.money);
        }
    }
}
