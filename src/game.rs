use bevy::prelude::*;

pub struct GamePlugin;

#[derive(Default)]
pub struct GameAssets {
    pub player_sprite: Handle<TextureAtlas>,
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

    cmd.spawn_bundle(SpriteSheetBundle {
        texture_atlas: game_assets.player_sprite.clone(),
        transform: Transform::from_scale(Vec3::new(1., 1., 1.)),
        ..Default::default()
    });
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets::default())
            .add_startup_system(setup_game);
    }
}
