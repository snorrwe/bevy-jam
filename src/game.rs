use crate::{
    animation::{Animation, RotationAnimation},
    collision,
    combat::{AttackState, AttackType, CombatComponent},
    enemy_logic::EnemySpawner,
    health::Health,
    interaction::MouseFollow,
    particles::Easing,
    worker_logic::{
        CanEatWorker, UnitFollowPlayer, WorkerColor, WorkerEye, WorkerHead,
    },
    GameTime, PlayerCamera, Selectable,
};
use bevy::prelude::*;
use rand::Rng;

pub struct GamePlugin;

#[derive(Default, Component)]
pub struct PlayerController;

#[derive(Default, Component)]
pub struct MovementAnimationController {
    is_moving: bool,
    last_frame_pos: Vec3,
}

#[derive(Default)]
pub struct GameAssets {
    pub player_sprite: Handle<TextureAtlas>,
    pub worker_head: Handle<TextureAtlas>,
    pub worker_head_eating: Handle<TextureAtlas>,
    pub worker_eye: Handle<TextureAtlas>,
    pub worker_body: Handle<TextureAtlas>,
    pub circle_sprite: Handle<TextureAtlas>,
}

#[derive(Default, Component)]
pub struct DontSortZ;
#[derive(Default, Component)]
pub struct ZOffset {
    offset: f32,
}
#[derive(Component)]
pub struct Velocity(pub f32);

#[derive(Component)]
pub struct AvoidOthers {
    pub is_enabled: bool,
}

#[derive(Component)]
pub struct SpawnAllies {
    max_count: u32,
    time_between_spawns: Timer,
}

fn animate_on_movement_system(
    mut movement_animators: Query<(
        &Transform,
        &mut MovementAnimationController,
        Entity,
    )>,
    mut cmd: Commands,
) {
    for (tr, mut mov, e) in movement_animators.iter_mut() {
        if mov.is_moving == false {
            if tr.translation != mov.last_frame_pos {
                mov.is_moving = true;
                cmd.entity(e).insert(RotationAnimation(Animation::<Quat> {
                    from: Quat::from_rotation_z(-0.18),
                    to: Quat::from_rotation_z(0.18),
                    timer: Timer::from_seconds(0.5, true),
                    easing: Easing::PulsateInOutCubic,
                }));
            }
        } else {
            if tr.translation == mov.last_frame_pos {
                mov.is_moving = false;
                cmd.entity(e).insert(RotationAnimation(Animation::<Quat> {
                    from: Quat::from_rotation_z(tr.rotation.z),
                    to: Quat::from_rotation_z(0.),
                    timer: Timer::from_seconds(0.1, false),
                    easing: Easing::Linear,
                }));
            }
        }

        mov.last_frame_pos = tr.translation;
    }
}

fn avoid_others_system(
    avoiders: Query<
        (&GlobalTransform, Entity, &AvoidOthers),
        Without<MouseFollow>,
    >,
    player: Query<
        &GlobalTransform,
        (With<PlayerController>, Without<AvoidOthers>),
    >,
    mut transforms: Query<&mut Transform>,
    time: Res<GameTime>,
) {
    let mut change_these_vec: Vec<(Entity, Vec3)> = vec![];
    let player_tr = player.single();
    let player_tr_vec2 = player_tr.translation().truncate();
    for (tr, e, avoider) in avoiders.iter() {
        if !avoider.is_enabled {
            continue;
        }
        let tr_vec2 = tr.translation().truncate();

        for (tr2, e2, _) in avoiders.iter() {
            let tr2_vec2 = tr2.translation().truncate();
            if e != e2 && (tr_vec2 - tr2_vec2).length() < 70. {
                change_these_vec.push((e, (tr_vec2 - tr2_vec2).extend(0.)));
            }
        }

        if (tr_vec2 - player_tr_vec2).length() < 100. {
            change_these_vec.push((e, (tr_vec2 - player_tr_vec2).extend(0.)));
        }
    }

    for (e, dir) in change_these_vec.iter() {
        if let Ok(mut tr) = transforms.get_mut(*e) {
            let mut direction = *dir;
            if direction == Vec3::ZERO {
                let mut rng = rand::thread_rng();
                direction = Vec3::new(
                    rng.gen_range(-1.0..=1.0),
                    rng.gen_range(-1.0..=1.0),
                    0.,
                );
            }
            tr.translation +=
                direction.normalize() * time.delta_seconds() * 100.;
        }
    }
}

fn spawn_workers_system(
    mut worker_spawners: Query<(&mut SpawnAllies, &GlobalTransform)>,
    workers: Query<Entity, With<UnitFollowPlayer>>,
    time: Res<GameTime>,
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
                    global_tr.translation() + Vec3::new(100., 100., 0.),
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
    time: Res<GameTime>,
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
    game_assets.circle_sprite = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/misc/circle.png"),
        Vec2::new(50., 50.),
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
        time_between_spawns: Timer::from_seconds(5., true),
    })
    .insert(EnemySpawner {
        time_between_spawns: Timer::from_seconds(5., true),
        distance_from_spawn_point: 400.,
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
    .insert(AvoidOthers { is_enabled: true })
    .insert(Selectable)
    .insert(MovementAnimationController {
        is_moving: false,
        last_frame_pos: pos,
    })
    .insert(Velocity(100.))
    .insert(WorkerColor {
        color: starter_colors[rng.gen_range(0..starter_colors.len())],
    })
    .insert(CanEatWorker {
        entity_to_eat: None,
    })
    .insert(Health {
        current_health: 10.,
        max_health: 10.,
    })
    .insert(CombatComponent {
        target: None,
        damage: 1.,
        time_between_attacks: Timer::from_seconds(1., true),
        attack_range: 70.,
        attack_type: AttackType::Melee,
        attack_state: AttackState::NotAttacking,
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
            .add_system(animate_on_movement_system)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                camera_follow_player_system,
            );
    }
}
