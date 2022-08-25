use bevy::prelude::*;

use crate::{
    combat::{AttackState, AttackType, CombatComponent},
    enemy_logic::BasicEnemyLogic,
    game::{GameAssets, Harvester, PlayerController},
    get_children_recursive,
    health::Health,
    interaction::{MouseFollow, Selected},
    GameTime,
};

pub struct WorkerLogicPlugin;

#[derive(Component)]
pub struct WorkerHead;
#[derive(Component)]
pub struct WorkerEye;
#[derive(Component)]
pub struct UnitFollowPlayer;

#[derive(Component)]
pub struct TankComponent;
#[derive(Component)]
pub struct HealerComponent;

#[derive(Component, Clone, Copy, PartialEq)]
pub enum UnitClass {
    Worker,
    Ranged,
    Sworder,
    Tank,
    Piker,
    Healer,
}
#[derive(Component, Clone, Copy, PartialEq)]
pub enum UnitSize {
    Small,
    Medium,
    Huge,
}

#[derive(Component)]
pub struct CanEatWorker {
    pub entity_to_eat: Option<Entity>,
}

fn get_next_size(size: UnitSize) -> UnitSize {
    match size {
        UnitSize::Small => {
            return UnitSize::Medium;
        }
        UnitSize::Medium => {
            return UnitSize::Huge;
        }
        UnitSize::Huge => {
            return UnitSize::Huge;
        }
    }
}

pub fn merge_units(
    unit_alpha: (UnitClass, UnitSize),
    unit_beta: (UnitClass, UnitSize),
) -> (UnitClass, UnitSize) {
    let unit_alpha_size = unit_alpha.1;
    let unit_beta_size = unit_beta.1;
    let unit_alpha_class = unit_alpha.0;
    let unit_beta_class = unit_beta.0;
    let mut return_var = unit_alpha.clone();

    if unit_alpha_class == unit_beta_class && unit_alpha_size == unit_beta_size
    {
        return_var.1 = get_next_size(return_var.1);
    }

    match unit_alpha_class {
        UnitClass::Worker => match unit_beta_class {
            UnitClass::Ranged => {
                return_var.0 = UnitClass::Healer;
            }
            UnitClass::Sworder => {
                return_var.0 = UnitClass::Tank;
            }
            _ => {}
        },
        UnitClass::Sworder => match unit_beta_class {
            UnitClass::Worker => {
                return_var.0 = UnitClass::Tank;
            }
            UnitClass::Ranged => {
                return_var.0 = UnitClass::Piker;
            }
            _ => {}
        },
        UnitClass::Ranged => match unit_beta_class {
            UnitClass::Sworder => {
                return_var.0 = UnitClass::Piker;
            }
            UnitClass::Worker => {
                return_var.0 = UnitClass::Healer;
            }
            _ => {}
        },

        _ => {}
    }
    return return_var;
}

fn get_index_from_unit_class(class: UnitClass) -> usize {
    match class {
        UnitClass::Worker => 0,
        UnitClass::Ranged => 1,
        UnitClass::Sworder => 2,
        UnitClass::Tank => 3,
        UnitClass::Piker => 4,
        UnitClass::Healer => 5,
    }
}

fn change_sprite_based_on_class_system(
    mut workers: Query<(&UnitClass, &UnitSize, &mut TextureAtlasSprite)>,
) {
    let mut sprite_index;
    for (class, size, mut sprite) in workers.iter_mut() {
        sprite_index = get_index_from_unit_class(*class);
        match size {
            UnitSize::Small => {
                sprite_index += 0;
            }
            UnitSize::Medium => {
                sprite_index += 6;
            }
            UnitSize::Huge => {
                sprite_index += 12;
            }
        }
        sprite.index = sprite_index;
    }
}

