use crate::{
    animation::{Animation, RotationAnimation},
    audio::{AudioAssets, PlayAudioEventPositional},
    collision,
    easing::Easing,
    get_children_recursive,
    health::{hp_material, DestroyEntity, Health},
    interaction::MouseFollow,
    ui::{EndGameManager, EndGameState},
    worker_logic::{
        change_class, CanEatWorker, UnitClass, UnitFollowPlayer, UnitSize,
        WorkerHead,
    },
    DontDestroyBetweenLevels, GameTime, PlayerCamera, SceneState, Selectable,
};
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::Rng;

pub struct GamePlugin;

#[derive(Default, Component)]
pub struct PlayerController;

#[derive(Default, Component)]
pub struct MovementAnimationController {
    pub is_moving: bool,
    pub last_frame_pos: Vec3,
    pub time_to_stop_moving: Timer,
}

#[derive(Component, Clone, Copy)]
pub enum UnitType {
    Ally,
    Enemy,
}

#[derive(Default)]
pub struct GameAssets {
    pub player_sprite: Handle<TextureAtlas>,
    pub player_open_mouth: Handle<TextureAtlas>,
    pub player_eyes: Handle<TextureAtlas>,
    pub worker_head: Handle<TextureAtlas>,
    pub worker_head_eating: Handle<TextureAtlas>,
    pub worker_body: Handle<TextureAtlas>,
    pub circle_sprite: Handle<TextureAtlas>,
    pub hp_mesh: Handle<Mesh>,
    pub background: Handle<TextureAtlas>,
    pub forests: Handle<TextureAtlas>,
}

#[derive(Default)]
pub struct ResourceAssets {
    pub bloodrock_node: Handle<TextureAtlas>,
    pub bloodrock: Handle<TextureAtlas>,
}

#[derive(Component)]
pub struct BloodrockNode {
    pub amount_of_resource: usize,
}

#[derive(Default)]
pub struct MaxSupplyAmount(pub usize);

#[derive(Default)]
pub struct BloodrockAmount(pub usize);

#[derive(Component)]
pub struct WorkerResourceCarrySprite;

#[derive(Component)]
pub struct Harvester {
    pub target_node: Option<Entity>,
    pub harvest_speed: Timer,
    pub max_carryable_resource: usize,
    pub current_carried_resource: usize,
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
    pub max_count: u32,
    pub time_between_spawns: Timer,
}

fn check_lose_system(
    player: Query<Entity, With<PlayerController>>,
    mut end_game_state: ResMut<EndGameManager>,
) {
    if player.iter().len() <= 0 {
        end_game_state.state = EndGameState::Lose;
    }
}

fn harvester_carrying_something_system(
    children: Query<&Children>,
    units: Query<Entity, With<UnitFollowPlayer>>,
    mut carry_indicator_sprites: Query<
        &mut Transform,
        With<WorkerResourceCarrySprite>,
    >,
    harvesters: Query<&Harvester>,
) {
    for e in units.iter() {
        let mut sprite_size = 0.;

        if let Ok(harvester) = harvesters.get(e) {
            sprite_size = harvester.current_carried_resource as f32
                / harvester.max_carryable_resource as f32;
        }

        get_children_recursive(e, &children, &mut |child| {
            if let Ok(mut child_tr) = carry_indicator_sprites.get_mut(child) {
                child_tr.scale =
                    Vec3::splat(0.).lerp(Vec3::splat(1.5), sprite_size);
            }
        });
    }
}

