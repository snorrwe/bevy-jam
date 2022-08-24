use crate::{
    game::{BloodrockAmount, MaxSupplyAmount},
    worker_logic::UnitFollowPlayer,
    SceneState,
};
use bevy::prelude::*;
pub struct UIPlugin;

#[derive(Component)]
pub struct MainInGameNode;

#[derive(Component)]
pub struct BloodrockText;

#[derive(Component)]
pub struct SupplyText;

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

fn setup_in_game_ui(mut cmd: Commands, asset_server: Res<AssetServer>) {
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
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
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
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    ))
                    .insert(SupplyText);
            });
    });
}

fn destroy_in_game_ui(mut cmd: Commands) {}

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(SceneState::InGame)
                .with_system(setup_in_game_ui),
        )
        .add_system_set(
            SystemSet::on_exit(SceneState::InGame)
                .with_system(destroy_in_game_ui),
        )
        .add_system_set(
            SystemSet::on_update(SceneState::InGame)
                .with_system(update_bloodrock_text)
                .with_system(update_supply_text),
        );
    }
}
