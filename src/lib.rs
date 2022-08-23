mod collision;
mod combat;
mod enemy_logic;
mod game;
mod health;
mod interaction;
mod worker_logic;

use bevy::prelude::*;

pub const LAUNCHER_TITLE: &str = "Bevy Jam - TBA";

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Selectable;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SceneState {
    MainMenu,
    InGame,
}

#[derive(Clone, Copy, Default, Component)]
pub struct PlayerCamera;

fn setup_player_camera(mut cmd: Commands) {
    let mut camera_transform = Camera2dBundle::default().transform;
    camera_transform.scale = Vec3::splat(2.2);
    camera_transform.scale = Vec3::splat(1.);

    cmd.spawn_bundle(Camera2dBundle {
        transform: camera_transform,
        ..Default::default()
    })
    .insert(PlayerCamera);
}

fn teardown_player_camera(
    mut cmd: Commands,
    q: Query<Entity, With<PlayerCamera>>,
) {
    for e in q.iter() {
        cmd.entity(e).despawn_recursive();
    }
}

pub fn get_children_recursive(
    entity: Entity,
    children_query: &Query<&Children>,
    callback: &mut impl FnMut(Entity),
) {
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            callback(*child);
            get_children_recursive(*child, children_query, callback);
        }
    }
}

pub fn app() -> App {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        title: LAUNCHER_TITLE.to_string(),
        canvas: Some("#bevy".to_string()),
        fit_canvas_to_parent: true,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(collision::CollisionPlugin)
    .add_plugin(interaction::InteractionPlugin)
    .add_plugin(game::GamePlugin)
    .add_plugin(worker_logic::WorkerLogicPlugin)
    .add_plugin(enemy_logic::EnemyLogicPlugin)
    .add_plugin(health::HealthPlugin)
    .add_plugin(combat::CombatPlugin)
    .add_state(SceneState::InGame) // FIXME: main menu
    .add_system_set(
        SystemSet::on_enter(SceneState::InGame)
            .with_system(setup_player_camera),
    )
    .add_system_set(
        SystemSet::on_exit(SceneState::InGame)
            .with_system(teardown_player_camera),
    );
    app
}
