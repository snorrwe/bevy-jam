use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    collision,
    combat::{AttackType, CombatComponent},
    game::{
        spawn_bloodrock_node, AvoidOthers, DontSortZ,
        MovementAnimationController, PlayerController, ResourceAssets,
        UnitType, Velocity, ZOffset,
    },
    health::{hp_material, Health, SpawnResourceNodeOnDeath},
    ui::{EndGameManager, EndGameState},
    worker_logic::{
        HealerComponent, HealingState, TankComponent, UnitFollowPlayer,
    },
    GameTime, SceneState,
};
use rand::Rng;
#[derive(Default)]
pub struct EnemyAssets {
    enemies: Handle<TextureAtlas>,
    boss1: Handle<TextureAtlas>,
    boss2: Handle<TextureAtlas>,
}
pub struct EnemyLogicPlugin;

#[derive(Component)]
pub struct BasicEnemyLogic;

#[derive(Component)]
pub struct EnemySpawner {
    pub time_between_spawns: Timer,
    pub distance_from_spawn_point: f32,
}
#[derive(Clone, Copy)]
enum EnemyTypesToSpawn {
    Thrash,
    Ranged,
    Sworder,
    Piker,
    Armored,
    Healer,
    BloodrockNode,
    Boss1,
    Boss2,
}

#[derive(Clone)]
pub struct Wave {
    spawn_data: Vec<(Vec<EnemyTypesToSpawn>, Vec3)>,
    time_to_spawn_after_last_wave: Timer,
}
#[derive(Clone)]
pub struct Level {
    pub waves: Vec<Wave>,
    pub current_wave_index: usize,
}
#[derive(Deref, DerefMut)]
pub struct LevelManager {
    pub current_level: Level,
}

