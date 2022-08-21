use bevy::prelude::*;

pub struct GamePlugin;

#[derive(Default)]
pub struct GameAssets {
    pub player_sprite: Handle<TextureAtlas>,
    pub worker_head: Handle<TextureAtlas>,
    pub worker_head_eating: Handle<TextureAtlas>,
    pub worker_eye: Handle<TextureAtlas>,
    pub worker_body: Handle<TextureAtlas>,
}

fn z_sorter(
    mut q_transform: Query<(&mut Transform, &GlobalTransform), Without<Camera>>,
) {
    for (mut tr, global_tr) in q_transform.iter_mut() {
        tr.translation.z = global_tr.translation().y; //TODO: Add offset component that can track other entity, and calculate own z from that and a given constant (for body parts)
    }
}

fn setup_game(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut game_assets: ResMut<GameAssets>,
) {
    game_assets.player_sprite = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/player/blob.png"),
        Vec2::new(364., 307.),
        1,
        1,
    ));
    game_assets.worker_body = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/workers/workerbody.png"),
        Vec2::new(88., 104.),
        1,
        1,
    ));
    game_assets.worker_head = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/workers/workerhead.png"),
        Vec2::new(89., 95.),
        1,
        1,
    ));
    game_assets.worker_eye = texture_atlases.add(TextureAtlas::from_grid(
        asset_server.load("sprites/workers/workereyes.png"),
        Vec2::new(106., 74.),
        1,
        1,
    ));

    cmd.spawn_bundle(SpriteSheetBundle {
        texture_atlas: game_assets.player_sprite.clone(),
        transform: Transform::from_scale(Vec3::new(1., 1., 1.)),
        ..Default::default()
    });

    spawn_regular_unit(&mut cmd, &game_assets);
}

fn spawn_regular_unit(cmd: &mut Commands, game_assets: &GameAssets) {
    cmd.spawn_bundle(SpriteSheetBundle {
        texture_atlas: game_assets.worker_body.clone(),
        transform: Transform::from_translation(Vec3::new(180., 0., 10.)),
        ..Default::default()
    })
    .with_children(|child| {
        child
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_assets.worker_head.clone(),
                transform: Transform::from_translation(Vec3::new(0., 85., 0.)),
                ..Default::default()
            })
            .with_children(|child2| {
                child2.spawn_bundle(SpriteSheetBundle {
                    texture_atlas: game_assets.worker_eye.clone(),
                    ..Default::default()
                });
            });
    });
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets::default())
            .add_startup_system(setup_game)
            .add_system(z_sorter);
    }
}
