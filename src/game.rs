use bevy::prelude::*;

use crate::{collision, PlayerCamera, Selectable};

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

fn z_sorter(
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
        tr.translation.z = -global_tr.translation().y / 1000.;
    }

    for (mut tr, global_tr, z_offset) in q_transform_with_z_order.iter_mut() {
        tr.translation.z =
            -(global_tr.translation().y + z_offset.offset) / 1000.;
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
        Vec2::new(364., 307.),
        1,
        1,
    ));
    game_assets.worker_body = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/workers/workerbody.png"),
        Vec2::new(88., 104.),
        1,
        1,
    ));
    game_assets.worker_head = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/workers/workerhead.png"),
        Vec2::new(89., 95.),
        1,
        1,
    ));
    game_assets.worker_eye = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/workers/workereyes.png"),
        Vec2::new(106., 74.),
        1,
        1,
    ));

    cmd.spawn_bundle(SpriteSheetBundle {
        texture_atlas: game_assets.player_sprite.clone(),
        transform: Transform::from_scale(Vec3::new(1., 1., 1.)),
        ..Default::default()
    })
    .insert(PlayerController)
    .insert(ZOffset { offset: -100. });

    spawn_regular_unit(&mut cmd, &game_assets);
}

fn spawn_regular_unit(cmd: &mut Commands, game_assets: &GameAssets) {
    cmd.spawn_bundle(SpriteSheetBundle {
        texture_atlas: game_assets.worker_body.clone(),
        ..Default::default()
    })
    .insert_bundle(collision::AABBBundle {
        desc: collision::AABBDescriptor {
            radius: Vec3::splat(150.),
        },
        filter: collision::CollisionFilter {
            self_layers: collision::CollisionType::WORKER,
            collisions_mask: collision::CollisionType::WORKER_COLLISIONS,
        },
        ..Default::default()
    })
    .insert(Selectable)
    // multiple bundles have transforms, insert at the end for safety
    .insert(Transform::from_translation(Vec3::new(180., 0., 10.)))
    .with_children(|child| {
        child
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_assets.worker_head.clone(),
                transform: Transform::from_translation(Vec3::new(
                    0., 85., 0.001,
                )),
                ..Default::default()
            })
            .insert(DontSortZ)
            .with_children(|child2| {
                child2
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: game_assets.worker_eye.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            0., 0., 0.001,
                        )),
                        ..Default::default()
                    })
                    .insert(DontSortZ);
            });
    });
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets::default())
            .add_startup_system(setup_game)
            .add_system_to_stage(CoreStage::PostUpdate, z_sorter)
            .add_system(player_controll_system)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                camera_follow_player_system,
            );
    }
}