pub fn get_test_level() -> Level {
    return Level {
        waves: vec![
            Wave {
                spawn_data: vec![(
                    vec![EnemyTypesToSpawn::Thrash, EnemyTypesToSpawn::Thrash],
                    Vec3::new(0., 1300., 0.),
                )],
                time_to_spawn_after_last_wave: Timer::from_seconds(3., false),
            },
            Wave {
                spawn_data: vec![
                    (vec![EnemyTypesToSpawn::Thrash], Vec3::new(0., 1200., 0.)),
                    (
                        vec![EnemyTypesToSpawn::Ranged],
                        Vec3::new(-1300., 100., 0.),
                    ),
                    (
                        vec![EnemyTypesToSpawn::Thrash],
                        Vec3::new(1300., -100., 0.),
                    ),
                ],
                time_to_spawn_after_last_wave: Timer::from_seconds(15., false),
            },
            Wave {
                spawn_data: vec![
                    (
                        vec![
                            EnemyTypesToSpawn::Thrash,
                            EnemyTypesToSpawn::Thrash,
                            EnemyTypesToSpawn::Thrash,
                            EnemyTypesToSpawn::Thrash,
                        ],
                        Vec3::new(0., 1200., 0.),
                    ),
                    (
                        vec![EnemyTypesToSpawn::BloodrockNode],
                        Vec3::new(-500., 100., 0.),
                    ),
                ],
                time_to_spawn_after_last_wave: Timer::from_seconds(15., false),
            },
            Wave {
                spawn_data: vec![(
                    vec![
                        EnemyTypesToSpawn::Ranged,
                        EnemyTypesToSpawn::Thrash,
                        EnemyTypesToSpawn::Thrash,
                    ],
                    Vec3::new(0., 1200., 0.),
                )],
                time_to_spawn_after_last_wave: Timer::from_seconds(25., false),
            },
            Wave {
                spawn_data: vec![(
                    vec![
                        EnemyTypesToSpawn::Ranged,
                        EnemyTypesToSpawn::Armored,
                        EnemyTypesToSpawn::Thrash,
                        EnemyTypesToSpawn::Thrash,
                    ],
                    Vec3::new(0., 1200., 0.),
                )],
                time_to_spawn_after_last_wave: Timer::from_seconds(25., false),
            },
            Wave {
                spawn_data: vec![
                    (
                        vec![
                            EnemyTypesToSpawn::Armored,
                            EnemyTypesToSpawn::Healer,
                        ],
                        Vec3::new(0., 1200., 0.),
                    ),
                    (
                        vec![
                            EnemyTypesToSpawn::Ranged,
                            EnemyTypesToSpawn::Ranged,
                            EnemyTypesToSpawn::Ranged,
                        ],
                        Vec3::new(0., -1200., 0.),
                    ),
                ],
                time_to_spawn_after_last_wave: Timer::from_seconds(30., false),
            },
            Wave {
                spawn_data: vec![(
                    vec![
                        EnemyTypesToSpawn::Ranged,
                        EnemyTypesToSpawn::Boss1,
                        EnemyTypesToSpawn::Ranged,
                    ],
                    Vec3::new(0., 1200., 0.),
                )],
                time_to_spawn_after_last_wave: Timer::from_seconds(30., false),
            },
            Wave {
                spawn_data: vec![
                    (
                        vec![
                            EnemyTypesToSpawn::Thrash,
                            EnemyTypesToSpawn::Thrash,
                            EnemyTypesToSpawn::Thrash,
                            EnemyTypesToSpawn::Thrash,
                        ],
                        Vec3::new(-500., 1200., 0.),
                    ),
                    (
                        vec![
                            EnemyTypesToSpawn::Thrash,
                            EnemyTypesToSpawn::Thrash,
                            EnemyTypesToSpawn::Thrash,
                            EnemyTypesToSpawn::Thrash,
                        ],
                        Vec3::new(500., -1200., 0.),
                    ),
                    (
                        vec![EnemyTypesToSpawn::BloodrockNode],
                        Vec3::new(500., -100., 0.),
                    ),
                ],
                time_to_spawn_after_last_wave: Timer::from_seconds(30., false),
            },
            Wave {
                spawn_data: vec![(
                    vec![
                        EnemyTypesToSpawn::Piker,
                        EnemyTypesToSpawn::Piker,
                        EnemyTypesToSpawn::Piker,
                        EnemyTypesToSpawn::Piker,
                    ],
                    Vec3::new(1500., 0., 0.),
                )],
                time_to_spawn_after_last_wave: Timer::from_seconds(15., false),
            },
            Wave {
                spawn_data: vec![
                    (
                        vec![
                            EnemyTypesToSpawn::Armored,
                            EnemyTypesToSpawn::Ranged,
                            EnemyTypesToSpawn::Ranged,
                            EnemyTypesToSpawn::Ranged,
                        ],
                        Vec3::new(-1500., 0., 0.),
                    ),
                    (
                        vec![EnemyTypesToSpawn::BloodrockNode],
                        Vec3::new(500., 100., 0.),
                    ),
                ],
                time_to_spawn_after_last_wave: Timer::from_seconds(15., false),
            },
            Wave {
                spawn_data: vec![(
                    vec![
                        EnemyTypesToSpawn::Thrash,
                        EnemyTypesToSpawn::Thrash,
                        EnemyTypesToSpawn::Ranged,
                    ],
                    Vec3::new(0., -1500., 0.),
                )],
                time_to_spawn_after_last_wave: Timer::from_seconds(15., false),
            },
            Wave {
                spawn_data: vec![
                    (
                        vec![
                            EnemyTypesToSpawn::Armored,
                            EnemyTypesToSpawn::Armored,
                        ],
                        Vec3::new(1200., 0., 0.),
                    ),
                    (
                        vec![
                            EnemyTypesToSpawn::Armored,
                            EnemyTypesToSpawn::Armored,
                        ],
                        Vec3::new(-1200., 0., 0.),
                    ),
                ],
                time_to_spawn_after_last_wave: Timer::from_seconds(20., false),
            },
            Wave {
                spawn_data: vec![
                    (
                        vec![
                            EnemyTypesToSpawn::Ranged,
                            EnemyTypesToSpawn::Ranged,
                            EnemyTypesToSpawn::Ranged,
                        ],
                        Vec3::new(1200., 0., 0.),
                    ),
                    (
                        vec![
                            EnemyTypesToSpawn::Ranged,
                            EnemyTypesToSpawn::Ranged,
                            EnemyTypesToSpawn::Ranged,
                        ],
                        Vec3::new(-1200., 0., 0.),
                    ),
                ],
                time_to_spawn_after_last_wave: Timer::from_seconds(15., false),
            },
            Wave {
                spawn_data: vec![
                    (
                        vec![
                            EnemyTypesToSpawn::Thrash,
                            EnemyTypesToSpawn::Thrash,
                            EnemyTypesToSpawn::Thrash,
                        ],
                        Vec3::new(1200., 0., 0.),
                    ),
                    (
                        vec![
                            EnemyTypesToSpawn::Thrash,
                            EnemyTypesToSpawn::Thrash,
                            EnemyTypesToSpawn::Thrash,
                        ],
                        Vec3::new(-1200., 0., 0.),
                    ),
                ],
                time_to_spawn_after_last_wave: Timer::from_seconds(5., false),
            },
            Wave {
                spawn_data: vec![(
                    vec![
                        EnemyTypesToSpawn::Healer,
                        EnemyTypesToSpawn::Healer,
                        EnemyTypesToSpawn::Boss2,
                    ],
                    Vec3::new(-1200., 1200., 0.),
                )],
                time_to_spawn_after_last_wave: Timer::from_seconds(30., false),
            },
        ],
        current_wave_index: 0,
    };
}

