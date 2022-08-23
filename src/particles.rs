use std::{f32::consts::TAU, time::Duration};

use bevy::prelude::*;
use rand::Rng;

#[derive(Default, Clone, Copy, Component)]
pub struct Velocity(pub Vec3);
#[derive(Default, Clone, Copy, Component)]
pub struct Acceleration(pub Vec3);
#[derive(Default, Clone, Component)]
pub struct Lifetime(pub Timer);

pub trait Lerp {
    fn lerp(&self, rhs: &Self, t: f32) -> Self;
}

pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

impl Lerp for Color {
    fn lerp(&self, rhs: &Self, t: f32) -> Self {
        let lhs = self.as_rgba();
        let rhs = rhs.as_rgba();

        Color::RgbaLinear {
            red: lerp_f32(lhs.r(), rhs.r(), t),
            green: lerp_f32(lhs.g(), rhs.g(), t),
            blue: lerp_f32(lhs.b(), rhs.b(), t),
            alpha: lerp_f32(lhs.a(), rhs.a(), t),
        }
    }
}

#[derive(Clone, Component)]
pub enum Easing {
    None,
    Linear,
    QuartOutInverted,
    QuartOut,
    OutElastic,
    PulsateInOutCubic,
    PulsateInOutCubicShifted,
}

impl Easing {
    pub fn get_easing(&self, percent: f32) -> f32 {
        match self {
            Easing::None => 1.,
            Easing::Linear => percent,
            Easing::QuartOutInverted => 1. - ezing::quart_out(percent),
            Easing::QuartOut => ezing::quart_out(percent),
            Easing::OutElastic => ezing::elastic_out(percent),
            Easing::PulsateInOutCubic => {
                if percent < 0.5 {
                    ezing::cubic_inout(percent * 2.)
                } else {
                    1. - ezing::circ_in((percent - 0.5) * 2.)
                }
            }
            Easing::PulsateInOutCubicShifted => {
                return Easing::PulsateInOutCubic.get_easing(percent) + 0.1;
            }
        }
    }
}

impl Default for Easing {
    fn default() -> Self {
        Easing::None
    }
}

#[derive(Default, Clone, Component)]
pub struct SizeOverLifetime {
    pub start_size: Vec3,
    pub end_size: Vec3,
    pub easing: Easing,
}

#[derive(Default, Clone, Component)]
pub struct SpriteColorOverLifetime {
    pub start_color: Color,
    pub end_color: Color,
    pub easing: Easing,
}

#[derive(Bundle, Default)]
struct ParticleBundle {
    vel: Velocity,
    acc: Acceleration,
    lifetime: Lifetime,
    transform: Transform,
    global_transform: GlobalTransform,
    easing: Easing,
    size_over_lifetime: SizeOverLifetime,
}

#[derive(Component)]
pub struct SpawnTimer(pub Timer);
#[derive(Clone, Component)]
pub enum ParticleBody {
    SpriteSheet {
        sheet_bundle: SpriteSheetBundle,
        color_over_lifetime: Option<SpriteColorOverLifetime>,
    },
}

#[derive(Component)]
pub struct SpawnConfig {
    pub min_count: usize,
    pub max_count: usize,
    pub min_life: Duration,
    pub max_life: Duration,
    pub min_vel: f32,
    pub max_vel: f32,
    pub min_acc: f32,
    pub max_acc: f32,
    pub easing: Easing,
    pub size_over_lifetime: SizeOverLifetime,
    pub bodies: Vec<ParticleBody>,
}