fn harvester_logic_system(
    time: Res<GameTime>,
    mut harvesters: Query<(
        &mut Harvester,
        &mut Transform,
        &mut AvoidOthers,
        &GlobalTransform,
        &Velocity,
    )>,
    mut nodes: Query<
        (&GlobalTransform, &mut BloodrockNode, Entity),
        (Without<Harvester>, Without<PlayerController>),
    >,
    player_pos_q: Query<
        &GlobalTransform,
        (With<PlayerController>, Without<Harvester>),
    >,
    mut destroy_event_writer: EventWriter<DestroyEntity>,
    mut life_soul_amount: ResMut<BloodrockAmount>,
) {
    for player_p in player_pos_q.iter() {
        let player_pos = player_p.translation().truncate();
        for (mut harvester, mut tr, mut avoid_others, global_tr, velocity) in
            harvesters.iter_mut()
        {
            if let Some(target) = harvester.target_node {
                avoid_others.is_enabled = false;
                if let Ok((node_tr, mut resource_node, _)) =
                    nodes.get_mut(target)
                {
                    if (node_tr.translation().truncate()
                        - global_tr.translation().truncate())
                    .length()
                        < 60.
                    {
                        harvester.harvest_speed.tick(time.delta());

                        if harvester.harvest_speed.finished() {
                            harvester.harvest_speed.reset();
                            //Needs this check, since the node will be deleted in the postupdate stage
                            if resource_node.amount_of_resource > 0 {
                                resource_node.amount_of_resource -= 1;
                                harvester.current_carried_resource += 1;

                                harvester.current_carried_resource = harvester
                                    .current_carried_resource
                                    .clamp(0, harvester.max_carryable_resource);

                                let resource_node_depleted =
                                    resource_node.amount_of_resource == 0;

                                if resource_node_depleted {
                                    destroy_event_writer
                                        .send(DestroyEntity(target));
                                }
                                if resource_node_depleted || {
                                    harvester.current_carried_resource
                                        == harvester.max_carryable_resource
                                } {
                                    harvester.target_node = None;
                                }
                            }
                        }
                    } else {
                        let dir = (node_tr.translation().truncate()
                            - global_tr.translation().truncate())
                        .extend(0.)
                        .normalize();

                        tr.translation +=
                            dir * time.delta_seconds() * velocity.0;
                    }
                } else {
                    harvester.target_node = None;
                }
            } else {
                avoid_others.is_enabled = harvester.current_carried_resource
                    != harvester.max_carryable_resource;

                if (player_pos - global_tr.translation().truncate()).length()
                    < 100.
                {
                    if harvester.current_carried_resource
                        == harvester.max_carryable_resource
                    {
                        life_soul_amount.0 +=
                            harvester.current_carried_resource;
                        info!("Soul amount: {}", life_soul_amount.0);
                        harvester.current_carried_resource = 0;
                    }
                } else {
                    let dir = (player_pos - global_tr.translation().truncate())
                        .extend(0.)
                        .normalize();

                    tr.translation += dir * time.delta_seconds() * velocity.0;
                }

                //IF HAND IS NOT FULL - CHECK IF THERE'S A NODE NEARBY
                if harvester.current_carried_resource
                    != harvester.max_carryable_resource
                {
                    let mut closest_node: (Option<Entity>, f32) =
                        (None, 9999999.);
                    for (node_tr, _, e) in nodes.iter() {
                        let distance = (global_tr.translation().truncate()
                            - node_tr.translation().truncate())
                        .length();
                        if distance < closest_node.1 {
                            closest_node.1 = distance;
                            closest_node.0 = Some(e);
                        }
                    }
                    harvester.target_node = closest_node.0;
                }
            }
        }
    }
}

