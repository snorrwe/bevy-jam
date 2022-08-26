use crate::{
    animation::{Animation, RotationAnimation},
    easing::Easing,
    enemy_logic::BasicEnemyLogic,
    game::{AvoidOthers, GameAssets, UnitType, Velocity},
    health::{Health, HealthChangedEvent},
    worker_logic::{
        HealerComponent, HealingState, TankComponent, UnitFollowPlayer,
    },
    GameTime,
};
use bevy::prelude::*;

pub enum AttackType {
    Melee,
    Ranged,
    None,
}

#[derive(Component)]
pub struct Projectile {
    target: Entity,
    damage: f32,
    speed: f32,
}

//Helps sync up animation with damage dealing
pub enum AttackState {
    NotAttacking,
    AttackStart { timer: Timer },
    AttackMiddle { timer: Timer },
    AttackEnd { timer: Timer },
}

#[derive(Component)]
pub struct CombatComponent {
    pub target: Option<Entity>,
    pub damage: f32,
    pub time_between_attacks: Timer,
    pub attack_range: f32,
    pub attack_type: AttackType,
    pub attack_state: AttackState,
    pub target_type: UnitType,
}

fn healer_heal_component(
    mut cmd: Commands,
    mut healers: Query<(
        &mut Transform,
        &mut HealerComponent,
        &Velocity,
        Entity,
    )>,
    allys: Query<Entity, With<UnitFollowPlayer>>,
    enemies: Query<Entity, With<BasicEnemyLogic>>,
    healths: Query<&Health>,
    global_transform: Query<&GlobalTransform>,
    time: Res<GameTime>,
    game_assets: Res<GameAssets>,
) {
    for (mut tr, mut healer_comp, vel, healer_entity) in healers.iter_mut() {
        if let Some(target_entity) = healer_comp.target {
            if let Ok(health_comp) = healths.get(target_entity) {
                if health_comp.current_health >= health_comp.max_health {
                    healer_comp.target = None;

                    cmd.entity(healer_entity).insert(RotationAnimation(
                        Animation::<Quat> {
                            from: tr.rotation,
                            to: Quat::from_rotation_z(0.),
                            timer: Timer::from_seconds(0.2, false),
                            easing: Easing::QuartOut,
                        },
                    ));
                }
            }

            healer_comp.time_between_heals.tick(time.delta());
            if let Ok(global_tr) = global_transform.get(target_entity) {
                let pointing_vec = global_tr.translation().truncate()
                    - tr.translation.truncate();
                let distance = pointing_vec.length();
                let dir = pointing_vec.extend(0.).normalize();

                if distance < healer_comp.range {
                    match &mut healer_comp.state {
                        HealingState::Idle => {
                            if healer_comp.time_between_heals.finished() {
                                healer_comp.time_between_heals.reset();
                                healer_comp.state = HealingState::Casting(
                                    Timer::from_seconds(0.8, false),
                                );
                                cmd.entity(healer_entity).insert(
                                    RotationAnimation(Animation::<Quat> {
                                        from: Quat::from_rotation_z(-0.2),
                                        to: Quat::from_rotation_z(0.2),
                                        timer: Timer::from_seconds(0.2, true),
                                        easing: Easing::PulsateInOutCubic,
                                    }),
                                );
                            }
                        }
                        HealingState::Casting(ref mut timer) => {
                            timer.tick(time.delta());
                            if timer.finished() {
                                let mut proj_transform =
                                    Transform::from_translation(tr.translation);
                                proj_transform.scale = Vec3::splat(0.3);
                                cmd.spawn_bundle(SpriteSheetBundle {
                                    texture_atlas: game_assets
                                        .circle_sprite
                                        .clone(),
                                    sprite: TextureAtlasSprite {
                                        color: Color::GREEN,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .insert(Projectile {
                                    speed: 500.,
                                    damage: -healer_comp.heal_amount,
                                    target: target_entity,
                                })
                                .insert(proj_transform);

                                cmd.entity(healer_entity).insert(
                                    RotationAnimation(Animation::<Quat> {
                                        from: tr.rotation,
                                        to: Quat::from_rotation_z(0.),
                                        timer: Timer::from_seconds(0.2, false),
                                        easing: Easing::QuartOut,
                                    }),
                                );
                                healer_comp.state = HealingState::Idle;
                            }
                        }
                    }
                } else if matches!(healer_comp.state, HealingState::Idle) {
                    tr.translation += dir * vel.0 * time.delta_seconds();
                }
            }
        } else {
            let mut least_healthy_ally: (Option<Entity>, f32) =
                (None, 99999999.);
            match healer_comp.target_type {
                UnitType::Ally => {
                    for e in allys.iter() {
                        if let Ok(h) = healths.get(e) {
                            if h.current_health < h.max_health
                                && h.current_health < least_healthy_ally.1
                            {
                                least_healthy_ally.1 = h.current_health;
                                least_healthy_ally.0 = Some(e);
                            }
                        }
                    }
                }
                UnitType::Enemy => {
                    for e in enemies.iter() {
                        if let Ok(h) = healths.get(e) {
                            if h.current_health < h.max_health
                                && h.current_health < least_healthy_ally.1
                            {
                                least_healthy_ally.1 = h.current_health;
                                least_healthy_ally.0 = Some(e);
                            }
                        }
                    }
                }
            }
            if least_healthy_ally.0.is_some() {}
            healer_comp.target = least_healthy_ally.0;
        }
    }
}

fn tank_aggro_component(
    mut tanks: Query<
        (&GlobalTransform, &mut TankComponent, Entity),
        Without<BasicEnemyLogic>,
    >,
    mut enemies: Query<
        (&mut CombatComponent, &GlobalTransform),
        With<BasicEnemyLogic>,
    >,
    mut allys: Query<
        (&mut CombatComponent, &GlobalTransform),
        Without<BasicEnemyLogic>,
    >,
    time: Res<GameTime>,
) {
    for (tank_tr, mut tank_comp, e) in tanks.iter_mut() {
        tank_comp.time_between_taunts.tick(time.delta());
        if tank_comp.time_between_taunts.just_finished() {
            tank_comp.time_between_taunts.reset();

            //Spawn particles, sound
            match tank_comp.target_type {
                UnitType::Ally => {
                    for (mut ally_combat_comp, ally_tr) in allys.iter_mut() {
                        if (tank_tr.translation().truncate()
                            - ally_tr.translation().truncate())
                        .length()
                            < 300.
                        {
                            ally_combat_comp.target = Some(e);
                        }
                    }
                }
                UnitType::Enemy => {
                    for (mut enemy_combat_comp, enemy_tr) in enemies.iter_mut()
                    {
                        if (tank_tr.translation().truncate()
                            - enemy_tr.translation().truncate())
                        .length()
                            < 300.
                        {
                            enemy_combat_comp.target = Some(e);
                        }
                    }
                }
            }
        }
    }
}

fn projectile_flying_system(
    mut cmd: Commands,
    mut projectiles: Query<(&mut Transform, &Projectile, Entity)>,
    global_trs: Query<&GlobalTransform>,
    mut health_changed_event_writer: EventWriter<HealthChangedEvent>,
    time: Res<GameTime>,
) {
    for (mut proj_tr, proj, e) in projectiles.iter_mut() {
        if let Ok(target_tr) = global_trs.get(proj.target) {
            let dir_vector = target_tr.translation().truncate()
                - proj_tr.translation.truncate();
            let distance = dir_vector.length();
            let direction = dir_vector.normalize();

            if distance < 10. {
                health_changed_event_writer.send(HealthChangedEvent {
                    target: proj.target,
                    amount: -proj.damage,
                });
                cmd.entity(e).despawn_recursive();
            } else {
                proj_tr.translation +=
                    direction.extend(0.) * proj.speed * time.delta_seconds();
            }
        } else {
            cmd.entity(e).despawn_recursive();
        }
    }
}

fn combat_system(
    time: Res<GameTime>,
    mut combatant: Query<(
        &mut CombatComponent,
        &mut Transform,
        &Velocity,
        Entity,
    )>,
    mut avoid_others: Query<&mut AvoidOthers>,
    transform_query: Query<&GlobalTransform>,
    mut health_changed_event_writer: EventWriter<HealthChangedEvent>,
    game_assets: Res<GameAssets>,
    mut cmd: Commands,
) {
    for (mut combat_comp, mut tr, vel, e) in combatant.iter_mut() {
        if let Ok(mut avoid_other) = avoid_others.get_mut(e) {
            avoid_other.is_enabled = !combat_comp.target.is_some();
        }
        if !matches!(combat_comp.attack_state, AttackState::NotAttacking) {
            continue;
        }
        combat_comp.time_between_attacks.tick(time.delta());
        let mut own_global_pos = Vec2::ZERO;
        if let Ok(global_tr) = transform_query.get(e) {
            own_global_pos = global_tr.translation().truncate();
        }

        if let Some(target) = combat_comp.target {
            if let Ok(target_tr) = transform_query.get(target) {
                let target_pos = target_tr.translation().truncate();

                let distance = (target_pos - own_global_pos).length();
                let direction = (target_pos - own_global_pos).normalize();

                if combat_comp.attack_range >= distance
                    && combat_comp.time_between_attacks.finished()
                {
                    cmd.entity(e).insert(RotationAnimation(
                        Animation::<Quat> {
                            from: Quat::from_rotation_z(0.),
                            to: Quat::from_rotation_z(-0.5),
                            timer: Timer::from_seconds(0.2, false),
                            easing: Easing::QuartOut,
                        },
                    ));

                    combat_comp.attack_state = AttackState::AttackStart {
                        timer: Timer::from_seconds(0.3, false),
                    };
                } else if combat_comp.attack_range < distance {
                    tr.translation +=
                        direction.extend(0.) * time.delta_seconds() * vel.0;
                }
            }
        }
    }

    for (mut combat_comp, tr, _, e) in combatant.iter_mut() {
        if let Some(target) = combat_comp.target {
            match &mut combat_comp.attack_state {
                AttackState::AttackStart { ref mut timer } => {
                    timer.tick(time.delta());
                    if timer.finished() {
                        cmd.entity(e).insert(RotationAnimation(Animation::<
                            Quat,
                        > {
                            from: Quat::from_rotation_z(-0.5),
                            to: Quat::from_rotation_z(0.7),
                            timer: Timer::from_seconds(0.1, false),
                            easing: Easing::QuartOut,
                        }));

                        combat_comp.attack_state = AttackState::AttackMiddle {
                            timer: Timer::from_seconds(0.3, false),
                        };

                        match combat_comp.attack_type {
                            AttackType::Ranged => {
                                let mut proj_transform =
                                    Transform::from_translation(tr.translation);
                                proj_transform.scale = Vec3::splat(0.3);
                                cmd.spawn_bundle(SpriteSheetBundle {
                                    texture_atlas: game_assets
                                        .circle_sprite
                                        .clone(),
                                    ..Default::default()
                                })
                                .insert(Projectile {
                                    speed: 500.,
                                    damage: combat_comp.damage,
                                    target: target,
                                })
                                .insert(proj_transform);
                            }
                            AttackType::Melee => {
                                health_changed_event_writer.send(
                                    HealthChangedEvent {
                                        amount: -combat_comp.damage,
                                        target: target,
                                    },
                                );
                            }
                            AttackType::None => {}
                        }
                    }
                }
                AttackState::AttackMiddle { ref mut timer } => {
                    timer.tick(time.delta());
                    if timer.finished() {
                        cmd.entity(e).insert(RotationAnimation(Animation::<
                            Quat,
                        > {
                            from: Quat::from_rotation_z(0.7),
                            to: Quat::from_rotation_z(0.),
                            timer: Timer::from_seconds(0.3, false),
                            easing: Easing::QuartOut,
                        }));

                        combat_comp.attack_state = AttackState::AttackEnd {
                            timer: Timer::from_seconds(0.3, false),
                        };
                    }
                }
                AttackState::AttackEnd { ref mut timer } => {
                    timer.tick(time.delta());
                    if timer.finished() {
                        combat_comp.attack_state = AttackState::NotAttacking;
                    }
                }
                AttackState::NotAttacking => {}
            }
        }
    }
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(combat_system)
            .add_system(projectile_flying_system)
            .add_system(tank_aggro_component)
            .add_system(healer_heal_component);
    }
}
