mod thruster;

use crate::AppSystems;
use crate::asset_tracking::LoadResource;
use crate::physics::calc_gravity::{Attractee, Attractor};
use crate::physics::directional_forces::{GravityForce, Mass};
use crate::physics::velocity::Velocity;
use crate::sun_system::thruster::{Thruster, ThrusterDirection};
use bevy::color::palettes::basic::{GRAY, YELLOW};
use bevy::input::common_conditions::{input_just_pressed, input_just_released};
use bevy::prelude::*;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<SolarSystemAssets>();
    app.add_systems(
        FixedUpdate,
        (thruster::apply_thrust_force).in_set(AppSystems::Physics).run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        (thruster::toggle_thruster
            .run_if(
                input_just_pressed(thruster::THRUSTER_KEY)
                    .or(input_just_released(thruster::THRUSTER_KEY)),
            )
            .in_set(AppSystems::RecordInput)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct SolarSystemAssets {
    #[dependency]
    sun: Handle<Image>,
    #[dependency]
    pub(crate) collector: Handle<Image>,
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
    info!("Adding sun");
    commands.spawn((
        Attractor,
        Mass(10000000000.0),
        Name::new("Sun"),
        Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.02)),
        Sprite::from(solar_system_assets.sun.clone()),
    ));

    info!("Adding orbiting satellite");
    commands.spawn((
        Name::new("satelite"),
        Attractee,
        Thruster::new(ThrusterDirection::RadialIn, 2.0),
        GravityForce::default(),
        Velocity(Vec2::new(0.0, 0.1)),
        Mass(1.0),
        Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)).with_scale(Vec3::splat(0.025)),
        Sprite::from(solar_system_assets.collector.clone()),
    ));
}
