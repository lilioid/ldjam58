use bevy::color::palettes::basic::{GRAY, YELLOW};
use bevy::image::ImageLoaderSettings;
use bevy::prelude::*;
use crate::asset_tracking::LoadResource;
use crate::physics::calc_gravity::{Attractee, Attractor};
use crate::physics::apply_directional_force::{GravityForce, Mass};
use crate::physics::velocity::Velocity;

pub(crate) struct SunSystemPlugin;


pub(super) fn plugin(app: &mut App) {
    app.load_resource::<SolarSystemAssets>();
}


#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct SolarSystemAssets {
    #[dependency]
    sun: Handle<Image>,
    #[dependency]
    collector: Handle<Image>,
}

impl FromWorld for SolarSystemAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            sun: assets.load("sun.png"),
            collector: assets.load("collector.png"),
        }
    }
}

pub fn init_sun_system(mut commands: Commands, solar_system_assets: Res<SolarSystemAssets>) {
    debug!("Adding sun");
    commands.spawn((
        Attractor,
        Mass(10000000000.0),
        Name::new("Sun"),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_scale(Vec3::splat(0.2)),
        Sprite::from(solar_system_assets.sun.clone())
    ));

    debug!("Adding orbiting satellite");
    commands.spawn((
        Attractee,
        GravityForce(Vec2::new(1.0, 0.0)),
        Velocity(Vec2::new(0.0, 0.0)),
        Mass(1.0),
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)).with_scale(Vec3::splat(0.3)),
        Sprite::from(solar_system_assets.collector.clone())
    ));
}