#[derive(Bundle)]
pub struct EmitterBundle {
    pub lifetime: Lifetime,
    pub spawn_timer: SpawnTimer,
    pub config: SpawnConfig,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

fn update_emitters_system(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(&mut SpawnTimer, &SpawnConfig, &GlobalTransform)>,
) {
    let mut rng = rand::thread_rng();
    let delta = time.delta();
    q.for_each_mut(move |(mut t, conf, tr)| {
        t.0.tick(delta);
        if t.0.just_finished() {
            let min = conf.min_count as f32;
            let max = conf.max_count as f32;
            let t = rng.gen_range(0.0..1.0) + 0.1;

            let count = min + (max - min) * t;

            let mut i = 0.0;
            while i < count {
                i += 1.0;

                let rad = rng.gen_range(0.0..TAU);

                if conf.bodies.is_empty() {
                    continue;
                }
                let body_ind = rng.gen_range(0..conf.bodies.len());
                let mut cmd;
                let mut trans;
                match &conf.bodies[body_ind] {
                    ParticleBody::SpriteSheet {
                        sheet_bundle,
                        color_over_lifetime,
                    } => {
                        cmd = commands.spawn_bundle(sheet_bundle.clone());
                        trans = sheet_bundle.transform;
                        if let Some(color_over_lifetime) =
                            color_over_lifetime.clone()
                        {
                            cmd.insert(color_over_lifetime);
                        }
                    }
                }

                trans.translation = tr.translation();
                let rot = Quat::from_rotation_z(rad);
                let vel = rot.mul_vec3(Vec3::X);

                cmd.insert_bundle(ParticleBundle {
                    vel: Velocity(
                        vel * rng.gen_range(conf.min_vel..conf.max_vel),
                    ),
                    acc: Acceleration(
                        vel * rng.gen_range(conf.min_acc..conf.max_acc),
                    ),
                    lifetime: Lifetime(Timer::new(
                        rng.gen_range(conf.min_life..conf.max_life),
                        false,
                    )),
                    transform: trans,
                    global_transform: Default::default(),
                    easing: conf.easing.clone(),
                    size_over_lifetime: conf.size_over_lifetime.clone(),
                });
            }
        }
    });
}

fn particle_scaling_system(
    mut q: Query<(&mut Transform, &Lifetime, &SizeOverLifetime)>,
) {
    q.par_for_each_mut(128, |(mut tr, lt, sol)| {
        tr.scale = sol
            .start_size
            .lerp(sol.end_size, sol.easing.get_easing(lt.0.percent()));
    });
}

fn particle_movement_system(
    time: Res<Time>,
    mut q: ParamSet<(
        Query<(&mut Velocity, &Acceleration)>,
        Query<(&Velocity, &mut Transform, &Lifetime, &Easing)>,
    )>,
) {
    // initially I forgot about deltatime (facepalm)
    // we multiply by 60 because all velocities / accelerations were tuned for 60 fps
    let dt = time.delta_seconds();
    q.p0().par_for_each_mut(128, move |(mut vel, acc)| {
        vel.0 += acc.0 * dt * 60.;
    });
    q.p1().par_for_each_mut(128, move |(vel, mut tr, lt, es)| {
        tr.translation += vel.0 * es.get_easing(lt.0.percent()) * dt * 60.;
    });
}

fn update_lifetimes_system(time: Res<Time>, mut q: Query<&mut Lifetime>) {
    let dt = time.delta();
    q.par_for_each_mut(128, move |mut lt| {
        lt.0.tick(dt);
    });
}

/// Fades out particle child sprites
fn fade_sprites_system(
    mut q_particle: Query<(
        &Lifetime,
        &mut TextureAtlasSprite,
        &SpriteColorOverLifetime,
    )>,
) {
    q_particle.par_for_each_mut(128, move |(lt, mut sprite, color)| {
        let c = color
            .start_color
            .lerp(&color.end_color, color.easing.get_easing(lt.0.percent()));
        sprite.color = c;
    });
}

fn kill_system(mut cmd: Commands, q: Query<(Entity, &Lifetime)>) {
    q.iter()
        .filter_map(|(e, lt)| (!lt.0.repeating() && lt.0.finished()).then(|| e))
        .for_each(|entity| {
            cmd.entity(entity).despawn_recursive();
        });
}

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(particle_movement_system)
            .add_system(update_lifetimes_system)
            .add_system(particle_scaling_system)
            .add_system(fade_sprites_system)
            .add_system(kill_system)
            .add_system(update_emitters_system);
    }
}
