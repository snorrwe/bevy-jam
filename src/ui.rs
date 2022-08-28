use crate::{
    easing::Easing,
    enemy_logic::LevelManager,
    game::{BloodrockAmount, MaxSupplyAmount},
    lerp::Lerp,
    worker_logic::UnitFollowPlayer,
    DontDestroyBetweenLevels, GameTime, SceneState,
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

#[derive(Component)]
pub struct WaveText;

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
    mut wave_texts: Query<&mut Text, (With<WaveText>, Without<SupplyText>)>,
    level_manager: Res<LevelManager>,
    max_supply: Res<MaxSupplyAmount>,

    workers: Query<Entity, With<UnitFollowPlayer>>,
) {
    for mut text in supply_texts.iter_mut() {
        text.sections[0].value =
            format!("Units: {} / {}", workers.iter().len(), max_supply.0);
    }

    for mut text in wave_texts.iter_mut() {
        text.sections[0].value = format!(
            "Wave: {} / {}",
            level_manager.current_level.current_wave_index,
            level_manager.current_level.waves.len()
        );
    }
}
#[derive(Component)]
pub struct FaderScreenComponent;

fn setup_in_game_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    root_node: Query<Entity, With<RootNode>>,
    mut losing_manager: ResMut<EndGameManager>,
) {
    losing_manager.state = EndGameState::NotEndGame;
    losing_manager.time_to_fade_in = Timer::from_seconds(1., false);
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

                            size: Size::new(
                                Val::Percent(100.0),
                                Val::Percent(100.0),
                            ),
                            ..Default::default()
                        },
                        color: UiColor(Color::BLACK),
                        ..Default::default()
                    })
                    .insert(Fade {
                        start_color: Color::BLACK,
                        end_color: Color::rgba(0., 0., 0., 0.),
                        time_to_fade: Timer::from_seconds(3., false),
                        easing: Easing::QuartOut,
                    })
                    .insert(FaderScreenComponent);

                child
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
                                right: Val::Auto,
                                left: Val::Percent(0.),
                                top: Val::Percent(10.),
                                bottom: Val::Auto,
                            },

                            size: Size::new(Val::Percent(100.0), Val::Auto),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::FlexEnd,
                            ..Default::default()
                        },
                        color: UiColor(Color::NONE),
                        ..Default::default()
                    })
                    .with_children(|child| {
                        child
                            .spawn_bundle(TextBundle::from_section(
                                "",
                                TextStyle {
                                    font: asset_server
                                        .load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 50.0,
                                    color: Color::WHITE,
                                },
                            ))
                            .insert(EndGameTextComponent);
                    });

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
                                "Units: 0 / 10",
                                TextStyle {
                                    font: asset_server
                                        .load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 30.0,
                                    color: Color::WHITE,
                                },
                            ))
                            .insert(SupplyText);
                    });

                child
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
                                right: Val::Auto,
                                left: Val::Percent(0.),
                                top: Val::Percent(3.),
                                bottom: Val::Auto,
                            },

                            size: Size::new(Val::Percent(100.0), Val::Auto),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::FlexEnd,
                            ..Default::default()
                        },
                        color: UiColor(Color::NONE),
                        ..Default::default()
                    })
                    .with_children(|child| {
                        child
                            .spawn_bundle(TextBundle::from_section(
                                "",
                                TextStyle {
                                    font: asset_server
                                        .load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 35.0,
                                    color: Color::WHITE,
                                },
                            ))
                            .insert(WaveText);
                    });
                child
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
                                left: Val::Px(40.),
                                right: Val::Auto,
                                bottom: Val::Px(40.),
                                top: Val::Auto,
                            },
                            size: Size::new(Val::Px(100.0), Val::Auto),
                            ..Default::default()
                        },
                        color: UiColor(Color::NONE),
                        ..Default::default()
                    })
                    .with_children(|child| {
                        child.spawn_bundle(TextBundle::from_section(
                            "WASD to move
Drag and Drop units to combine them
Space to spawn new units (for 10 bloodrocks)",
                            TextStyle {
                                font: asset_server
                                    .load("fonts/FiraSans-Bold.ttf"),
                                font_size: 30.0,
                                color: Color::WHITE,
                            },
                        ));
                    });
            });
        });
    }
}

