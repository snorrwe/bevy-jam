use crate::{
    collision,
    interaction::MouseFollow,
    worker_logic::{
        CanEatWorker, UnitFollowPlayer, WorkerColor, WorkerEye, WorkerHead,
    },
    PlayerCamera, Selectable,
};
use bevy::prelude::*;
use rand::Rng;

pub struct GamePlugin;

#[derive(Default, Component)]
pub struct PlayerController;

#[derive(Default)]
pub struct GameAssets {
    pub player_sprite: Handle<TextureAtlas>,
    pub worker_head: Handle<TextureAtlas>,
    pub worker_head_eating: Handle<TextureAtlas>,
    pub worker_eye: Handle<TextureAtlas>,
    pub worker_body: Handle<TextureAtlas>,
}

#[derive(Default, Component)]
pub struct DontSortZ;
#[derive(Default, Component)]
pub struct ZOffset {
    offset: f32,
}

#[derive(Component)]
pub struct AvoidOthers;

#[derive(Component)]
pub struct SpawnAllies {
    max_count: u32,
    time_between_spawns: Timer,
}

fn avoid_others_system(
    avoiders: Query<
        (&GlobalTransform, Entity),
        (With<AvoidOthers>, Without<MouseFollow>),
    >,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    let mut change_these_vec: Vec<(Entity, Vec3)> = vec![];
    for (tr, e) in avoiders.iter() {
        for (tr2, e2) in avoiders.iter() {
            if e != e2 && (tr.translation() - tr2.translation()).length() < 70.
            {
                change_these_vec
                    .push((e, tr.translation() - tr2.translation()));
            }
        }
    }

    for (e, dir) in change_these_vec.iter() {
        if let Ok(mut tr) = transforms.get_mut(*e) {
            tr.translation += dir.normalize() * time.delta_seconds() * 100.;
        }
    }
}

fn spawn_workers_system(
    mut worker_spawners: Query<(&mut SpawnAllies, &GlobalTransform)>,
    workers: Query<Entity, With<UnitFollowPlayer>>,
    time: Res<Time>,
    mut cmd: Commands,
    game_assets: Res<GameAssets>,
) {
    for (mut spawner, global_tr) in worker_spawners.iter_mut() {
        if workers.iter().len() < spawner.max_count as usize {
            spawner.time_between_spawns.tick(time.delta());
            if spawner.time_between_spawns.finished() {
                spawn_regular_unit(
                    &mut cmd,
                    &game_assets,
                    global_tr.translation(),
                );
            }
        }
    }
}

fn z_sorter_system(
    mut q_transform_without_z_order: Query<
        (&mut Transform, &GlobalTransform),
        (Without<Camera>, Without<DontSortZ>, Without<ZOffset>),
    >,
    mut q_transform_with_z_order: Query<
        (&mut Transform, &GlobalTransform, &ZOffset),
        (Without<Camera>, Without<DontSortZ>),
    >,
) {
    for (mut tr, global_tr) in q_transform_without_z_order.iter_mut() {
        tr.translation.z = -global_tr.translation().y / 999999.;
    }

    for (mut tr, global_tr, z_offset) in q_transform_with_z_order.iter_mut() {
        tr.translation.z =
            -(global_tr.translation().y + z_offset.offset) / 999999.;
    }
}

fn handle_keyboard_movement(delta: &mut Vec2, keyboard_input: &Input<KeyCode>) {
    for key in keyboard_input.get_pressed() {
        match key {
            KeyCode::A | KeyCode::Left => {
                delta.x -= 1.0;
            }
            KeyCode::D | KeyCode::Right => {
                delta.x += 1.0;
            }
            KeyCode::W | KeyCode::Up => {
                delta.y += 1.0;
            }
            KeyCode::S | KeyCode::Down => {
                delta.y -= 1.0;
            }
            _ => {}
        }
    }
}

