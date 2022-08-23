use bevy::prelude::*;

use crate::{
    collision::AABBCollision, game::PlayerController, interaction::MouseFollow,
    interaction::Selected,
};

pub struct WorkerLogicPlugin;

#[derive(Component)]
pub struct UnitFollowPlayer;

#[derive(Component)]
pub struct CanEatWorker {
    pub entity_to_eat: Option<Entity>,
}

fn eat_other_worker_system(
    selected: Res<Selected>,
    global_tr: Query<&GlobalTransform>,
    mut can_eat_workers: Query<(&mut CanEatWorker, Entity)>,
) {
    if let Some(selected_e) = selected.0 {
        if let Ok(selected_tr) = global_tr.get(selected_e) {
            let selected_entity_pos = selected_tr.translation();

            for (mut can_eat, e) in can_eat_workers.iter_mut() {
                can_eat.entity_to_eat = None;
                if let Ok(can_eat_tr) = global_tr.get(e) {
                    if (can_eat_tr.translation().truncate()
                        - selected_entity_pos.truncate())
                    .length()
                        < 10.
                    {
                        can_eat.entity_to_eat = Some(selected_e);
                    }
                }
            }
        }
    }
}

fn player_follower_system(
    mut q_player_followers: Query<
        &mut Transform,
        (With<UnitFollowPlayer>, Without<MouseFollow>),
    >,
    player: Query<&GlobalTransform, With<PlayerController>>,
    time: Res<Time>,
) {
    let player_tr = player.single();
    for mut tr in q_player_followers.iter_mut() {
        let direction_vector = player_tr.translation() - tr.translation;

        let direction_vector = direction_vector.truncate();
        if direction_vector.length() < 200. {
            continue;
        }
        let direction_vector = direction_vector.normalize();

        tr.translation +=
            direction_vector.extend(0.) * time.delta_seconds() * 150.;
    }
}

impl Plugin for WorkerLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_follower_system)
            .add_system(eat_other_worker_system);
    }
}
