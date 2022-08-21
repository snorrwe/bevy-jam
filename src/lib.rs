mod collision;
mod game;

use bevy::{prelude::*, render::camera::*};

pub const LAUNCHER_TITLE: &str = "Bevy Jam - TBA";

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SceneState {
    MainMenu,
    InGame,
}

fn setup_test(mut cmd: Commands) {
    cmd.spawn_bundle(collision::AABBBundle {
        desc: collision::AABBDescriptor { radius: Vec3::ONE },
        filter: collision::CollisionFilter {
            self_layers: collision::CollisionType::PLAYER,
            collisions_mask: collision::CollisionType::PLAYER_COLLISIONS,
        },
        ..Default::default()
    });
}

#[derive(Clone, Copy, Default, Component)]
pub struct PlayerCamera;

fn setup_player_camera(mut cmd: Commands) {
    let mut camera_transform = Camera2dBundle::default().transform;

    camera_transform.scale = Vec3::splat(2.2);

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
    .add_plugin(game::GamePlugin)
    .add_startup_system(setup_test) // FIXME: remove
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