fn animate_on_movement_system(
    mut movement_animators: Query<(
        &Transform,
        &mut MovementAnimationController,
        Entity,
    )>,
    mut cmd: Commands,
    time: Res<GameTime>,
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
                mov.time_to_stop_moving.reset();
            }
        } else {
            if tr.translation == mov.last_frame_pos {
                mov.time_to_stop_moving.tick(time.delta());
                if mov.time_to_stop_moving.finished() {
                    mov.is_moving = false;
                    cmd.entity(e).insert(RotationAnimation(
                        Animation::<Quat> {
                            from: Quat::from_rotation_z(tr.rotation.z),
                            to: Quat::from_rotation_z(0.),
                            timer: Timer::from_seconds(0.1, false),
                            easing: Easing::Linear,
                        },
                    ));
                }
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
    for player_tr in player.iter() {
        let player_tr_vec2 = player_tr.translation().truncate();
        for (tr, e, avoider) in avoiders.iter() {
            let mut avoid_distance = (70., 100.);
            if !avoider.is_enabled {
                avoid_distance = (25., 30.);
            }
            let tr_vec2 = tr.translation().truncate();

            for (tr2, e2, _) in avoiders.iter() {
                let tr2_vec2 = tr2.translation().truncate();
                if e != e2 && (tr_vec2 - tr2_vec2).length() < avoid_distance.0 {
                    change_these_vec.push((e, (tr_vec2 - tr2_vec2).extend(0.)));
                }
            }

            if (tr_vec2 - player_tr_vec2).length() < avoid_distance.1 {
                change_these_vec
                    .push((e, (tr_vec2 - player_tr_vec2).extend(0.)));
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
}

fn spawn_workers_system(
    mut worker_spawners: Query<(&mut SpawnAllies, &GlobalTransform)>,
    workers: Query<Entity, With<UnitFollowPlayer>>,
    time: Res<GameTime>,
    mut cmd: Commands,
    game_assets: Res<GameAssets>,
    resource_assets: Res<ResourceAssets>,
    mut hp_assets: ResMut<Assets<hp_material::HpMaterial>>,
) {
    for (mut spawner, global_tr) in worker_spawners.iter_mut() {
        if workers.iter().len() < spawner.max_count as usize {
            spawner.time_between_spawns.tick(time.delta());
            if spawner.time_between_spawns.finished() {
                spawn_unit_with_class(
                    &mut cmd,
                    &game_assets,
                    &resource_assets,
                    global_tr.translation() + Vec3::new(100., 100., 0.),
                    UnitClass::Sworder,
                    &mut *hp_assets,
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

fn handle_keyboard_movement(
    delta: &mut Vec2,
    pressed_space: &mut bool,
    keyboard_input: &Input<KeyCode>,
) {
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
    for key in keyboard_input.get_just_pressed() {
        match key {
            KeyCode::Space => {
                *pressed_space = true;
            }
            _ => {}
        }
    }
}

fn handle_pausing_system(
    inputs: Res<Input<KeyCode>>,
    mut app_state: ResMut<State<SceneState>>,
) {
    for key in inputs.get_just_pressed() {
        match key {
            KeyCode::Escape => match app_state.current() {
                SceneState::InGame => {
                    app_state.push(SceneState::Paused).unwrap_or_default();
                }
                SceneState::Paused => {
                    app_state
                        .pop()
                        .expect("Couldnt unpause, because we failed to pop!");
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn player_controll_system(
    mut q_player: Query<&mut Transform, With<PlayerController>>,
    workers: Query<Entity, With<UnitFollowPlayer>>,
    inputs: Res<Input<KeyCode>>,
    time: Res<GameTime>,
    mut cmd: Commands,
    game_assets: Res<GameAssets>,
    resource_assets: Res<ResourceAssets>,
    mut bloodrock: ResMut<BloodrockAmount>,
    max_supply: Res<MaxSupplyAmount>,
    mut hp_assets: ResMut<Assets<hp_material::HpMaterial>>,
    audio_assets: Res<AudioAssets>,
    mut send_audio_event: EventWriter<PlayAudioEventPositional>,
) {
    let delta_time = time.delta_seconds();
    let mut delta_movement = Vec2::new(0., 0.);
    let mut pressed_space = false;
    handle_keyboard_movement(&mut delta_movement, &mut pressed_space, &inputs);

    let player_speed = 300.;

    for mut tr in q_player.iter_mut() {
        tr.translation += delta_movement.extend(0.) * player_speed * delta_time;

        if pressed_space
            && workers.iter().len() < max_supply.0
            && bloodrock.0 >= 10
        {
            send_audio_event.send(PlayAudioEventPositional {
                sound: audio_assets.spawning_unit.clone(),
                position: tr.translation,
            });
            bloodrock.0 -= 10;
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..=2);
            let spawn_point = tr.translation
                + Vec3::new(
                    rng.gen_range(-1.0..=1.0),
                    rng.gen_range(-1.0..=1.0),
                    0.,
                )
                .normalize()
                    * 100.;
            if index == 0 {
                spawn_unit_with_class(
                    &mut cmd,
                    &game_assets,
                    &resource_assets,
                    spawn_point,
                    UnitClass::Worker,
                    &mut *hp_assets,
                )
            } else if index == 1 {
                spawn_unit_with_class(
                    &mut cmd,
                    &game_assets,
                    &resource_assets,
                    spawn_point,
                    UnitClass::Sworder,
                    &mut *hp_assets,
                );
            } else if index == 2 {
                spawn_unit_with_class(
                    &mut cmd,
                    &game_assets,
                    &resource_assets,
                    spawn_point,
                    UnitClass::Ranged,
                    &mut *hp_assets,
                );
            }
        }
    }
}

fn camera_follow_player_system(
    player_q: Query<&GlobalTransform, With<PlayerController>>,
    mut camera_q: Query<&mut Transform, With<PlayerCamera>>,
) {
    for mut camera_tr in camera_q.iter_mut() {
        for player_tr in player_q.iter() {
            camera_tr.translation = Vec3::new(
                player_tr.translation().x,
                player_tr.translation().y,
                camera_tr.translation.z,
            );
        }
    }
}

fn setup_game(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut game_assets: ResMut<GameAssets>,
    mut resource_assets: ResMut<ResourceAssets>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
) {
    game_assets.hp_mesh = mesh_assets.add(Mesh::from(shape::Quad {
        size: Vec2::new(50.0, 10.0),
        flip: false,
    }));
    game_assets.player_sprite = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/player/playerclosedmouth.png"),
        Vec2::new(180., 172.),
        1,
        1,
    ));
    game_assets.player_open_mouth =
        texture_atlases.add(TextureAtlas::from_grid(
            asset_server.load("sprites/player/playeropenmouth.png"),
            Vec2::new(180., 172.),
            1,
            1,
        ));
    game_assets.player_eyes = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/player/playereyes.png"),
        Vec2::new(116., 89.),
        1,
        1,
    ));
    game_assets.worker_body = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/workers/workerbody.png"),
        Vec2::new(70., 70.),
        6,
        3,
    ));
    game_assets.worker_head = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/workers/workerhead.png"),
        Vec2::new(40., 38.),
        1,
        1,
    ));

    game_assets.worker_head_eating =
        texture_atlases.add(TextureAtlas::from_grid(
            asset_server.load("sprites/workers/headopenmouth.png"),
            Vec2::new(84., 64.),
            1,
            1,
        ));
    game_assets.circle_sprite = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/misc/circle.png"),
        Vec2::new(50., 50.),
        1,
        1,
    ));

    game_assets.background = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/misc/background.png"),
        Vec2::new(1000., 1000.),
        1,
        1,
    ));
    game_assets.forests = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/misc/forest1.png"),
        Vec2::new(242., 128.),
        1,
        1,
    ));

    resource_assets.bloodrock_node =
        texture_atlases.add(TextureAtlas::from_grid(
            asset_server.load("sprites/resources/bloodrocknode.png"),
            Vec2::new(77., 59.),
            1,
            1,
        ));

    resource_assets.bloodrock = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/resources/bloodrock.png"),
        Vec2::new(41., 41.),
        1,
        1,
    ));

    //Make background / borders
    for c in -2..=2 {
        for r in -2..=2 {
            let mut background_tr = Transform::from_scale(Vec3::splat(1.2));
            background_tr.translation =
                Vec3::new(c as f32 * 1000. * 1.2, r as f32 * 1000. * 1.2, 0.);

            cmd.spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_assets.background.clone(),
                transform: background_tr,
                ..Default::default()
            })
            .insert(ZOffset { offset: 10000. })
            .insert(DontDestroyBetweenLevels);
        }
    }

    //generate trees
    let mut rng = rand::thread_rng();
    for c in -5..=5 {
        for r in 0..=5 {
            let mut forest_tr: Transform =
                Transform::from_scale(Vec3::splat(2.2));
            forest_tr.translation = Vec3::new(
                c as f32 * 242. * 2.2 + r as f32 * 60.,
                700. + r as f32 * 100.,
                0.,
            );

            cmd.spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_assets.forests.clone(),
                transform: forest_tr,
                sprite: TextureAtlasSprite {
                    flip_x: rng.gen::<bool>(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(ZOffset { offset: -100. })
            .insert(DontDestroyBetweenLevels);
        }
    }
    for c in -5..=5 {
        for r in 0..=5 {
            let mut forest_tr: Transform =
                Transform::from_scale(Vec3::splat(2.2));
            forest_tr.translation = Vec3::new(
                c as f32 * 242. * 2.2 + r as f32 * 60.,
                -1000. + r as f32 * 100.,
                0.,
            );

            cmd.spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_assets.forests.clone(),
                transform: forest_tr,
                sprite: TextureAtlasSprite {
                    flip_x: rng.gen::<bool>(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(ZOffset { offset: -100. })
            .insert(DontDestroyBetweenLevels);
        }
    }

    for i in -1..=0 {
        for r in -4..=6 {
            let mut forest_tr: Transform =
                Transform::from_scale(Vec3::splat(2.2));
            forest_tr.translation =
                Vec3::new(-1500. + i as f32 * 550., r as f32 * 100., 0.);

            cmd.spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_assets.forests.clone(),
                transform: forest_tr,
                sprite: TextureAtlasSprite {
                    flip_x: rng.gen::<bool>(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(ZOffset { offset: -100. })
            .insert(DontDestroyBetweenLevels);
        }
    }
    for i in -1..=0 {
        for r in -4..=6 {
            let mut forest_tr: Transform =
                Transform::from_scale(Vec3::splat(2.2));
            forest_tr.translation =
                Vec3::new(1500. + i as f32 * 550., r as f32 * 100., 0.);

            cmd.spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_assets.forests.clone(),
                transform: forest_tr,
                sprite: TextureAtlasSprite {
                    flip_x: rng.gen::<bool>(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(ZOffset { offset: -100. })
            .insert(DontDestroyBetweenLevels);
        }
    }
}

pub fn spawn_bloodrock_node(
    cmd: &mut Commands,
    resource_assets: &ResourceAssets,
    pos: Vec3,
) {
    cmd.spawn_bundle(SpriteSheetBundle {
        texture_atlas: resource_assets.bloodrock_node.clone(),
        ..Default::default()
    })
    .insert(BloodrockNode {
        amount_of_resource: 100,
    })
    .insert(Transform::from_translation(pos));
}

fn spawn_unit_with_class(
    cmd: &mut Commands,
    game_assets: &GameAssets,
    resource_assets: &ResourceAssets,
    pos: Vec3,
    class: UnitClass,
    hp_assets: &mut Assets<hp_material::HpMaterial>,
) {
    let mut carry_sprite_transform =
        Transform::from_translation(Vec3::new(0., 0., 0.12));
    carry_sprite_transform.scale = Vec3::splat(0.);

    let entity_id = cmd
        .spawn_bundle(SpriteSheetBundle {
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
            time_to_stop_moving: Timer::from_seconds(0.3, false),
        })
        .insert(Velocity(100.))
        .insert(CanEatWorker {
            entity_to_eat: None,
        })
        .insert(class)
        .insert(UnitSize::Small)
        // multiple bundles have transforms, insert at the end for safety
        .insert(Transform::from_translation(pos))
        .with_children(|child| {
            child
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: resource_assets.bloodrock.clone(),
                    transform: carry_sprite_transform,
                    ..Default::default()
                })
                .insert(DontSortZ)
                .insert(WorkerResourceCarrySprite);
            child
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: game_assets.worker_head.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        0., 27., 0.1,
                    )),
                    ..Default::default()
                })
                .insert(DontSortZ)
                .insert(WorkerHead);
            child
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: bevy::sprite::Mesh2dHandle(
                        game_assets.hp_mesh.clone(),
                    ),
                    material: hp_assets.add(hp_material::HpMaterial {
                        color_empty: Color::RED,
                        color_full: Color::GREEN,
                        hp: 0.0,
                        hp_max: 0.0,
                    }),
                    transform: Transform::from_translation(
                        Vec3::Z * 200.0 + Vec3::Y * 60.0,
                    ),
                    ..Default::default()
                })
                .insert(DontSortZ);
        })
        .id();
    let mut health_comp = Health {
        current_health: 10.,
        max_health: 10.,
        armor: 0.,
    };
    change_class(entity_id, cmd, class, &mut health_comp);
    cmd.entity(entity_id).insert(health_comp);
}

