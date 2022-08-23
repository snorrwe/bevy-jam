mod animation;
mod collision;
mod combat;
mod enemy_logic;
mod game;
mod health;
mod interaction;
mod lerp;
mod particles;
mod worker_logic;

use std::time::Duration;

use bevy::prelude::*;

pub const LAUNCHER_TITLE: &str = "Bevy Jam - TBA";

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Selectable;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SceneState {
    MainMenu,
    InGame,
}

pub struct GameTime {
    pub real_delta: Duration,
    pub time_scale: f32,
    pub time_to_reset_time_scale: Timer,
}

impl Default for GameTime {
    fn default() -> Self {
        GameTime {
            real_delta: Duration::default(),
            time_scale: DEFAULT_TIME_SCALE,
            time_to_reset_time_scale: Timer::default(),
        }
    }
}

impl GameTime {
    pub fn delta_seconds(&self) -> f32 {
        let delta_secs = self.real_delta.as_secs() as f32
            + self.real_delta.subsec_nanos() as f32 * 1e-9;

        // clamp delta-time to 0.1 sec
        delta_secs.min(0.1) * self.time_scale
    }

    pub fn delta(&self) -> Duration {
        // clamp delta-time to 0.1 sec
        self.real_delta
            .min(Duration::from_millis(100))
            .mul_f32(self.time_scale)
    }
}

pub struct ChangeTimeScaleEvent {
    /// TimeScale cant be negative for now.
    pub new_time_scale: f32,
    /// Timescale will be reset after N seconds
    pub seconds_to_change: f32,
}

const DEFAULT_TIME_SCALE: f32 = 1.0;

fn game_time_update(
    time: Res<Time>,
    mut game_time: ResMut<GameTime>,
    mut change_events: EventReader<ChangeTimeScaleEvent>,
) {
    let delta = time.delta();
    game_time.real_delta = delta;

    for event in change_events.iter() {
        game_time.time_scale = event.new_time_scale;
        game_time.time_to_reset_time_scale =
            Timer::from_seconds(event.seconds_to_change, false);
    }

    game_time.time_to_reset_time_scale.tick(delta);

    if game_time.time_to_reset_time_scale.just_finished() {
        game_time.time_scale = DEFAULT_TIME_SCALE;
    }
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
    .add_plugin(animation::AnimationsPlugin)
    .add_plugin(particles::ParticlePlugin)
    .add_state(SceneState::InGame) // FIXME: main menu
    .add_system_set(
        SystemSet::on_enter(SceneState::InGame)
            .with_system(setup_player_camera),
    )
    .add_system_set(
        SystemSet::on_update(SceneState::InGame).with_system(game_time_update),
    )
    .add_system_set(
        SystemSet::on_exit(SceneState::InGame)
            .with_system(teardown_player_camera),
    )
    .insert_resource(GameTime::default())
    .add_event::<ChangeTimeScaleEvent>();

    app
}
