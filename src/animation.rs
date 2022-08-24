use std::time::Duration;

use bevy::prelude::*;

use crate::{lerp::Lerp, particles::Easing, GameTime};

#[derive(Clone)]
pub struct Animation<T> {
    pub from: T,
    pub to: T,
    pub timer: Timer,
    pub easing: Easing,
}

impl<T: Lerp> Animation<T> {
    pub fn tick(&mut self, dt: Duration) -> T {
        self.timer.tick(dt);
        let t = self.timer.percent();
        let t = self.easing.get_easing(t);
        self.from.lerp(&self.to, t)
    }
}

#[derive(Clone, Component)]
pub struct RotationAnimation(pub Animation<Quat>);

pub fn update_rotations_animations(
    time: Res<GameTime>,
    mut q: Query<(&mut RotationAnimation, &mut Transform)>,
) {
    let dt = time.delta();

    q.par_for_each_mut(128, |(mut anim, mut tr)| {
        tr.rotation = anim.0.tick(dt);
    });
}

pub struct AnimationsPlugin;

impl Plugin for AnimationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_rotations_animations);
    }
}