fn destroy_in_game_ui(
    mut cmd: Commands,
    main_node: Query<Entity, With<MainInGameNode>>,
) {
    info!("Destroying in game ui");
    for e in main_node.iter() {
        cmd.entity(e).despawn_recursive();
    }
}

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
            fill: egui::Color32::from_rgb(115, 99, 114),
            shadow: egui::epaint::Shadow::small_light(),
            rounding: egui::Rounding::from(8.),
            ..Default::default()
        })
        .default_pos(egui::Pos2 { x: -500., y: -500. })
        .anchor(egui::Align2::CENTER_TOP, egui::Vec2 { x: 0., y: 300. })
        .show(egui_ctx.ctx_mut(), |ui| {
            let widget_visuals = egui::style::WidgetVisuals {
                bg_fill: egui::Color32::from_rgb(170, 192, 170),
                bg_stroke: egui::Stroke {
                    width: 1.,
                    color: egui::Color32::from_rgb(220, 238, 209),
                },
                rounding: egui::Rounding::from(8.),
                fg_stroke: egui::Stroke {
                    width: 5.,
                    color: egui::Color32::BLACK,
                },
                expansion: 0.,
            };
            ui.visuals_mut().widgets = egui::style::Widgets {
                inactive: widget_visuals,
                ..Default::default()
            };

            match *ui_state {
                UIState::None => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.);
                        if ui
                            .add_sized(
                                [220.0, 80.0],
                                egui::Button::new("Start game"),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            app_state
                                .set(SceneState::InGame)
                                .unwrap_or_default();
                        }
                        ui.add_space(50.);
                        if ui
                            .add_sized(
                                [220.0, 80.0],
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
                            .add_sized([220.0, 80.0], egui::Button::new("Quit"))
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            exit.send(AppExit);
                        }
                        ui.add_space(20.);
                    });
                }
                _ => {}
            }
        });
}

#[derive(Component)]
pub struct MainMenuNode;
fn setup_main_menu(
    mut cmd: Commands,
    root_node: Query<Entity, With<RootNode>>,
) {
    for node in root_node.iter() {
        cmd.entity(node).with_children(|child| {
            child
                .spawn_bundle(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        size: Size::new(
                            Val::Percent(100.0),
                            Val::Percent(100.0),
                        ),
                        ..Default::default()
                    },
                    color: UiColor(Color::BLACK),
                    ..Default::default()
                })
                .insert(MainMenuNode);
        });
    }
}

#[derive(Component)]
pub struct Fade {
    start_color: Color,
    end_color: Color,
    time_to_fade: Timer,
    easing: Easing,
}

fn fader_system(
    mut faders: Query<(&mut Fade, &mut UiColor)>,
    time: Res<GameTime>,
) {
    for (mut fade_comp, mut color) in faders.iter_mut() {
        fade_comp.time_to_fade.tick(time.delta());
        color.0 = fade_comp.start_color.lerp(
            &fade_comp.end_color,
            fade_comp
                .easing
                .get_easing(fade_comp.time_to_fade.percent()),
        );
    }
}
fn destroy_main_menu(
    mut cmd: Commands,
    root_node: Query<Entity, With<MainMenuNode>>,
) {
    for node in root_node.iter() {
        cmd.entity(node).despawn_recursive();
    }
}