fn move_enemies_to_arena(
    mut enemies: Query<(&mut Transform, &Velocity), With<BasicEnemyLogic>>,
    time: Res<GameTime>,
) {
    for (mut tr, vel) in enemies.iter_mut() {
        if tr.translation.x < -1000.
            || tr.translation.x > 700.
            || tr.translation.y < -650.
            || tr.translation.y > 650.
        {
            let pos = tr.translation;
            tr.translation += (Vec3::new(0., 0., 0.) - pos)
                .truncate()
                .normalize()
                .extend(0.)
                * vel.0
                * 1.2
                * time.delta_seconds();
        }
    }
}

fn level_progresser_system(
    game_time: Res<GameTime>,
    mut level_manager: ResMut<LevelManager>,
    enemy_assets: Res<EnemyAssets>,
    mut cmd: Commands,
    enemies: Query<Entity, With<BasicEnemyLogic>>,
    mut hp_assets: ResMut<Assets<hp_material::HpMaterial>>,
    // FIXME: reuse the same mesh?
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut end_game_state: ResMut<EndGameManager>,
    resource_assets: Res<ResourceAssets>,
) {
    //TODO: win level when last wave is spawned && no enemies left!
    if level_manager.current_level.waves.len()
        <= level_manager.current_level.current_wave_index
    {
        if enemies.iter().len() < 1 {
            end_game_state.state = EndGameState::Win;
        }
        return;
    } else {
        let current_wave_index = level_manager.current_level.current_wave_index;
        let current_wave =
            &mut level_manager.current_level.waves[current_wave_index];
        current_wave
            .time_to_spawn_after_last_wave
            .tick(game_time.delta());
        if current_wave.time_to_spawn_after_last_wave.finished() {
            let mut rng = rand::thread_rng();
            for enemies in current_wave.spawn_data.iter_mut() {
                let location = enemies.1;
                for enemy in enemies.0.iter() {
                    let spawn_location = location
                        + Vec3::new(
                            rng.gen_range(-1.0..=1.0),
                            rng.gen_range(-1.0..=1.0),
                            0.,
                        )
                        .normalize()
                            * 500.;
                    spawn_enemy_based_on_type(
                        *enemy,
                        &mut cmd,
                        &enemy_assets,
                        spawn_location,
                        &mut *hp_assets,
                        &mut *mesh_assets,
                        &resource_assets,
                    );
                }
            }

            level_manager.current_level.current_wave_index += 1;
        }
    }
}

fn get_random_enemy() -> EnemyTypesToSpawn {
    let enemy_types = [
        EnemyTypesToSpawn::Thrash,
        EnemyTypesToSpawn::Ranged,
        EnemyTypesToSpawn::Sworder,
    ];
    let mut rng = rand::thread_rng();
    return enemy_types[rng.gen_range(0..enemy_types.len())].clone();
}

