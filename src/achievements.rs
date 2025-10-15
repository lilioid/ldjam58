use bevy::prelude::*;
use crate::GameplaySystem;

pub struct AchievementsPlugin;
#[derive(Event, Debug, Copy, Clone)]
pub struct FullOrbitAchieved {
    pub entity: Entity,
}

impl Plugin for AchievementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_full_orbit);
        app.add_systems(Update, update_neon_circle_fx.in_set(GameplaySystem));
    }
}



#[derive(Component, Debug, Copy, Clone)]
pub struct FullOrbitAwarded;

#[derive(Component, Debug, Copy, Clone)]
struct NeonCircleFx {
    center: Vec2,
    elapsed: f32,
    duration: f32,
}

fn on_full_orbit(
    ev: On<FullOrbitAchieved>,
    q_t: Query<&Transform>,
    mut commands: Commands,
) {
    if let Ok(t) = q_t.get(ev.entity) {
        commands.spawn(NeonCircleFx { center: t.translation.xy(), elapsed: 0.2, duration: 1.0 });
    }
}

fn update_neon_circle_fx(
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut q: Query<(Entity, &mut NeonCircleFx)>,
    time: Res<Time>,
) {
    for (e, mut fx) in q.iter_mut() {
        fx.elapsed += time.delta_secs();
        let t = (fx.elapsed / fx.duration).clamp(0.0, 1.0);

        let radius = 3.0 + 6.0 * t; // expand
        let alpha = (1.0 - t).powf(2.0); // fade out
        let color = Color::srgb(0.2, 0.8, 1.0).with_alpha(0.75 * alpha);

        let iso = Isometry2d::from_translation(fx.center);
        gizmos.circle_2d(iso, radius, color);

        if fx.elapsed >= fx.duration { commands.entity(e).despawn(); }
    }
}