fn ui_menus_system(
    mut egui_ctx: ResMut<EguiContext>,
    mut ui_state: ResMut<UIState>,
) {
    match *ui_state {
        UIState::Options => {
            egui::Window::new("")
                .id(egui::Id::new(2))
                .resizable(false)
                .title_bar(false)
                .frame(egui::Frame {
                    fill: egui::Color32::from_rgb(115, 99, 114),
                    shadow: egui::epaint::Shadow::small_light(),
                    rounding: egui::Rounding::from(8.),
                    ..Default::default()
                })
                .default_pos(egui::Pos2 { x: -500., y: -500. })
                .anchor(egui::Align2::CENTER_TOP, egui::Vec2 { x: 0., y: 300. })
                .show(egui_ctx.ctx_mut(), |ui| {
                    let widget_visuals = egui::style::WidgetVisuals {
                        bg_fill: egui::Color32::from_rgb(170, 192, 170),
                        bg_stroke: egui::Stroke {
                            width: 1.,
                            color: egui::Color32::from_rgb(220, 238, 209),
                        },
                        rounding: egui::Rounding::from(8.),
                        fg_stroke: egui::Stroke {
                            width: 5.,
                            color: egui::Color32::BLACK,
                        },
                        expansion: 0.,
                    };
                    ui.visuals_mut().widgets = egui::style::Widgets {
                        inactive: widget_visuals,
                        ..Default::default()
                    };

                    ui.vertical_centered(|ui| {
                        ui.separator();
                        if ui
                            .add_sized(
                                [220.0, 80.0],
                                egui::Button::new("Back to Menu"),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            *ui_state = UIState::None
                        }
                        ui.add_space(20.);
                    });
                });
        }
        _ => {}
    }
}

fn pause_menu_logic(
    mut egui_ctx: ResMut<EguiContext>,
    mut ui_state: ResMut<UIState>,
    mut app_state: ResMut<State<SceneState>>,
) {
    egui::Window::new("")
        .id(egui::Id::new(1))
        .resizable(false)
        .title_bar(false)
        .frame(egui::Frame {
            fill: egui::Color32::from_rgb(115, 99, 114),
            shadow: egui::epaint::Shadow::small_light(),
            rounding: egui::Rounding::from(8.),
            ..Default::default()
        })
        .default_pos(egui::Pos2 { x: -500., y: -500. })
        .anchor(egui::Align2::CENTER_TOP, egui::Vec2 { x: 0., y: 300. })
        .show(egui_ctx.ctx_mut(), |ui| {
            let widget_visuals = egui::style::WidgetVisuals {
                bg_fill: egui::Color32::from_rgb(170, 192, 170),
                bg_stroke: egui::Stroke {
                    width: 1.,
                    color: egui::Color32::from_rgb(220, 238, 209),
                },
                rounding: egui::Rounding::from(8.),
                fg_stroke: egui::Stroke {
                    width: 5.,
                    color: egui::Color32::BLACK,
                },
                expansion: 0.,
            };
            ui.visuals_mut().widgets = egui::style::Widgets {
                inactive: widget_visuals,
                ..Default::default()
            };

            match *ui_state {
                UIState::None => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.);
                        if ui
                            .add_sized(
                                [220.0, 80.0],
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
                                [220.0, 80.0],
                                egui::Button::new("Options"),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            *ui_state = UIState::Options;
                        }
                        ui.add_space(20.);

                        ui.separator();
                        if ui
                            .add_sized(
                                [220.0, 80.0],
                                egui::Button::new("Back to Menu"),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            app_state
                                .set(SceneState::MainMenu)
                                .unwrap_or_default();
                        }
                        ui.add_space(20.);
                    });
                }
                _ => {}
            }
        });
}

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
    .insert(RootNode)
    .insert(DontDestroyBetweenLevels);
}

fn clear_ui_states(mut ui_state: ResMut<UIState>) {
    *ui_state = UIState::None;
}

pub enum EndGameState {
    NotEndGame,
    Win,
    Lose,
}

pub struct EndGameManager {
    pub state: EndGameState,
    pub time_to_fade_in: Timer,
}

