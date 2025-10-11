use bevy::prelude::*;
use crate::GameplaySystem;
use crate::collision::{ HitBox};
use crate::score::Score;
use crate::sun_system::{SolarSystemAssets, Sun};
use super::SunFlameConfig;

#[derive(Resource, Default)]
pub struct RedStarState {
    pub converted: bool,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RedStarState>();
    app.add_systems(Update, red_star_conversion.in_set(GameplaySystem));
}

fn red_star_conversion(
    mut state: ResMut<RedStarState>,
    score: Res<Score>,
    assets: Res<SolarSystemAssets>,
    mut sun_q: Query<(Entity, &mut Transform, &mut HitBox, &mut Sprite), With<Sun>>, 
    mut cfg: ResMut<SunFlameConfig>,
) {
    if state.converted { return; }
    // Treat reaching 10k energy as reaching level 2
    if score.energy_stored < 10_000.0 { return; }

    state.converted = true;

    if let Ok((sun_entity, mut sun_t, mut sun_hitbox, mut sun_sprite)) = sun_q.single_mut() {
        // Visual: make sun slightly bigger and swap sprite to red sun
        sun_t.scale *= 1.25;
        sun_hitbox.radius *= 1.70;
        *sun_sprite = Sprite::from(assets.redsun.clone());

        // Tint flames to redder hues and slightly expand
        cfg.core = Color::srgb(1.00, 0.25, 0.10);
        cfg.glow = Color::srgb(1.00, 0.10, 0.05);
        cfg.outer_r *= 1.15;

    }
}


