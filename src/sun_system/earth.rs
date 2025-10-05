use crate::asset_tracking::LoadResource;
use crate::screens::Screen;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<EarthAssets>();
    app.add_systems(OnEnter(Screen::Gameplay), init_earth);
}

#[derive(Resource, Asset, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct EarthAssets {
    #[dependency]
    earth: Handle<Image>,
}

impl FromWorld for EarthAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            earth: assets.load("planet.png"),
        }
    }
}

/// A marker component for the players home planet
#[derive(Component)]
#[require(Transform)]
pub struct Earth;

fn init_earth(mut commands: Commands, assets: Res<EarthAssets>) {
    info!("Init earth");

    commands.spawn((
        Name::new("Earth"), 
        Earth,
        Transform::from_translation(Vec3::new(90.0, 0.0, 0.0)).with_scale(Vec3::splat(0.004)),
        Sprite::from(assets.earth.clone()),
    ));
}
