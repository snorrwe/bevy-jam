pub mod hp_material;

use crate::{
    animation::{Animation, RotationAnimation},
    audio::{AudioAssets, PlayAudioEventPositional},
    combat::{AttackState, CombatComponent},
    easing::Easing,
    game::{spawn_bloodrock_node, BloodrockAmount, GameAssets, ResourceAssets},
    interaction::{Hovered, Selected},
    particles,
    worker_logic::HealerComponent,
};
use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;

#[derive(Component, Copy, Clone)]
pub struct Health {
    pub max_health: f32,
    pub current_health: f32,
    pub armor: f32,
}

pub struct HealthChangedEvent {
    pub target: Entity,
    pub amount: f32,
    pub piercing: f32,
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
    mut combat_comps: Query<(&mut CombatComponent, Entity)>,
    mut healer_comps: Query<(&mut HealerComponent, Entity)>,
    transforms: Query<&GlobalTransform>,
    local_transforms: Query<&Transform>,
    spawn_on_death: Query<&SpawnResourceNodeOnDeath>,
    mut selected: ResMut<Selected>,
    mut hovered: ResMut<Hovered>,
    mut amount_of_bloodrock: ResMut<BloodrockAmount>,
) {
    for event in destroy_event_reader.iter() {
        //Clear out targets
        for (mut combat_comp, combat_entity) in combat_comps.iter_mut() {
            if let Some(e) = combat_comp.target {
                if e == event.0 {
                    combat_comp.target = None;
                    combat_comp.attack_state = AttackState::NotAttacking;
                    if let Ok(transform) = local_transforms.get(combat_entity) {
                        cmd.entity(combat_entity).insert(RotationAnimation(
                            Animation::<Quat> {
                                from: transform.rotation,
                                to: Quat::from_rotation_z(0.),
                                timer: Timer::from_seconds(0.2, false),
                                easing: Easing::QuartOut,
                            },
                        ));
                    }
                }
            }
        }
        //TODO: maybe make a single target component, so we dont have to add this for every new component with a target
        for (mut healer_comp, healer_entity) in healer_comps.iter_mut() {
            if let Some(e) = healer_comp.target {
                if e == event.0 {
                    healer_comp.target = None;
                    if let Ok(transform) = local_transforms.get(healer_entity) {
                        cmd.entity(healer_entity).insert(RotationAnimation(
                            Animation::<Quat> {
                                from: transform.rotation,
                                to: Quat::from_rotation_z(0.),
                                timer: Timer::from_seconds(0.2, false),
                                easing: Easing::QuartOut,
                            },
                        ));
                    }
                }
            }
        }
        if let Ok(e) = transforms.get(event.0) {
            if let Ok(spawn) = spawn_on_death.get(event.0) {
                amount_of_bloodrock.0 += 1;
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
    audio_assets: Res<AudioAssets>,
    mut send_audio_event: EventWriter<PlayAudioEventPositional>,
) {
    for event in health_changed_events.iter() {
        if let Ok((mut health, _, tr)) = health_query.get_mut(event.target) {
            let modifier = (1. - health.armor + event.piercing).clamp(0.01, 1.);

            let mut amount = event.amount;
            if event.amount < 0. {
                amount *= modifier;
                spawn_health_particles(
                    &mut commands,
                    game_assets.circle_sprite.clone(),
                    tr.translation(),
                );
                send_audio_event.send(PlayAudioEventPositional {
                    sound: audio_assets.getting_damaged.clone(),
                    position: tr.translation(),
                });
            } else {
                spawn_healing_particles(
                    &mut commands,
                    game_assets.circle_sprite.clone(),
                    tr.translation(),
                );
                send_audio_event.send(PlayAudioEventPositional {
                    sound: audio_assets.getting_healed.clone(),
                    position: tr.translation(),
                });
            }
            health.current_health += amount;
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

fn spawn_healing_particles(
    commands: &mut Commands,
    texture: Handle<TextureAtlas>,
    pos: Vec3,
) {
    let body = particles::ParticleBody::SpriteSheet {
        sheet_bundle: SpriteSheetBundle {
            texture_atlas: texture.clone(),
            sprite: TextureAtlasSprite {
                color: Color::GREEN,
                ..Default::default()
            },
            transform: Transform::from_scale(Vec3::splat(0.)),
            ..Default::default()
        },
        color_over_lifetime: Some(particles::SpriteColorOverLifetime {
            start_color: Color::GREEN,
            end_color: Color::DARK_GREEN,
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
            min_count: 3,
            max_count: 6,
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
        app.add_plugin(
            bevy::sprite::Material2dPlugin::<hp_material::HpMaterial>::default(
            ),
        )
        .add_event::<HealthChangedEvent>()
        .add_event::<DestroyEntity>()
        .add_system(health_change_system)
        .add_system(hp_material::update_hp_materials)
        .add_system(hp_material::update_hp_bar_transform)
        .add_asset::<hp_material::HpMaterial>()
        .add_system_to_stage(CoreStage::PostUpdate, destroyer_system);
    }
}