fn player_controll_system(
    mut q_player: Query<&mut Transform, With<PlayerController>>,
    inputs: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    let mut delta_movement = Vec2::new(0., 0.);
    handle_keyboard_movement(&mut delta_movement, &inputs);

    let player_speed = 300.;

    for mut tr in q_player.iter_mut() {
        tr.translation += delta_movement.extend(0.) * player_speed * delta_time;
    }
}

fn camera_follow_player_system(
    player_q: Query<&GlobalTransform, With<PlayerController>>,
    mut camera_q: Query<&mut Transform, With<PlayerCamera>>,
) {
    let mut camera_tr = camera_q.single_mut();
    let player_tr = player_q.single();

    camera_tr.translation = Vec3::new(
        player_tr.translation().x,
        player_tr.translation().y,
        camera_tr.translation.z,
    );
}

fn setup_game(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut game_assets: ResMut<GameAssets>,
) {
    game_assets.player_sprite = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/player/blob.png"),
        Vec2::new(146., 124.),
        1,
        1,
    ));
    game_assets.worker_body = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/workers/workerbody.png"),
        Vec2::new(35., 42.),
        1,
        1,
    ));
    game_assets.worker_head = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/workers/workerhead.png"),
        Vec2::new(37., 38.),
        1,
        1,
    ));
    game_assets.worker_eye = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/workers/workereyes.png"),
        Vec2::new(47., 26.),
        1,
        1,
    ));
    game_assets.worker_head_eating =
        texture_atlases.add(TextureAtlas::from_grid(
            asset_server.load("sprites/workers/headopenmouth.png"),
            Vec2::new(71., 52.),
            1,
            1,
        ));

    cmd.spawn_bundle(SpriteSheetBundle {
        texture_atlas: game_assets.player_sprite.clone(),
        transform: Transform::from_scale(Vec3::new(1., 1., 1.)),
        ..Default::default()
    })
    .insert(PlayerController)
    .insert(SpawnAllies {
        max_count: 10,
        time_between_spawns: Timer::from_seconds(1., true),
    })
    .insert(ZOffset { offset: -50. });

    spawn_regular_unit(&mut cmd, &game_assets, Vec3::new(180., 10., 0.));
}

fn spawn_regular_unit(cmd: &mut Commands, game_assets: &GameAssets, pos: Vec3) {
    let starter_colors = [
        Color::rgb(0., 1., 0.),
        Color::rgb(0., 0., 1.),
        Color::rgb(1., 0., 0.),
    ];
    let mut rng = rand::thread_rng();

    cmd.spawn_bundle(SpriteSheetBundle {
        texture_atlas: game_assets.worker_body.clone(),
        ..Default::default()
    })
    .insert_bundle(collision::AABBBundle {
        desc: collision::AABBDescriptor {
            radius: Vec3::splat(50.),
        },
        filter: collision::CollisionFilter {
            self_layers: collision::CollisionType::WORKER,
            collisions_mask: collision::CollisionType::WORKER_COLLISIONS,
        },
        ..Default::default()
    })
    .insert(UnitFollowPlayer)
    .insert(AvoidOthers)
    .insert(Selectable)
    .insert(WorkerColor {
        color: starter_colors[rng.gen_range(0..starter_colors.len())],
    })
    .insert(CanEatWorker {
        entity_to_eat: None,
    })
    // multiple bundles have transforms, insert at the end for safety
    .insert(Transform::from_translation(pos))
    .with_children(|child| {
        child
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_assets.worker_head.clone(),
                transform: Transform::from_translation(Vec3::new(0., 35., 0.1)),
                ..Default::default()
            })
            .insert(DontSortZ)
            .insert(WorkerHead)
            .with_children(|child2| {
                child2
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: game_assets.worker_eye.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            0., 0., 0.1,
                        )),
                        ..Default::default()
                    })
                    .insert(DontSortZ)
                    .insert(WorkerEye);
            });
    });
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets::default())
            .add_startup_system(setup_game)
            .add_system_to_stage(CoreStage::PostUpdate, z_sorter_system)
            .add_system(player_controll_system)
            .add_system(spawn_workers_system)
            .add_system(avoid_others_system)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                camera_follow_player_system,
            );
    }
}
