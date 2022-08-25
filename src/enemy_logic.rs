use bevy::prelude::*;

use crate::{
    collision,
    combat::{AttackState, AttackType, CombatComponent},
    game::Velocity,
    health::{Health, SpawnResourceNodeOnDeath},
    worker_logic::UnitFollowPlayer,
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
) {
    let mut rng = rand::thread_rng();
    for (mut enemy_spawner, global_tr) in enemy_spawners.iter_mut() {
        enemy_spawner.time_between_spawns.tick(time.delta());
        if enemy_spawner.time_between_spawns.finished() {
            enemy_spawner.time_between_spawns.reset();
            spawn_regular_enemy(
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
            )
        }
    }
}

fn spawn_regular_enemy(
    cmd: &mut Commands,
    game_assets: &EnemyAssets,
    pos: Vec3,
) {
    cmd.spawn_bundle(SpriteSheetBundle {
        texture_atlas: game_assets.basic_enemy_sprite.clone(),
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
    .insert(Health {
        current_health: 5.,
        max_health: 5.,
    })
    .insert(CombatComponent {
        target: None,
        damage: 1.,
        time_between_attacks: Timer::from_seconds(1., true),
        attack_range: 80.,
        attack_type: AttackType::Melee,
        attack_state: AttackState::NotAttacking,
    })
    .insert(Transform::from_translation(pos))
    .insert(Velocity(150.))
    .insert(BasicEnemyLogic)
    .insert(SpawnResourceNodeOnDeath { chance: 10. });
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
