use crate::{
    animation::{Animation, RotationAnimation},
    easing::Easing,
    game::{AvoidOthers, GameAssets, Velocity},
    health::HealthChangedEvent,
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
            .add_system(projectile_flying_system);
    }
}
