use bevy::{math::Vec3A, prelude::*, render::camera::*};

use crate::{
    collision::AABB,
    worker_logic::{CanEatWorker, WorkerColor},
    ChangeTimeScaleEvent, PlayerCamera, Selectable, DEFAULT_TIME_SCALE,
};

pub struct InteractionPlugin;

pub struct Hovered(pub Option<Entity>);
pub struct Selected(pub Option<Entity>);
#[derive(Component)]
pub struct MouseFollow;

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

fn mouse_follow_system(
    mut cursor_pos: Local<Vec2>,
    mut cur_move: EventReader<CursorMoved>,
    windows: Res<Windows>,
    cameras: Query<
        (&GlobalTransform, &OrthographicProjection),
        With<PlayerCamera>,
    >,
    mut followers: Query<&mut Transform, With<MouseFollow>>,
) {
    for m in cur_move.iter() {
        let win = windows.get(m.id).expect("window not found");

        *cursor_pos = m.position;
        for (cam_tr, proj) in cameras.iter() {
            let cursor_world =
                window_to_world(*cursor_pos, &win, *cam_tr, proj);

            for mut tr in followers.iter_mut() {
                tr.translation =
                    Vec3::new(cursor_world.x, cursor_world.y, tr.translation.z);
            }
        }
    }
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
    mut cmd: Commands,
    mut time_event: EventWriter<ChangeTimeScaleEvent>,
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
        if let Some(e) = selected.0 {
            cmd.entity(e).insert(MouseFollow);
            time_event.send(ChangeTimeScaleEvent {
                new_time_scale: 0.1 * DEFAULT_TIME_SCALE,
            });
        }
    }
}

fn deselect_on_mouse_up(
    btn: Res<Input<MouseButton>>,
    mut selected: ResMut<Selected>,
    mut cmd: Commands,
    mut eater: Query<(&mut CanEatWorker, Entity)>,
    mut worker_color: Query<(&mut WorkerColor, &mut Transform)>,
    mut time_event: EventWriter<ChangeTimeScaleEvent>,
) {
    if btn.just_released(MouseButton::Left) {
        if let Some(e) = selected.0.take() {
            time_event.send(ChangeTimeScaleEvent {
                new_time_scale: DEFAULT_TIME_SCALE,
            });
            cmd.entity(e).remove::<MouseFollow>();

            for (eats, eater_entity) in eater.iter_mut() {
                if let Some(entity_to_eat) = eats.entity_to_eat {
                    if entity_to_eat == e {
                        info!("{:?} ate {:?}", eater_entity, e);
                        cmd.entity(e).despawn_recursive();
                    }

                    let mut prey_color: Color = Color::BLACK;
                    let mut prey_size: f32 = 0.;
                    if let Ok((prey_c, prey_tr)) = worker_color.get_mut(e) {
                        prey_color = prey_c.color;
                        prey_size = prey_tr.scale.x;
                    }
                    if prey_size != 0. {
                        if let Ok((mut eater_color, mut tr)) =
                            worker_color.get_mut(eater_entity)
                        {
                            eater_color.color = Color::rgb(
                                (eater_color.color.r() + prey_color.r()) / 2.,
                                (eater_color.color.g() + prey_color.g()) / 2.,
                                (eater_color.color.b() + prey_color.b()) / 2.,
                            );
                            tr.scale += prey_size / 10.;
                        }
                    }

                    selected.0 = None;
                    return;
                }
            }
        }
    }
}

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(select_worker_system)
            .add_system_to_stage(CoreStage::PostUpdate, deselect_on_mouse_up)
            .add_system(mouse_follow_system)
            .insert_resource(Hovered(None))
            .insert_resource(Selected(None));
    }
}
