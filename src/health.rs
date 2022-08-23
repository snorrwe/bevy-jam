use crate::combat::CombatComponent;
use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub max_health: f32,
    pub current_health: f32,
}

pub struct HealthChangedEvent {
    pub target: Entity,
    pub amount: f32,
}

pub struct DestroyEntity(Entity);

fn destroyer_system(
    mut cmd: Commands,
    mut destroy_event_reader: EventReader<DestroyEntity>,
    mut combat_comps: Query<&mut CombatComponent>,
) {
    for event in destroy_event_reader.iter() {
        //Clear out targets
        for mut combat_comp in combat_comps.iter_mut() {
            if let Some(e) = combat_comp.target {
                if e == event.0 {
                    combat_comp.target = None;
                }
            }
        }

        cmd.entity(event.0).despawn_recursive();
    }
}

fn health_change_system(
    mut destroy_event_writer: EventWriter<DestroyEntity>,
    mut health_changed_events: EventReader<HealthChangedEvent>,
    mut health_query: Query<(&mut Health, Entity)>,
) {
    for event in health_changed_events.iter() {
        if let Ok((mut health, _)) = health_query.get_mut(event.target) {
            health.current_health += event.amount;
            health.current_health =
                health.current_health.clamp(0., health.max_health);
        }
    }

    for (health, e) in health_query.iter() {
        if health.current_health <= 0. {
            destroy_event_writer.send(DestroyEntity(e));
        }
    }
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HealthChangedEvent>()
            .add_event::<DestroyEntity>()
            .add_system(health_change_system)
            .add_system_to_stage(CoreStage::PostUpdate, destroyer_system);
    }
}
