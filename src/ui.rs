use crate::{
    game::{BloodrockAmount, MaxSupplyAmount},
    worker_logic::UnitFollowPlayer,
    SceneState,
};
use bevy::prelude::*;
pub struct UIPlugin;
use bevy::app::AppExit;
use bevy_egui::{egui, EguiContext, EguiSettings};

#[derive(Component)]
pub struct MainInGameNode;

#[derive(Component)]
pub struct BloodrockText;

#[derive(Component)]
pub struct SupplyText;

#[derive(PartialEq, Clone)]
pub enum UIState {
    Options,
    None,
}

fn update_bloodrock_text(
    mut resource_texts: Query<&mut Text, With<BloodrockText>>,
    bloodrock_amount: Res<BloodrockAmount>,
) {
    for mut resource_text in resource_texts.iter_mut() {
        resource_text.sections[0].value = format!(": {}", bloodrock_amount.0);
    }
}

fn update_supply_text(
    mut supply_texts: Query<&mut Text, With<SupplyText>>,
    max_supply: Res<MaxSupplyAmount>,

    workers: Query<Entity, With<UnitFollowPlayer>>,
) {
    for mut text in supply_texts.iter_mut() {
        text.sections[0].value =
            format!("Supply: {} / {}", workers.iter().len(), max_supply.0);
    }
}

fn setup_in_game_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    root_node: Query<Entity, With<RootNode>>,
) {
    for node in root_node.iter() {
        commands.entity(node).with_children(|cmd| {
            //Main node
            cmd.spawn_bundle(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::FlexStart,
                    align_content: AlignContent::FlexStart,
                    justify_content: JustifyContent::FlexStart,
                    ..Default::default()
                },
                color: UiColor(Color::NONE),
                ..Default::default()
            })
            .insert(MainInGameNode)
            .with_children(|child| {
                child
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
                                right: Val::Auto,
                                left: Val::Px(0.),
                                top: Val::Px(0.),
                                bottom: Val::Auto,
                            },
                            ..Default::default()
                        },
                        color: UiColor(Color::NONE),
                        ..Default::default()
                    })
                    .with_children(|child| {
                        child.spawn_bundle(ImageBundle {
                            style: Style {
                                size: Size::new(Val::Px(70.0), Val::Px(70.0)),
                                ..Default::default()
                            },
                            image: asset_server
                                .load("sprites/resources/bloodrock.png")
                                .into(),
                            ..Default::default()
                        });
                        child
                            .spawn_bundle(TextBundle::from_section(
                                ": 0",
                                TextStyle {
                                    font: asset_server
                                        .load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 30.0,
                                    color: Color::WHITE,
                                },
                            ))
                            .insert(BloodrockText);
                    });
                child
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
                                right: Val::Px(0.),
                                left: Val::Auto,
                                top: Val::Px(0.),
                                bottom: Val::Auto,
                            },
                            size: Size::new(Val::Px(400.0), Val::Auto),
                            ..Default::default()
                        },
                        color: UiColor(Color::NONE),
                        ..Default::default()
                    })
                    .with_children(|child| {
                        child
                            .spawn_bundle(TextBundle::from_section(
                                "Supply: 0 / 10",
                                TextStyle {
                                    font: asset_server
                                        .load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 30.0,
                                    color: Color::WHITE,
                                },
                            ))
                            .insert(SupplyText);
                    });
            });
        });
    }
}

fn destroy_in_game_ui(
    mut cmd: Commands,
    main_node: Query<Entity, With<MainInGameNode>>,
) {
    for e in main_node.iter() {
        cmd.entity(e).despawn_recursive();
    }
}
fn setup_main_menu(mut cmd: Commands) {}