fn spawn_enemy_based_on_type(
    enemy_type: EnemyTypesToSpawn,
    mut cmd: &mut Commands,
    enemy_assets: &EnemyAssets,
    pos: Vec3,
    hp_assets: &mut Assets<hp_material::HpMaterial>,
    mesh_assets: &mut Assets<Mesh>,
    resource_assets: &ResourceAssets,
) {
    let mut spawn_enemy = |health: Health,
                           combat_compo: Option<&CombatComponent>,
                           index: usize|
     -> Entity {
        spawn_regular_enemy(
            &mut cmd,
            pos,
            &mut *hp_assets,
            &mut *mesh_assets,
            &health,
            combat_compo,
            &enemy_assets.enemies,
            index,
        )
    };

    match enemy_type {
        EnemyTypesToSpawn::Thrash => {
            spawn_enemy(
                Health {
                    current_health: 3.,
                    max_health: 3.,
                    armor: 0.,
                },
                Some(&CombatComponent {
                    target_type: UnitType::Ally,
                    attack_type: AttackType::Melee,
                    damage: 0.6,
                    time_between_attacks: Timer::from_seconds(1., true),
                    attack_range: 80.,
                    piercing: 0.,
                    ..Default::default()
                }),
                0,
            );
        }
        EnemyTypesToSpawn::Ranged => {
            spawn_enemy(
                Health {
                    current_health: 5.,
                    max_health: 5.,
                    armor: 0.,
                },
                Some(&CombatComponent {
                    target_type: UnitType::Ally,
                    attack_type: AttackType::Ranged,
                    damage: 1.,
                    time_between_attacks: Timer::from_seconds(1., true),
                    attack_range: 200.,
                    piercing: 0.,
                    ..Default::default()
                }),
                2,
            );
        }
        EnemyTypesToSpawn::Sworder => {
            spawn_enemy(
                Health {
                    current_health: 7.,
                    max_health: 7.,
                    armor: 0.,
                },
                Some(&CombatComponent {
                    target_type: UnitType::Ally,
                    attack_type: AttackType::Melee,
                    damage: 1.3,
                    time_between_attacks: Timer::from_seconds(1., true),
                    attack_range: 80.,
                    piercing: 0.,
                    ..Default::default()
                }),
                1,
            );
        }
        EnemyTypesToSpawn::Piker => {
            spawn_enemy(
                Health {
                    current_health: 7.,
                    max_health: 7.,
                    armor: 0.,
                },
                Some(&CombatComponent {
                    target_type: UnitType::Ally,
                    attack_type: AttackType::Melee,
                    damage: 1.25,
                    time_between_attacks: Timer::from_seconds(1.3, true),
                    attack_range: 120.,
                    piercing: 0.75,
                    ..Default::default()
                }),
                3,
            );
        }
        EnemyTypesToSpawn::Armored => {
            let entity = spawn_enemy(
                Health {
                    current_health: 15.,
                    max_health: 15.,
                    armor: 0.8,
                },
                Some(&CombatComponent {
                    target_type: UnitType::Ally,
                    attack_type: AttackType::Melee,
                    damage: 0.2,
                    time_between_attacks: Timer::from_seconds(2., true),
                    attack_range: 80.,
                    piercing: 0.,
                    ..Default::default()
                }),
                4,
            );
            cmd.entity(entity).insert(TankComponent {
                time_between_taunts: Timer::from_seconds(3., true),
                target_type: UnitType::Ally,
            });
        }
        EnemyTypesToSpawn::Healer => {
            let entity = spawn_enemy(
                Health {
                    current_health: 7.,
                    max_health: 7.,
                    armor: 0.,
                },
                None,
                5,
            );
            cmd.entity(entity).insert(HealerComponent {
                heal_amount: 0.2,
                range: 200.,
                time_between_heals: Timer::from_seconds(2., true),
                target: None,
                state: HealingState::Idle,
                target_type: UnitType::Enemy,
            });
        }
        EnemyTypesToSpawn::Boss1 => {
            cmd.spawn_bundle(SpriteSheetBundle {
                texture_atlas: enemy_assets.boss1.clone(),
                ..Default::default()
            })
            .insert(ZOffset { offset: -100. })
            .insert_bundle(collision::AABBBundle {
                desc: collision::AABBDescriptor {
                    radius: Vec3::splat(50.),
                },
                filter: collision::CollisionFilter {
                    self_layers: collision::CollisionType::WORKER,
                    collisions_mask:
                        collision::CollisionType::WORKER_COLLISIONS,
                },
                ..Default::default()
            })
            .insert(Health {
                current_health: 30.,
                max_health: 30.,
                armor: 0.,
            })
            .insert(CombatComponent {
                target_type: UnitType::Ally,
                attack_type: AttackType::Melee,
                damage: 1.,
                time_between_attacks: Timer::from_seconds(1., true),
                attack_range: 80.,
                piercing: 0.,
                ..Default::default()
            })
            .insert(Transform::from_translation(pos))
            .insert(Velocity(150.))
            .insert(BasicEnemyLogic)
            .insert(SpawnResourceNodeOnDeath { chance: 0. })
            .insert(AvoidOthers { is_enabled: true })
            .insert(MovementAnimationController {
                is_moving: false,
                last_frame_pos: pos,
                time_to_stop_moving: Timer::from_seconds(0.3, false),
            })
            .with_children(|cmd| {
                cmd.spawn_bundle(MaterialMesh2dBundle {
                    mesh: bevy::sprite::Mesh2dHandle(mesh_assets.add(
                        Mesh::from(shape::Quad {
                            size: Vec2::new(100.0, 10.0),
                            flip: false,
                        }),
                    )),
                    material: hp_assets.add(hp_material::HpMaterial {
                        color_empty: Color::RED,
                        color_full: Color::ORANGE_RED,
                        hp: 50.0,
                        hp_max: 100.0,
                    }),
                    transform: Transform::from_translation(
                        Vec3::Z * 200.0 + Vec3::Y * 60.0,
                    ),
                    ..Default::default()
                })
                .insert(DontSortZ);
            });
        }
        EnemyTypesToSpawn::Boss2 => {
            cmd.spawn_bundle(SpriteSheetBundle {
                texture_atlas: enemy_assets.boss2.clone(),
                ..Default::default()
            })
            .insert(ZOffset { offset: -100. })
            .insert_bundle(collision::AABBBundle {
                desc: collision::AABBDescriptor {
                    radius: Vec3::splat(50.),
                },
                filter: collision::CollisionFilter {
                    self_layers: collision::CollisionType::WORKER,
                    collisions_mask:
                        collision::CollisionType::WORKER_COLLISIONS,
                },
                ..Default::default()
            })
            .insert(Health {
                current_health: 30.,
                max_health: 30.,
                armor: 0.,
            })
            .insert(CombatComponent {
                target_type: UnitType::Ally,
                attack_type: AttackType::Ranged,
                damage: 0.5,
                time_between_attacks: Timer::from_seconds(0.5, true),
                attack_range: 200.,
                piercing: 0.,
                ..Default::default()
            })
            .insert(Transform::from_translation(pos))
            .insert(Velocity(150.))
            .insert(BasicEnemyLogic)
            .insert(SpawnResourceNodeOnDeath { chance: 0. })
            .insert(AvoidOthers { is_enabled: true })
            .insert(MovementAnimationController {
                is_moving: false,
                last_frame_pos: pos,
                time_to_stop_moving: Timer::from_seconds(0.3, false),
            })
            .with_children(|cmd| {
                cmd.spawn_bundle(MaterialMesh2dBundle {
                    mesh: bevy::sprite::Mesh2dHandle(mesh_assets.add(
                        Mesh::from(shape::Quad {
                            size: Vec2::new(100.0, 10.0),
                            flip: false,
                        }),
                    )),
                    material: hp_assets.add(hp_material::HpMaterial {
                        color_empty: Color::RED,
                        color_full: Color::ORANGE_RED,
                        hp: 50.0,
                        hp_max: 100.0,
                    }),
                    transform: Transform::from_translation(
                        Vec3::Z * 200.0 + Vec3::Y * 60.0,
                    ),
                    ..Default::default()
                })
                .insert(DontSortZ);
            });
        }
        EnemyTypesToSpawn::BloodrockNode => {
            spawn_bloodrock_node(&mut cmd, &resource_assets, pos)
        }
    }
}