use std::time::Duration;
#[derive(Component)]
pub struct EndGameTextComponent;
fn end_game_manager_system(
    mut end_game_manager: ResMut<EndGameManager>,
    time: Res<GameTime>,
    mut end_game_text: Query<&mut Text, With<EndGameTextComponent>>,
    mut egui_ctx: ResMut<EguiContext>,
    mut app_state: ResMut<State<SceneState>>,
    mut cmd: Commands,
    fader_screen: Query<Entity, With<FaderScreenComponent>>,
) {
    match end_game_manager.state {
        EndGameState::Lose => {
            if end_game_manager.time_to_fade_in.elapsed()
                == Duration::from_millis(0)
            {
                for e in fader_screen.iter() {
                    cmd.entity(e).insert(Fade {
                        start_color: Color::rgba(0., 0., 0., 0.),
                        end_color: Color::rgba(0., 0., 0., 0.5),
                        time_to_fade: Timer::from_seconds(1., false),
                        easing: Easing::QuartOut,
                    });
                }
            }
            end_game_manager.time_to_fade_in.tick(time.delta());
            if end_game_manager.time_to_fade_in.finished() {
                for mut text in end_game_text.iter_mut() {
                    text.sections[0].value = format!("You lost!");
                }
            }
        }
        EndGameState::Win => {
            if end_game_manager.time_to_fade_in.elapsed()
                == Duration::from_millis(0)
            {
                for e in fader_screen.iter() {
                    cmd.entity(e).insert(Fade {
                        start_color: Color::rgba(0., 0., 0., 0.),
                        end_color: Color::rgba(0., 0., 0., 0.5),
                        time_to_fade: Timer::from_seconds(1., false),
                        easing: Easing::QuartOut,
                    });
                }
            }

            end_game_manager.time_to_fade_in.tick(time.delta());
            if end_game_manager.time_to_fade_in.finished() {
                for mut text in end_game_text.iter_mut() {
                    text.sections[0].value = format!("You win!");
                }
            }
        }

        _ => {}
    }

    if !matches!(end_game_manager.state, EndGameState::NotEndGame)
        && end_game_manager.time_to_fade_in.finished()
    {
        egui::Window::new("")
            .id(egui::Id::new(5))
            .resizable(false)
            .title_bar(false)
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(115, 99, 114),
                shadow: egui::epaint::Shadow::small_light(),
                rounding: egui::Rounding::from(8.),
                ..Default::default()
            })
            .default_pos(egui::Pos2 { x: -500., y: -500. })
            .anchor(egui::Align2::CENTER_TOP, egui::Vec2 { x: 0., y: 300. })
            .show(egui_ctx.ctx_mut(), |ui| {
                let widget_visuals = egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_rgb(170, 192, 170),
                    bg_stroke: egui::Stroke {
                        width: 1.,
                        color: egui::Color32::from_rgb(220, 238, 209),
                    },
                    rounding: egui::Rounding::from(8.),
                    fg_stroke: egui::Stroke {
                        width: 5.,
                        color: egui::Color32::BLACK,
                    },
                    expansion: 0.,
                };
                ui.visuals_mut().widgets = egui::style::Widgets {
                    inactive: widget_visuals,
                    ..Default::default()
                };

                ui.vertical_centered(|ui| {
                    ui.separator();

                    if ui
                        .add_sized(
                            [220.0, 80.0],
                            egui::Button::new("Back to Menu"),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        app_state.set(SceneState::MainMenu).unwrap_or_default();
                    }
                    ui.add_space(20.);
                });
            });
    }
}

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UIState::None)
            .insert_resource(EndGameManager {
                time_to_fade_in: Timer::from_seconds(1., false),
                state: EndGameState::NotEndGame,
            })
            .add_startup_system(ui_first_setup)
            .add_system(fader_system)
            .add_system(ui_menus_system)
            .add_system_set(
                SystemSet::on_enter(SceneState::InGame)
                    .with_system(setup_in_game_ui),
            )
            .add_system_set(
                SystemSet::on_update(SceneState::InGame)
                    .with_system(update_bloodrock_text)
                    .with_system(update_supply_text)
                    .with_system(end_game_manager_system),
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
            )
            .add_system_set(
                SystemSet::on_exit(SceneState::Paused)
                    .with_system(clear_ui_states),
            );
    }
}
