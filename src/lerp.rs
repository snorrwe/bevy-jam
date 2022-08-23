use bevy::prelude::{Color, Quat};

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

impl Lerp for Quat {
    fn lerp(&self, rhs: &Self, t: f32) -> Self {
        self.slerp(*rhs, t)
    }
}