pub enum LevelState {
    SpawnedStuff,
    NeedToSpawnStuff,
}

fn spawn_stuff(
    mut cmd: Commands,
    game_assets: Res<GameAssets>,
    resource_assets: Res<ResourceAssets>,
    mut level_state: ResMut<LevelState>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut hp_assets: ResMut<Assets<hp_material::HpMaterial>>,

    mut bloodrock_amount: ResMut<BloodrockAmount>,
) {
    if matches!(*level_state, LevelState::NeedToSpawnStuff) {
        bloodrock_amount.0 = 20;
        *level_state = LevelState::SpawnedStuff;

        spawn_bloodrock_node(
            &mut cmd,
            &resource_assets,
            Vec3::new(100., 100., 0.),
        );
        cmd.spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_assets.player_sprite.clone(),
            transform: Transform::from_scale(Vec3::new(1., 1., 1.)),
            ..Default::default()
        })
        .insert(PlayerController)
        .insert(ZOffset { offset: -50. })
        .insert(MovementAnimationController {
            is_moving: false,
            last_frame_pos: Vec3::splat(0.),
            time_to_stop_moving: Timer::from_seconds(0.3, false),
        })
        .insert(Health {
            current_health: 30.,
            max_health: 30.,
            armor: 0.,
        })
        .with_children(|child| {
            child
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: game_assets.player_eyes.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        0., 0., 0.1,
                    )),
                    ..Default::default()
                })
                .insert(DontSortZ);
            child
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: bevy::sprite::Mesh2dHandle(mesh_assets.add(
                        Mesh::from(shape::Quad {
                            size: Vec2::new(100.0, 13.0),
                            flip: false,
                        }),
                    )),
                    material: hp_assets.add(hp_material::HpMaterial {
                        color_empty: Color::RED,
                        color_full: Color::GREEN,
                        hp: 0.0,
                        hp_max: 0.0,
                    }),
                    transform: Transform::from_translation(
                        Vec3::Z * 200.0 + Vec3::Y * 100.0,
                    ),
                    ..Default::default()
                })
                .insert(DontSortZ);
        });

        spawn_unit_with_class(
            &mut cmd,
            &game_assets,
            &resource_assets,
            Vec3::new(180., 10., 0.),
            UnitClass::Sworder,
            &mut *hp_assets,
        );
        spawn_unit_with_class(
            &mut cmd,
            &game_assets,
            &resource_assets,
            Vec3::new(0., 200., 0.),
            UnitClass::Worker,
            &mut *hp_assets,
        );
    }
}

