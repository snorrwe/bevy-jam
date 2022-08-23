use crate::{game::Velocity, health::HealthChangedEvent, GameTime};
use bevy::prelude::*;

pub enum AttackType {
    Melee,
    Ranged,
    None,
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

fn combat_system(
    time: Res<GameTime>,
    mut combatant: Query<(
        &mut CombatComponent,
        &mut Transform,
        &Velocity,
        Entity,
    )>,
    transform_query: Query<&GlobalTransform>,
    mut health_changed_event_writer: EventWriter<HealthChangedEvent>,
) {
    for (mut combat_comp, mut tr, vel, e) in combatant.iter_mut() {
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
                    combat_comp.attack_state = AttackState::AttackStart {
                        timer: Timer::from_seconds(0.3, false),
                    };
                } else {
                    tr.translation +=
                        direction.extend(0.) * time.delta_seconds() * vel.0;
                }
            }
        }
    }

    for (mut combat_comp, _, _, _) in combatant.iter_mut() {
        if let Some(target) = combat_comp.target {
            match &mut combat_comp.attack_state {
                AttackState::AttackStart { ref mut timer } => {
                    timer.tick(time.delta());
                    if timer.finished() {
                        combat_comp.attack_state = AttackState::AttackMiddle {
                            timer: Timer::from_seconds(0.3, false),
                        };
                    }
                }
                AttackState::AttackMiddle { ref mut timer } => {
                    timer.tick(time.delta());
                    if timer.finished() {
                        combat_comp.attack_state = AttackState::AttackEnd {
                            timer: Timer::from_seconds(0.3, false),
                        };
                        health_changed_event_writer.send(HealthChangedEvent {
                            amount: -combat_comp.damage,
                            target: target,
                        });
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
        app.add_system(combat_system);
    }
}