fn enemy_targetting_logic_system(
    mut enemies: Query<
        (&mut CombatComponent, &GlobalTransform),
        With<BasicEnemyLogic>,
    >,
    allys: Query<
        (Entity, &GlobalTransform),
        (With<UnitFollowPlayer>, Without<BasicEnemyLogic>),
    >,
    player: Query<Entity, With<PlayerController>>,
) {
    //TODO: find closest ally that the enemy can attack
    for (mut enemy_combat, enemy_tr) in enemies.iter_mut() {
        let mut closest_target = (None, 99999.);
        for (ally_entity, ally_tr) in allys.iter() {
            if (enemy_tr.translation().truncate()
                - ally_tr.translation().truncate())
            .length()
                < closest_target.1
            {
                closest_target = (
                    Some(ally_entity),
                    (enemy_tr.translation().truncate()
                        - ally_tr.translation().truncate())
                    .length(),
                );
            }
        }
        if enemy_combat.target == None {
            enemy_combat.target = closest_target.0;
        }

        if enemy_combat.target == None {
            for p in player.iter() {
                enemy_combat.target = Some(p);
            }
        }
    }
}

fn enemy_spawner_system(
    time: Res<GameTime>,
    enemy_assets: Res<EnemyAssets>,
    mut cmd: Commands,
    mut enemy_spawners: Query<(&mut EnemySpawner, &GlobalTransform)>,
    mut hp_assets: ResMut<Assets<hp_material::HpMaterial>>,
    // FIXME: reuse the same mesh?
    mut mesh_assets: ResMut<Assets<Mesh>>,
    resource_assets: Res<ResourceAssets>,
) {
    let mut rng = rand::thread_rng();
    for (mut enemy_spawner, global_tr) in enemy_spawners.iter_mut() {
        enemy_spawner.time_between_spawns.tick(time.delta());
        if enemy_spawner.time_between_spawns.finished() {
            enemy_spawner.time_between_spawns.reset();

            spawn_enemy_based_on_type(
                get_random_enemy(),
                &mut cmd,
                &enemy_assets,
                global_tr.translation()
                    + (Vec3::new(
                        rng.gen_range(-1.0..=1.0),
                        rng.gen_range(-1.0..=1.0),
                        0.,
                    )
                    .normalize()
                        * enemy_spawner.distance_from_spawn_point),
                &mut *hp_assets,
                &mut *mesh_assets,
                &resource_assets,
            );
        }
    }
}

