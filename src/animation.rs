use std::time::Duration;

use bevy::prelude::*;

use crate::lerp::Lerp;

#[derive(Clone)]
pub struct Animation<T> {
    from: T,
    to: T,
    timer: Timer,
    // TODO: easing
}

impl<T: Lerp> Animation<T> {
    pub fn tick(&mut self, dt: Duration) -> T {
        self.timer.tick(dt);
        let t = self.timer.percent();
        self.from.lerp(&self.to, t)
    }
}

#[derive(Clone, Component)]
pub struct RotationAnimation(pub Animation<Quat>);

pub fn update_rotations_animations(
    time: Res<Time>,
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