fn set_stats_based_on_class_and_size_system(
    units: Query<(Entity, &UnitClass, &UnitSize, &Transform)>,
    mut combat_comps: Query<&mut CombatComponent>,
    mut harvester_comps: Query<&mut Harvester>,
    //mut healer_comps: Query<&mut HealerComponent>,
    //mut tank_comps: Query<&TankComponent>,
) {
    for (e, class, size, tr) in units.iter() {
        if let Ok(mut combat_comp) = combat_comps.get_mut(e) {
            let mut damage = 1.;
            let mut time_to_attack = 1.;
            match class {
                UnitClass::Piker => {
                    damage = 1.;
                    time_to_attack = 1.5;
                }
                UnitClass::Sworder => {
                    damage = 1.5;
                    time_to_attack = 0.75;
                }
                UnitClass::Ranged => {
                    damage = 0.5;
                    time_to_attack = 1.;
                }
                _ => {}
            }
            match size {
                UnitSize::Small => {}
                UnitSize::Medium => {
                    damage *= 2.;
                    time_to_attack /= 1.5;
                }
                UnitSize::Huge => {
                    damage *= 3.;
                    time_to_attack /= 3.;
                }
            }
            damage += 1. - tr.scale.x;
            time_to_attack -= (1. - tr.scale.x) / 10.;
            combat_comp.damage = damage;
            if combat_comp.time_between_attacks.duration()
                != bevy::utils::Duration::from_secs_f32(time_to_attack)
            {
                combat_comp.time_between_attacks.set_duration(
                    bevy::utils::Duration::from_secs_f32(time_to_attack),
                );
            }
        }
        if let Ok(mut harvester_comp) = harvester_comps.get_mut(e) {
            let mut harvest_speed = 1.;
            let mut max_carryable_resource: usize = 3;
            match size {
                UnitSize::Small => {}
                UnitSize::Medium => {
                    harvest_speed = 0.80;
                    max_carryable_resource = 4;
                }
                UnitSize::Huge => {
                    harvest_speed = 0.5;
                    max_carryable_resource = 5;
                }
            }
            max_carryable_resource += ((1. - tr.scale.x) * 5.) as usize;
            harvest_speed -= (1. - tr.scale.x) / 10.;

            harvester_comp.max_carryable_resource = max_carryable_resource;

            if harvester_comp.harvest_speed.duration()
                != bevy::utils::Duration::from_secs_f32(harvest_speed)
            {
                harvester_comp.harvest_speed.set_duration(
                    bevy::utils::Duration::from_secs_f32(harvest_speed),
                );
            }
        }
    }
}

pub fn change_class(
    entity: Entity,
    cmd: &mut Commands,
    class: UnitClass,
    health: &mut Health,
) {
    let mut entity_commands = cmd.entity(entity);
    entity_commands.remove::<CombatComponent>();
    entity_commands.remove::<Harvester>();
    entity_commands.remove::<TankComponent>();
    entity_commands.remove::<HealerComponent>();

    match class {
        UnitClass::Worker => {
            entity_commands.insert(Harvester {
                target_node: None,
                harvest_speed: Timer::from_seconds(1., false),
                max_carryable_resource: 3,
                current_carried_resource: 0,
            });
        }
        UnitClass::Sworder => {
            entity_commands.insert(CombatComponent {
                target: None,
                damage: 1.,
                time_between_attacks: Timer::from_seconds(1., true),
                attack_range: 70.,
                attack_type: AttackType::Melee,
                attack_state: AttackState::NotAttacking,
            });
        }
        UnitClass::Piker => {
            entity_commands.insert(CombatComponent {
                target: None,
                damage: 1.5,
                time_between_attacks: Timer::from_seconds(1.5, true),
                attack_range: 100.,
                attack_type: AttackType::Melee,
                attack_state: AttackState::NotAttacking,
            });
        }
        UnitClass::Ranged => {
            entity_commands.insert(CombatComponent {
                target: None,
                damage: 0.5,
                time_between_attacks: Timer::from_seconds(1., true),
                attack_range: 300.,
                attack_type: AttackType::Ranged,
                attack_state: AttackState::NotAttacking,
            });
        }
        UnitClass::Tank => {
            entity_commands
                .insert(CombatComponent {
                    target: None,
                    damage: 0.5,
                    time_between_attacks: Timer::from_seconds(3., true),
                    attack_range: 70.,
                    attack_type: AttackType::Melee,
                    attack_state: AttackState::NotAttacking,
                })
                .insert(TankComponent);
            health.max_health *= 2.;
            health.current_health *= 2.;
        }
        UnitClass::Healer => {
            entity_commands.insert(HealerComponent);
        }
    }
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
                        < ally_combat.attack_range.max(200.) + 100.
                    {
                        ally_combat.target = Some(enemy);
                        break;
                    }
                }
            }
        }
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
    time: Res<GameTime>,
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
            .add_system(ally_targetting_logic_system)
            .add_system(set_stats_based_on_class_and_size_system)
            .add_system(change_sprite_based_on_class_system);
    }
}
