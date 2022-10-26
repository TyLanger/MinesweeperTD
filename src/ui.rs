use bevy::prelude::*;

use crate::tower::{setup_towers, TowerServer};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_tower_menu.after(setup_towers))
            .add_event::<ButtonPressEvent>()
            .add_system(update_buttons);
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

// Events
pub struct ButtonPressEvent {
    pub button_number: usize,
}

// Components
#[derive(Component)]
struct ButtonInfo {
    base_text: String,
    hovered_text: String,
    button_number: usize,
}

fn update_buttons(
    mut q_interaction: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    q_child: Query<&ButtonInfo>,
    mut q_text: Query<&mut Text>,
    mut ev_button_press: EventWriter<ButtonPressEvent>,
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
        });
}
