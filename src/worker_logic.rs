use bevy::prelude::*;

use crate::{
    combat::CombatComponent,
    enemy_logic::BasicEnemyLogic,
    game::{GameAssets, PlayerController},
    get_children_recursive,
    interaction::{MouseFollow, Selected},
};

pub struct WorkerLogicPlugin;

#[derive(Component)]
pub struct WorkerHead;
#[derive(Component)]
pub struct WorkerEye;
#[derive(Component)]
pub struct UnitFollowPlayer;

#[derive(Component)]
pub struct WorkerColor {
    pub color: Color,
}

#[derive(Component)]
pub struct CanEatWorker {
    pub entity_to_eat: Option<Entity>,
}

fn ally_targetting_logic_system(
    mut allys: Query<(&mut CombatComponent, Entity), With<UnitFollowPlayer>>,
    enemies: Query<Entity, With<BasicEnemyLogic>>,
    transforms: Query<&GlobalTransform>,
) {
    //TODO: find closest enemy that the worker can attack
    for (mut ally_combat, e) in allys.iter_mut() {
        if ally_combat.target == None {
            let mut ally_pos = Vec2::ZERO; //TODO: better error handling
            if let Ok(ally_tr) = transforms.get(e) {
                ally_pos = ally_tr.translation().truncate();
            }
            for enemy in enemies.iter() {
                if let Ok(enemy_tr) = transforms.get(enemy) {
                    if (ally_pos - enemy_tr.translation().truncate()).length()
                        < 300.
                    {
                        ally_combat.target = Some(enemy);
                        break;
                    }
                }
            }
        }
    }
}

fn color_worker_body_system(
    children: Query<&Children>,
    mut worker_color: Query<
        (&WorkerColor, &mut TextureAtlasSprite, Entity),
        Without<WorkerHead>,
    >,
    mut worker_head: Query<&mut TextureAtlasSprite, With<WorkerHead>>,
) {
    for (color, mut sprite, e) in worker_color.iter_mut() {
        sprite.color = color.color;

        get_children_recursive(e, &children, &mut |child| {
            if let Ok(mut head_sprite) = worker_head.get_mut(child) {
                head_sprite.color = color.color;
            }
        });
    }
}

fn change_head_system(
    mut cmd: Commands,
    children: Query<&Children>,
    heads: Query<&Handle<TextureAtlas>, With<WorkerHead>>,
    mut eyes: Query<&mut TextureAtlasSprite, With<WorkerEye>>,
    can_eats: Query<(Entity, &CanEatWorker)>,
    assets: Res<GameAssets>,
) {
    for (e, eater) in can_eats.iter() {
        get_children_recursive(e, &children, &mut |child| {
            if let Ok(texture_handle) = heads.get(child) {
                if let Some(_) = eater.entity_to_eat {
                    if *texture_handle != assets.worker_head_eating {
                        cmd.entity(child)
                            .insert(assets.worker_head_eating.clone());
                    }
                } else {
                    if *texture_handle != assets.worker_head {
                        cmd.entity(child).insert(assets.worker_head.clone());
                    }
                }
            }

            if let Ok(mut texture_atlas_sprite) = eyes.get_mut(child) {
                if let Some(_) = eater.entity_to_eat {
                    texture_atlas_sprite.color = Color::rgba(0., 0., 0., 0.);
                } else {
                    texture_atlas_sprite.color = Color::rgba(1., 1., 1., 1.);
                }
            }
        });
    }
}

fn eat_other_worker_system(
    selected: Res<Selected>,
    global_tr: Query<&GlobalTransform>,
    mut can_eat_workers: Query<(&mut CanEatWorker, Entity)>,
) {
    if let Some(selected_e) = selected.0 {
        if let Ok(selected_tr) = global_tr.get(selected_e) {
            let selected_entity_pos = selected_tr.translation();

            for (mut can_eat, e) in can_eat_workers.iter_mut() {
                can_eat.entity_to_eat = None;
                if e == selected_e {
                    continue;
                }
                if let Ok(can_eat_tr) = global_tr.get(e) {
                    if (can_eat_tr.translation().truncate()
                        - selected_entity_pos.truncate())
                    .length()
                        < 50.
                    {
                        can_eat.entity_to_eat = Some(selected_e);
                    }
                }
            }
        }
    } else {
        for (mut can_eat, _) in can_eat_workers.iter_mut() {
            can_eat.entity_to_eat = None;
        }
    }
}

fn player_follower_system(
    mut q_player_followers: Query<
        (&mut Transform, &CombatComponent),
        (With<UnitFollowPlayer>, Without<MouseFollow>),
    >,
    player: Query<&GlobalTransform, With<PlayerController>>,
    time: Res<Time>,
) {
    let player_tr = player.single();
    for (mut tr, cc) in q_player_followers.iter_mut() {
        if matches!(cc.target, None) {
            let direction_vector = player_tr.translation() - tr.translation;

            let direction_vector = direction_vector.truncate();
            if direction_vector.length() < 200. {
                continue;
            }
            let direction_vector = direction_vector.normalize();

            tr.translation +=
                direction_vector.extend(0.) * time.delta_seconds() * 150.;
        }
    }
}

impl Plugin for WorkerLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_follower_system)
            .add_system(eat_other_worker_system)
            .add_system(change_head_system)
            .add_system(color_worker_body_system)
            .add_system(ally_targetting_logic_system);
    }
}
