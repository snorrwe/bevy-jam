use crate::{game::BloodrockAmount, SceneState};
use bevy::prelude::*;
pub struct UIPlugin;

#[derive(Component)]
pub struct MainInGameNode;

#[derive(Component)]
pub struct BloodrockText;

fn update_bloodrock_text(
    mut resource_texts: Query<&mut Text, With<BloodrockText>>,
    bloodrock_amount: Res<BloodrockAmount>,
) {
    for mut resource_text in resource_texts.iter_mut() {
        resource_text.sections[0].value = format!(": {}", bloodrock_amount.0);
    }
}

fn setup_in_game_ui(mut cmd: Commands, asset_server: Res<AssetServer>) {
    //Main node
    cmd.spawn_bundle(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            ..Default::default()
        },
        color: UiColor(Color::NONE),
        ..Default::default()
    })
    .insert(MainInGameNode)
    .with_children(|child| {
        child.spawn_bundle(ImageBundle {
            style: Style {
                size: Size::new(Val::Px(70.0), Val::Px(70.0)),
                ..Default::default()
            },
            image: asset_server.load("sprites/resources/bloodrock.png").into(),
            ..Default::default()
        });
        child
            .spawn_bundle(
                TextBundle::from_section(
                    ": 0",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 30.0,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                }),
            )
            .insert(BloodrockText);
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
                .with_system(update_bloodrock_text),
        );
    }
}
