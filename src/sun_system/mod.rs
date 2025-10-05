pub mod navigation_instruments;
pub mod thruster;
mod earth;
mod asteroids;

use crate::{AppSystems, GameplaySystem};
use crate::asset_tracking::LoadResource;
use crate::physics::calc_gravity::{Attractee, Attractor};
use crate::physics::directional_forces::{GravityForce, Mass};
use crate::physics::velocity::Velocity;
use crate::screens::Screen;
use crate::sun_system::navigation_instruments::NavigationInstruments;
use crate::sun_system::thruster::{thruster_use_fuel, Thruster, ThrusterDirection};
use bevy::input::common_conditions::{input_just_pressed, input_just_released};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::collision::HitBox;

#[derive(Component)]
struct TiledGrid {
    cols: i32,
    rows: i32,
    tile_world_size: f32,
}

#[derive(Component)]
struct GridIndex {
    col: i32,
    row: i32,
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((earth::plugin, asteroids::plugin));
    app.load_resource::<SolarSystemAssets>();
    app.add_systems(
        FixedUpdate,
        (thruster::apply_thrust_force)
            .in_set(AppSystems::Physics)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        thruster::toggle_thruster
            .run_if(
                input_just_pressed(thruster::THRUSTER_KEY)
                    .or(input_just_released(thruster::THRUSTER_KEY)),
            )
            .in_set(AppSystems::RecordInput),
    );
    app.add_systems(
        Update,
        navigation_instruments::draw_nav_projections
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );
    
    app.add_systems(Update, thruster_use_fuel.in_set(GameplaySystem));
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct SolarSystemAssets {
    #[dependency]
    sun: Handle<Image>,

    #[dependency]
    pub(crate) collector: Handle<Image>,

    #[dependency]
    grid: Handle<Image>,
    
    #[dependency]
    pub crash: Handle<Image>,

    #[dependency]
    pub(crate) bg: Handle<Image>,

    #[dependency]
    pub(crate) font: Handle<Font>,

    #[dependency]
    pub crash_sound: Handle<AudioSource>,

    #[dependency]
    pub warning_sound: Handle<AudioSource>,

    #[dependency]
    pub music_loop: Handle<AudioSource>,

}

#[derive(Component)]
pub struct Satellite;

#[derive(Component, Debug, Copy, Clone)]
pub struct Level{
    pub level: f32,
}
// depending on level energy rate of one satellite increase

#[derive(Component)]
pub struct Sun;


impl FromWorld for SolarSystemAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            sun: assets.load("sun.png"),
            crash: assets.load("crash.png"),
            grid: assets.load("retro_grid.png"),
            collector: assets.load("satellite.png"),
            bg: assets.load("retro_grid_bg.png"),
            font: assets.load("fonts/lucon.ttf"),
            crash_sound: assets.load("sounds/collision.wav"),
            warning_sound: assets.load("sounds/beepx3.wav"),
            music_loop: assets.load("sounds/music_loop.wav"),
        }
    }
}

pub fn init_sun_system(mut commands: Commands, solar_system_assets: Res<SolarSystemAssets>) {
    info!("Adding sun");
    commands.spawn((
        Attractor,
        HitBox {
            radius: 20.0
        },
        Mass(100_000_000_000_000.0),
        Name::new("Sun"),
        Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.02)),
        Sprite::from(solar_system_assets.sun.clone()),
        Sun
    ));
}

pub fn setup_grid_image(mut commands: Commands, solar_system_assets: Res<SolarSystemAssets>) {
    commands.spawn((
        Name::new("GridImage"),
        Sprite::from(solar_system_assets.bg.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.0, -5.0)).with_scale(Vec3::splat(0.15))
    ));
}