use bevy::prelude::*;

use crate::{game::PlayerController, interaction::MouseFollow};

pub struct WorkerLogicPlugin;

#[derive(Component)]
pub struct UnitFollowPlayer;

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
        app.add_system(player_follower_system);
    }
}
