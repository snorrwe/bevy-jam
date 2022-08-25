use crate::{
    combat::CombatComponent,
    easing::Easing,
    game::{spawn_bloodrock_node, GameAssets, ResourceAssets},
    interaction::{Hovered, Selected},
    particles,
};
use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;

#[derive(Component)]
pub struct Health {
    pub max_health: f32,
    pub current_health: f32,
}

pub struct HealthChangedEvent {
    pub target: Entity,
    pub amount: f32,
}
#[derive(Component)]
pub struct SpawnResourceNodeOnDeath {
    pub chance: f32,
}

pub struct DestroyEntity(pub Entity);

fn destroyer_system(
    mut cmd: Commands,
    resource_assets: Res<ResourceAssets>,
    mut destroy_event_reader: EventReader<DestroyEntity>,
    mut combat_comps: Query<&mut CombatComponent>,
    transforms: Query<&GlobalTransform>,
    spawn_on_death: Query<&SpawnResourceNodeOnDeath>,
    mut selected: ResMut<Selected>,
    mut hovered: ResMut<Hovered>,
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
        if let Ok(e) = transforms.get(event.0) {
            if let Ok(spawn) = spawn_on_death.get(event.0) {
                let mut rng = rand::thread_rng();
                if rng.gen_range(0.0..100.0) < spawn.chance {
                    spawn_bloodrock_node(
                        &mut cmd,
                        &resource_assets,
                        e.translation(),
                    );
                }
            }
        }
        if let Some(e) = selected.0 {
            if e == event.0 {
                selected.0 = None;
            }
        }
        if let Some(e) = hovered.0 {
            if e == event.0 {
                hovered.0 = None;
            }
        }
        cmd.entity(event.0).despawn_recursive();
    }
}

fn health_change_system(
    mut destroy_event_writer: EventWriter<DestroyEntity>,
    mut health_changed_events: EventReader<HealthChangedEvent>,
    mut health_query: Query<(&mut Health, Entity, &GlobalTransform)>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    for event in health_changed_events.iter() {
        if let Ok((mut health, _, tr)) = health_query.get_mut(event.target) {
            health.current_health += event.amount;
            if event.amount < 0. {
                spawn_health_particles(
                    &mut commands,
                    game_assets.circle_sprite.clone(),
                    tr.translation(),
                );
            }

            health.current_health =
                health.current_health.clamp(0., health.max_health);
        }
    }

    for (health, e, _) in health_query.iter() {
        if health.current_health <= 0. {
            destroy_event_writer.send(DestroyEntity(e));
        }
    }
}

fn spawn_health_particles(
    commands: &mut Commands,
    texture: Handle<TextureAtlas>,
    pos: Vec3,
) {
    let body = particles::ParticleBody::SpriteSheet {
        sheet_bundle: SpriteSheetBundle {
            texture_atlas: texture.clone(),
            sprite: TextureAtlasSprite {
                color: Color::RED,
                ..Default::default()
            },
            transform: Transform::from_scale(Vec3::splat(0.)),
            ..Default::default()
        },
        color_over_lifetime: Some(particles::SpriteColorOverLifetime {
            start_color: Color::RED,
            end_color: Color::ORANGE_RED,
            easing: Easing::Linear,
        }),
    };
    commands.spawn_bundle(particles::EmitterBundle {
        lifetime: particles::Lifetime(Timer::new(
            Duration::from_millis(500),
            false,
        )),
        spawn_timer: particles::SpawnTimer(Timer::new(
            Duration::from_millis(40),
            false,
        )),
        config: particles::SpawnConfig {
            min_count: 6,
            max_count: 10,
            min_life: Duration::from_millis(400),
            max_life: Duration::from_millis(600),
            min_vel: -4.0,
            max_vel: 4.0,
            min_acc: -0.05,
            max_acc: -0.03,
            easing: Easing::OutElastic,
            size_over_lifetime: particles::SizeOverLifetime {
                start_size: Vec3::splat(0.6),
                end_size: Vec3::splat(0.1),
                easing: Easing::QuartOut,
            },
            bodies: vec![body],
        },
        transform: Transform::from_translation(pos),
        global_transform: Default::default(),
    });
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
