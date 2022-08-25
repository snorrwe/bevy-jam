use bevy::{
    prelude::*, reflect::TypeUuid, render::render_resource::AsBindGroup,
    sprite::Material2d,
};

use super::Health;

// FIXME: should be instanced

#[derive(Debug, Clone, TypeUuid, AsBindGroup)]
#[uuid = "04fbc2a5-871e-42ab-8ce7-d7137660054b"]
pub struct HpMaterial {
    #[uniform(0)]
    pub color_empty: Color,
    #[uniform(0)]
    pub color_full: Color,
    #[uniform(0)]
    pub hp: f32,
    #[uniform(0)]
    pub hp_max: f32,
}

impl Material2d for HpMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/health.wgsl".into()
    }
}

pub fn update_hp_materials(
    mut materials: ResMut<Assets<HpMaterial>>,
    mut q: Query<(&Handle<HpMaterial>, &Parent)>,
    hps: Query<&Health>,
) {
    q.for_each_mut(|(mat, parent)| {
        if let Some(mat) = materials.get_mut(mat) {
            if let Ok(hp) = hps.get(**parent) {
                mat.hp = hp.current_health;
                mat.hp_max = hp.max_health;
            }
        }
    });
}

pub fn update_hp_bar_transform(
    mut q: Query<(&mut Transform, &Parent), With<Handle<HpMaterial>>>,
    parents: Query<&GlobalTransform>,
) {
    q.par_for_each_mut(128, |(mut tr, parent)| {
        if let Ok(global_tr) = parents.get(**parent) {
            let (scale, rotation, _) =
                global_tr.to_scale_rotation_translation();
            let scale = 1.0 / scale;
            let rotation = rotation.inverse();
            tr.rotation = rotation;
            tr.scale = scale;
        }
    });
}
