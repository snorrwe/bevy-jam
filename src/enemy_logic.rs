use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    collision,
    combat::{AttackType, CombatComponent},
    game::{DontSortZ, UnitType, Velocity},
    health::{hp_material, Health, SpawnResourceNodeOnDeath},
    worker_logic::{
        HealerComponent, HealingState, TankComponent, UnitFollowPlayer,
    },
    GameTime,
};
use rand::Rng;
#[derive(Default)]
pub struct EnemyAssets {
    basic_enemy_sprite: Handle<TextureAtlas>,
}
pub struct EnemyLogicPlugin;

#[derive(Component)]
pub struct BasicEnemyLogic;

#[derive(Component)]
pub struct EnemySpawner {
    pub time_between_spawns: Timer,
    pub distance_from_spawn_point: f32,
}
#[derive(Clone)]
enum EnemyTypesToSpawn {
    Thrash,
    Ranged,
    Sworder,
    Piker,
    Armored,
    Healer,
    Boss1,
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
) {
    let mut spawn_enemy = |health: Health,
                           combat_compo: Option<&CombatComponent>,
                           color: Color|
     -> Entity {
        spawn_regular_enemy(
            &mut cmd,
            &enemy_assets,
            pos,
            &mut *hp_assets,
            &mut *mesh_assets,
            &health,
            combat_compo,
            color,
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
                    damage: 0.5,
                    time_between_attacks: Timer::from_seconds(1., true),
                    attack_range: 80.,
                    piercing: 0.,
                    ..Default::default()
                }),
                Color::WHITE,
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
                Color::GREEN,
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
                Color::RED,
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
                Color::ORANGE,
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
                Color::BLUE,
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
                Color::YELLOW,
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
        EnemyTypesToSpawn::Boss1 => {}
    }
}

fn enemy_targetting_logic_system(
    mut enemies: Query<&mut CombatComponent, With<BasicEnemyLogic>>,
    allys: Query<Entity, With<UnitFollowPlayer>>,
) {
    //TODO: find closest ally that the enemy can attack
    for mut enemy_combat in enemies.iter_mut() {
        if enemy_combat.target == None {
            enemy_combat.target = allys.iter().next();
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
            );
        }
    }
}

fn spawn_regular_enemy(
    cmd: &mut Commands,
    game_assets: &EnemyAssets,
    pos: Vec3,
    hp_assets: &mut Assets<hp_material::HpMaterial>,
    mesh_assets: &mut Assets<Mesh>,
    health: &Health,
    combat_comp: Option<&CombatComponent>,
    color: Color,
) -> Entity {
    let entity_id = cmd
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_assets.basic_enemy_sprite.clone(),
            sprite: TextureAtlasSprite {
                color: color,
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
        .insert(SpawnResourceNodeOnDeath { chance: 10. })
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
    enemy_assets.basic_enemy_sprite =
        texture_atlases.add(TextureAtlas::from_grid(
            asset_server.load("sprites/enemies/basic_enemy_sprite.png"),
            Vec2::new(60., 98.),
            1,
            1,
        ));
}

impl Plugin for EnemyLogicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemyAssets::default())
            .add_startup_system(setup_system)
            .add_system(enemy_spawner_system)
            .add_system(enemy_targetting_logic_system);
    }
}
