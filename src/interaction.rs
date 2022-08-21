use bevy::{math::Vec3A, prelude::*, render::camera::CameraProjection};

use crate::{collision::AABB, PlayerCamera, Selectable};

pub struct InteractionPlugin;

pub struct Hovered(pub Option<Entity>);
pub struct Selected(pub Option<Entity>);

fn window_to_world(
    window_pos: Vec2,
    window: &Window,
    mut cam_transform: GlobalTransform,
    projection: &OrthographicProjection,
) -> Vec3 {
    // normalized device coordinates
    let ndc = Vec3::new(
        (2.0 * window_pos.x) / window.width() - 1.,
        (2.0 * window_pos.y) / window.height() - 1.,
        projection.near,
    );

    // translation can be fairly large, compensate by computing at the origin and then translating
    // the result
    let trans = cam_transform.translation();
    *cam_transform.translation_mut() = Vec3A::ZERO;
    let ndc_to_world = cam_transform.compute_matrix()
        * projection.get_projection_matrix().inverse();
    let res = ndc_to_world.project_point3(ndc);

    res + trans
}

fn select_worker_system(
    mut cursor_pos: Local<Vec2>,
    windows: Res<Windows>,
    mut cur_move: EventReader<CursorMoved>,
    workers: Query<(Entity, &AABB), With<Selectable>>,
    cameras: Query<
        (&GlobalTransform, &OrthographicProjection),
        With<PlayerCamera>,
    >,
    mut hovered: ResMut<Hovered>,
    mut selected: ResMut<Selected>,
    btn: Res<Input<MouseButton>>,
) {
    for m in cur_move.iter() {
        let win = windows.get(m.id).expect("window not found");

        *cursor_pos = m.position;
        for (cam_tr, proj) in cameras.iter() {
            let cursor_world =
                window_to_world(*cursor_pos, &win, *cam_tr, proj);

            let cam_pos = cam_tr.translation();
            let d = cursor_world - cam_pos;

            hovered.0 = None;
            debug!("Handling hover for pos: {} d: {}", cam_pos, d);
            for (entity, aabb) in workers.iter() {
                trace!("Testing aabb: {:?}", aabb);
                if crate::collision::primitives::ray_aabb(
                    [cam_pos, d],
                    [aabb.min, aabb.max],
                )
                .is_some()
                {
                    trace!("Hovering {:?}", entity);
                    hovered.0 = Some(entity)
                }
            }
        }
    }
    if btn.just_pressed(MouseButton::Left) {
        debug!("Select entity: {:?}", hovered.0);
        selected.0 = hovered.0;
    }
}

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(select_worker_system)
            .insert_resource(Hovered(None))
            .insert_resource(Selected(None));
    }
}