fn spawn_regular_enemy(
    cmd: &mut Commands,
    pos: Vec3,
    hp_assets: &mut Assets<hp_material::HpMaterial>,
    mesh_assets: &mut Assets<Mesh>,
    health: &Health,
    combat_comp: Option<&CombatComponent>,
    texture_atlas: &Handle<TextureAtlas>,
    sprite_index: usize,
) -> Entity {
    let entity_id = cmd
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas.clone(),
            sprite: TextureAtlasSprite {
                index: sprite_index,
                ..Default::default()
            },
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
        .insert(*health)
        .insert(Transform::from_translation(pos))
        .insert(Velocity(150.))
        .insert(BasicEnemyLogic)
        .insert(SpawnResourceNodeOnDeath { chance: 0. })
        .insert(AvoidOthers { is_enabled: true })
        .insert(MovementAnimationController {
            is_moving: false,
            last_frame_pos: pos,
            time_to_stop_moving: Timer::from_seconds(0.3, false),
        })
        .with_children(|cmd| {
            cmd.spawn_bundle(MaterialMesh2dBundle {
                mesh: bevy::sprite::Mesh2dHandle(mesh_assets.add(Mesh::from(
                    shape::Quad {
                        size: Vec2::new(60.0, 10.0),
                        flip: false,
                    },
                ))),
                material: hp_assets.add(hp_material::HpMaterial {
                    color_empty: Color::RED,
                    color_full: Color::ORANGE_RED,
                    hp: 50.0,
                    hp_max: 100.0,
                }),
                transform: Transform::from_translation(
                    Vec3::Z * 200.0 + Vec3::Y * 60.0,
                ),
                ..Default::default()
            })
            .insert(DontSortZ);
        })
        .id();

    if let Some(combat_com) = combat_comp {
        cmd.entity(entity_id).insert(combat_com.clone());
    }
    return entity_id;
}

fn setup_system(
    mut enemy_assets: ResMut<EnemyAssets>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    enemy_assets.enemies = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/enemies/enemies.png"),
        Vec2::new(122., 115.),
        6,
        1,
    ));
    enemy_assets.boss1 = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/enemies/boss1.png"),
        Vec2::new(157., 244.),
        1,
        1,
    ));
    enemy_assets.boss2 = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/enemies/boss2.png"),
        Vec2::new(185., 225.),
        1,
        1,
    ));
}

impl Plugin for EnemyLogicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemyAssets::default())
            .insert_resource(LevelManager {
                current_level: get_test_level(),
            })
            .add_startup_system(setup_system)
            .add_system_set(
                SystemSet::on_update(SceneState::InGame)
                    .with_system(enemy_spawner_system)
                    .with_system(enemy_targetting_logic_system)
                    .with_system(level_progresser_system)
                    .with_system(move_enemies_to_arena),
            );
    }
}
