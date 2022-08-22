use bevy::asset::Handle;
use bevy::prelude::shape::Box;
use bevy::render::render_resource::AsBindGroup;
use bevy::render::render_resource::{
    BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BufferSize, ShaderStages,
};
use bevy::sprite::{
    Material2d, Material2dPipeline, MaterialMesh2dBundle, Mesh2dHandle,
};
use bevy::{
    ecs::system::lifetimeless::SRes,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::RenderAsset,
        render_resource::{
            BindGroup, BindGroupDescriptor, Buffer, BufferUsages,
        },
        renderer::RenderDevice,
    },
};

use super::AABBDescriptor;

#[derive(Component)]
pub struct AABBVis;

// This is the struct that will be passed to your shader
#[derive(Debug, Clone, TypeUuid, AsBindGroup)]
#[uuid = "0a705263-f68e-4fea-a120-ab07bae8078c"]
pub struct AABBMaterial {
    #[uniform(0)]
    pub color: Color,
}

pub struct AABBVizAssets {
    pub material: Handle<AABBMaterial>,
}

impl Material2d for AABBMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/collider.wgsl".into()
    }
}

pub(crate) fn on_new_aabb(
    assets: Res<AABBVizAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cmd: Commands,
    q: Query<(Entity, &AABBDescriptor), Added<AABBDescriptor>>,
) {
    for (entity, desc) in q.iter() {
        let radius = desc.radius;
        let [min_x, min_y, min_z] = (-radius).to_array();
        let [max_x, max_y, max_z] = radius.to_array();
        let child = cmd
            .spawn()
            .insert_bundle(MaterialMesh2dBundle {
                material: assets.material.clone(),
                mesh: Mesh2dHandle(meshes.add(Mesh::from(Box {
                    min_x,
                    min_y,
                    min_z,
                    max_x,
                    max_y,
                    max_z,
                }))),
                transform: Transform::from_translation(Vec3::Z * 20.0),
                ..Default::default()
            })
            .insert(AABBVis)
            .id();

        cmd.entity(entity).add_child(child);
    }
}

pub(crate) fn update_aabb_meshes(
    mut meshes: ResMut<Assets<Mesh>>,
    q: Query<(&Handle<Mesh>, &AABBVis, &Parent)>,
    qp: Query<&AABBDescriptor, Changed<AABBDescriptor>>,
) {
    for (mesh, _, parent) in q.iter() {
    //     if let Ok(desc) = qp.get(**parent) {
    //         let mesh = meshes.get_mut(mesh).unwrap();
    //
    //         *mesh = Box {
    //             size: desc.radius.truncate(),
    //             flip: false,
    //         }
    //         .into();
    //     }
        todo!()
    }
}

pub(crate) fn setup(
    mut assets: ResMut<AABBVizAssets>,
    mut mats: ResMut<Assets<AABBMaterial>>,
) {
    *assets = AABBVizAssets {
        material: mats.add(AABBMaterial { color: Color::RED }),
    };
}
