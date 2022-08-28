use crate::PlayerCamera;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

pub struct Options {
    pub master_volume: f32,
    pub music_volume: f32,
}

#[derive(Default, Clone)]
pub struct PlayAudioEventPositional {
    pub id: u32,
    pub position: Vec3,
    pub sound: Handle<AudioSource>,
}

#[derive(Default)]
pub struct AudioAssets {
    pub bow_release: Handle<AudioSource>,
    pub eating: Handle<AudioSource>,
    pub getting_damaged: Handle<AudioSource>,
    pub getting_healed: Handle<AudioSource>,
    pub soundtrack: Handle<AudioSource>,
    pub healer_casting: Handle<AudioSource>,
    pub mining: Handle<AudioSource>,
    pub sword_attack: Handle<AudioSource>,
    pub spawning_unit: Handle<AudioSource>,
}

pub fn audio_volume_manager_system(
    music_handle: Res<MusicHandle>,
    options: Res<Options>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Some(instance) = audio_instances.get_mut(&music_handle.0) {
        instance.set_volume(
            (options.master_volume / 100. * options.music_volume / 100.) as f64,
            AudioTween::default(),
        );
    }
}

#[derive(Default)]
pub struct MusicHandle(Handle<AudioInstance>);

pub fn audio_event_manager_system(
    mut audio_events: EventReader<PlayAudioEventPositional>,
    player_pos: Query<&GlobalTransform, With<PlayerCamera>>,
    audio: Res<Audio>,
    options: Res<Options>,
) {
    for audio_event in audio_events.iter() {
        let mut volume = 1.;

        for pos in player_pos.iter() {
            let distance = (pos.translation().truncate()
                - audio_event.position.truncate())
            .length();
            volume = (1000. / distance.max(0.01)).min(1.);
            volume *= options.master_volume / 100.;
        }

        audio
            .play(audio_event.sound.clone())
            .with_volume(volume as f64);
    }
}

pub struct AudioPlugin;
pub fn setup_system(
    asset_server: Res<AssetServer>,
    mut audio_assets: ResMut<AudioAssets>,
    mut music_handler: ResMut<MusicHandle>,
    audio: Res<Audio>,
) {
    audio_assets.bow_release = asset_server.load("audio/bowrelease.mp3");

    audio_assets.eating = asset_server.load("audio/eating.mp3");

    audio_assets.getting_damaged =
        asset_server.load("audio/gettingdamaged.mp3");

    audio_assets.getting_healed = asset_server.load("audio/gettinghealed.mp3");
    audio_assets.soundtrack = asset_server.load("audio/goovsgoblin.mp3");
    audio_assets.healer_casting = asset_server.load("audio/healercasting.mp3");
    audio_assets.mining = asset_server.load("audio/mininglooped.mp3");
    audio_assets.sword_attack = asset_server.load("audio/swordattack.mp3");
    audio_assets.spawning_unit = asset_server.load("audio/unitpoppingout.mp3");

    *music_handler = MusicHandle(
        audio
            .play(audio_assets.soundtrack.clone())
            .looped()
            .handle(),
    );
    info!("Playing sound!");
}

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Options {
            master_volume: 1.,
            music_volume: 1.,
        })
        .insert_resource(AudioAssets::default())
        .insert_resource(MusicHandle::default())
        .add_event::<PlayAudioEventPositional>()
        .add_startup_system(setup_system)
        .add_system(audio_volume_manager_system)
        .add_system(audio_event_manager_system);
    }
}
