use bevy::audio::Volume;
use bevy::prelude::*;
use crate::collision::FatalCollisionEvent;
use crate::screens::Screen;
use crate::sun_system::{SolarSystemAssets, Sun};
use crate::sun_system::asteroids::AsteroidSwarm;

pub(crate) struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), setup_sound);
        app.add_observer(handle_fatal_collision_event_for_sound);
        app.insert_resource(GlobalVolume::new(Volume::Linear(0.1)));
    }
}

fn setup_sound(mut commands: Commands, solar_system_assets: Res<SolarSystemAssets>) {
    commands.spawn((
        AudioPlayer::new(solar_system_assets.music_loop.clone()),
        PlaybackSettings::LOOP,
    ));
}

fn handle_fatal_collision_event_for_sound(
    event: On<FatalCollisionEvent>,
    mut commands: Commands,
    solar_system_assets: Res<SolarSystemAssets>,
    asteroid_swarm_query: Query<Entity, With<AsteroidSwarm>>,
    sun_query: Query<(), With<Sun>>,
) {
    if let Some(asteroid_swarm_entity) = asteroid_swarm_query.iter().next() {
        if event.destroyed == asteroid_swarm_entity { return; }
    }
    // Mute crash SFX when swallowed by the sun
    if sun_query.get(event.other).is_ok() {
        return;
    }
    commands.spawn((
        AudioPlayer::new(solar_system_assets.crash_sound.clone()),
        PlaybackSettings::DESPAWN,
    ));

}