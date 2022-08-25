use crate::{
    animation::{Animation, RotationAnimation},
    collision,
    easing::Easing,
    enemy_logic::EnemySpawner,
    get_children_recursive,
    health::{DestroyEntity, Health},
    interaction::MouseFollow,
    worker_logic::{
        change_class, CanEatWorker, UnitClass, UnitFollowPlayer, UnitSize,
        WorkerColor, WorkerEye, WorkerHead,
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

fn harvester_carrying_something_system(
    children: Query<&Children>,
    harvesters: Query<(&Harvester, Entity)>,
    mut carry_indicator_sprites: Query<
        &mut Transform,
        With<WorkerResourceCarrySprite>,
    >,
) {
    for (harvester, e) in harvesters.iter() {
        get_children_recursive(e, &children, &mut |child| {
            if let Ok(mut child_tr) = carry_indicator_sprites.get_mut(child) {
                child_tr.scale = Vec3::splat(0.).lerp(
                    Vec3::splat(1.5),
                    harvester.current_carried_resource as f32
                        / harvester.max_carryable_resource as f32,
                );
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
    let player_pos = player_pos_q.single().translation().truncate();

    for (mut harvester, mut tr, mut avoid_others, global_tr, velocity) in
        harvesters.iter_mut()
    {
        if let Some(target) = harvester.target_node {
            avoid_others.is_enabled = false;
            if let Ok((node_tr, mut resource_node, _)) = nodes.get_mut(target) {
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

                    tr.translation += dir * time.delta_seconds() * velocity.0;
                }
            }
        } else {
            avoid_others.is_enabled = harvester.current_carried_resource
                != harvester.max_carryable_resource;

            if (player_pos - global_tr.translation().truncate()).length() < 100.
            {
                if harvester.current_carried_resource
                    == harvester.max_carryable_resource
                {
                    life_soul_amount.0 += harvester.current_carried_resource;
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
                let mut closest_node: (Option<Entity>, f32) = (None, 9999999.);
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
    resource_assets: Res<ResourceAssets>,
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
            bloodrock.0 -= 10;
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..=2);
            if index == 0 {
                spawn_unit_with_class(
                    &mut cmd,
                    &game_assets,
                    &resource_assets,
                    tr.translation + Vec3::new(100., 100., 0.),
                    UnitClass::Worker,
                )
            } else if index == 1 {
                spawn_unit_with_class(
                    &mut cmd,
                    &game_assets,
                    &resource_assets,
                    tr.translation + Vec3::new(100., 100., 0.),
                    UnitClass::Sworder,
                );
            } else if index == 2 {
                spawn_unit_with_class(
                    &mut cmd,
                    &game_assets,
                    &resource_assets,
                    tr.translation + Vec3::new(100., 100., 0.),
                    UnitClass::Ranged,
                );
            }
        }
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
    mut resource_assets: ResMut<ResourceAssets>,
) {
    game_assets.player_sprite = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/player/blob.png"),
        Vec2::new(146., 124.),
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
    spawn_bloodrock_node(&mut cmd, &resource_assets, Vec3::new(100., 100., 0.));

    cmd.spawn_bundle(SpriteSheetBundle {
        texture_atlas: game_assets.player_sprite.clone(),
        transform: Transform::from_scale(Vec3::new(1., 1., 1.)),
        ..Default::default()
    })
    .insert(PlayerController)
    .insert(EnemySpawner {
        time_between_spawns: Timer::from_seconds(10., true),
        distance_from_spawn_point: 400.,
    })
    .insert(ZOffset { offset: -50. });

    spawn_unit_with_class(
        &mut cmd,
        &game_assets,
        &resource_assets,
        Vec3::new(180., 10., 0.),
        UnitClass::Sworder,
    );
    spawn_unit_with_class(
        &mut cmd,
        &game_assets,
        &resource_assets,
        Vec3::new(0., 200., 0.),
        UnitClass::Worker,
    );
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
) {
    let starter_colors = [
        Color::rgb(0., 1., 0.),
        Color::rgb(0., 0., 1.),
        Color::rgb(1., 0., 0.),
    ];
    let mut rng = rand::thread_rng();

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
        })
        .insert(Velocity(100.))
        .insert(WorkerColor {
            color: starter_colors[rng.gen_range(0..starter_colors.len())],
        })
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
                        0., 30., 0.1,
                    )),
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
        })
        .id();
    let mut health_comp = Health {
        current_health: 10.,
        max_health: 10.,
    };
    change_class(entity_id, cmd, class, &mut health_comp);
    cmd.entity(entity_id).insert(health_comp);
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets::default())
            .insert_resource(ResourceAssets::default())
            .insert_resource(MaxSupplyAmount(10))
            .insert_resource(BloodrockAmount(100))
            .add_startup_system(setup_game)
            .add_system_to_stage(CoreStage::PostUpdate, z_sorter_system)
            .add_system(player_controll_system)
            .add_system(spawn_workers_system)
            .add_system(avoid_others_system)
            .add_system(animate_on_movement_system)
            .add_system(harvester_logic_system)
            .add_system(harvester_carrying_something_system)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                camera_follow_player_system,
            );
    }
}