fn main_menu_logic(
    mut egui_ctx: ResMut<EguiContext>,
    mut exit: EventWriter<AppExit>,
    mut ui_state: ResMut<UIState>,
    mut app_state: ResMut<State<SceneState>>,
) {
    match *ui_state {
        UIState::None => {}
        _ => {
            return;
        }
    }
    egui::Window::new("")
        .id(egui::Id::new(0))
        .resizable(false)
        .title_bar(false)
        .frame(egui::Frame {
            fill: egui::Color32::from_rgb(41, 50, 65),
            shadow: egui::epaint::Shadow::small_light(),
            rounding: egui::Rounding::from(8.),
            ..Default::default()
        })
        .default_pos(egui::Pos2 { x: -500., y: -500. })
        .anchor(egui::Align2::CENTER_TOP, egui::Vec2 { x: 0., y: 300. })
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.visuals_mut().widgets = egui::style::Widgets {
                inactive: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_rgb(61, 90, 128),
                    bg_stroke: egui::Stroke {
                        width: 1.,
                        color: egui::Color32::from_rgb(100, 140, 160),
                    },
                    rounding: egui::Rounding::from(8.),
                    fg_stroke: egui::Stroke {
                        width: 2.,
                        color: egui::Color32::WHITE,
                    },
                    expansion: 0.,
                },
                ..Default::default()
            };

            match *ui_state {
                UIState::None => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.);
                        if ui
                            .add_sized(
                                [200.0, 50.0],
                                egui::Button::new("Start game"),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            //TODO: ADD FADE!! on InGame -s tart, or here i guesss
                            app_state
                                .set(SceneState::InGame)
                                .unwrap_or_default();
                        }
                        ui.add_space(50.);
                        if ui
                            .add_sized(
                                [200.0, 50.0],
                                egui::Button::new("Options"),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            *ui_state = UIState::Options;
                        }

                        ui.separator();
                        ui.add_space(20.);
                        if ui
                            .add_sized([200.0, 50.0], egui::Button::new("Quit"))
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            exit.send(AppExit);
                        }
                    });
                }
                _ => {}
            }
        });
}
fn destroy_main_menu(mut cmd: Commands) {}
fn pause_menu_logic(
    mut egui_ctx: ResMut<EguiContext>,
    mut ui_state: ResMut<UIState>,
    mut app_state: ResMut<State<SceneState>>,
) {
    egui::Window::new("")
        .id(egui::Id::new(0))
        .resizable(false)
        .title_bar(false)
        .frame(egui::Frame {
            fill: egui::Color32::from_rgb(41, 50, 65),
            shadow: egui::epaint::Shadow::small_light(),
            rounding: egui::Rounding::from(8.),
            ..Default::default()
        })
        .default_pos(egui::Pos2 { x: -500., y: -500. })
        .anchor(egui::Align2::CENTER_TOP, egui::Vec2 { x: 0., y: 300. })
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.visuals_mut().widgets = egui::style::Widgets {
                inactive: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_rgb(61, 90, 128),
                    bg_stroke: egui::Stroke {
                        width: 1.,
                        color: egui::Color32::from_rgb(100, 140, 160),
                    },
                    rounding: egui::Rounding::from(8.),
                    fg_stroke: egui::Stroke {
                        width: 2.,
                        color: egui::Color32::WHITE,
                    },
                    expansion: 0.,
                },
                ..Default::default()
            };

            match *ui_state {
                UIState::None => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.);
                        if ui
                            .add_sized(
                                [200.0, 50.0],
                                egui::Button::new("Resume"),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            app_state.pop().unwrap_or_default();
                        }
                        ui.add_space(20.);
                        if ui
                            .add_sized(
                                [200.0, 50.0],
                                egui::Button::new("Options"),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            *ui_state = UIState::Options;
                        }
                        ui.add_space(20.);
                        if ui
                            .add_sized(
                                [200.0, 50.0],
                                egui::Button::new("Back to Menu"),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            app_state
                                .set(SceneState::MainMenu)
                                .unwrap_or_default();
                        }
                    });
                }
                _ => {}
            }
        });
}

fn fader_system() {}

#[derive(Component)]
pub struct RootNode;
fn ui_first_setup(mut cmd: Commands) {
    cmd.spawn_bundle(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::FlexStart,
            align_content: AlignContent::FlexStart,
            justify_content: JustifyContent::FlexStart,
            ..Default::default()
        },
        color: UiColor(Color::NONE),
        ..Default::default()
    })
    .insert(RootNode);
}

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UIState::None)
            .add_startup_system(ui_first_setup)
            .add_system_set(
                SystemSet::on_enter(SceneState::InGame)
                    .with_system(setup_in_game_ui),
            )
            .add_system_set(
                SystemSet::on_update(SceneState::InGame)
                    .with_system(update_bloodrock_text)
                    .with_system(update_supply_text),
            )
            .add_system_set(
                SystemSet::on_enter(SceneState::MainMenu)
                    .with_system(setup_main_menu)
                    .with_system(destroy_in_game_ui),
            )
            .add_system_set(
                SystemSet::on_update(SceneState::MainMenu)
                    .with_system(main_menu_logic),
            )
            .add_system_set(
                SystemSet::on_exit(SceneState::MainMenu)
                    .with_system(destroy_main_menu),
            )
            .add_system_set(
                SystemSet::on_update(SceneState::Paused)
                    .with_system(pause_menu_logic),
            );
    }
}