fn freeze_time(mut game_time: ResMut<GameTime>) {
    game_time.time_scale = 0.;
}

fn resume_time(mut game_time: ResMut<GameTime>) {
    game_time.time_scale = 1.;
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets::default())
            .insert_resource(LevelState::NeedToSpawnStuff)
            .insert_resource(ResourceAssets::default())
            .insert_resource(MaxSupplyAmount(10))
            .insert_resource(BloodrockAmount(20))
            .add_startup_system(setup_game)
            .add_system_to_stage(CoreStage::PostUpdate, z_sorter_system)
            .add_system(handle_pausing_system)
            .add_system_set(
                SystemSet::on_update(SceneState::InGame)
                    .with_system(player_controll_system)
                    .with_system(spawn_workers_system)
                    .with_system(avoid_others_system)
                    .with_system(animate_on_movement_system)
                    .with_system(harvester_logic_system)
                    .with_system(harvester_carrying_something_system)
                    .with_system(check_lose_system),
            )
            .add_system_set(
                SystemSet::on_enter(SceneState::InGame)
                    .with_system(spawn_stuff),
            )
            .add_system_set(
                SystemSet::on_enter(SceneState::Paused)
                    .with_system(freeze_time),
            )
            .add_system_set(
                SystemSet::on_exit(SceneState::Paused).with_system(resume_time),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                camera_follow_player_system,
            );
    }
}
