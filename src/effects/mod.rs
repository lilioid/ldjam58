use bevy::prelude::*;
use crate::GameplaySystem;
use crate::sun_system::{Sun, Satellite, Level};
use crate::collision::{FatalCollisionEvent, HitBox};
mod red_star;
mod gravity_viz;

#[derive(Resource)]
struct SunFlameConfig {
    spikes: usize,
    inner_r: f32,
    outer_r: f32,
    speed: f32,
    variance: f32,
    core: Color,
    glow: Color,
}

impl Default for SunFlameConfig {
    fn default() -> Self {
        Self {
            spikes: 64,
            inner_r: 1.,
            outer_r: 22.0,
            speed: 0.1,
            variance: 12.0,
            core: Color::srgb(1.00, 0.60, 0.10),
            glow: Color::srgb(1.00, 0.30, 0.05),
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SunFlameConfig>();
    app.add_systems(Update, draw_sun_flames.in_set(GameplaySystem));
    app.add_observer(on_fatal_collision_swallow);
    app.add_systems(Update, (update_and_render_swallow_fx).in_set(GameplaySystem));
    red_star::plugin(app);
    gravity_viz::plugin(app);
}

fn draw_sun_flames(
    mut gizmos: Gizmos,
    sun: Query<(&GlobalTransform, &HitBox), With<Sun>>,
    time: Res<Time>,
    cfg: Res<SunFlameConfig>,
) {
    let Ok((gt, hb)) = sun.single() else { return; };
    let center = gt.translation().xy();
    let iso = Isometry2d::from_translation(center);

    let t = time.elapsed_secs();
    let n = cfg.spikes.max(3) as i32;
    let two_pi = std::f32::consts::TAU;

    // Scale effects with the sun radius (20.0 is the initial base radius)
    let effect_scale = hb.radius / 20.0;
    let inner_r = cfg.inner_r * effect_scale;
    let outer_r = cfg.outer_r * effect_scale;
    let variance = cfg.variance * effect_scale;

    gizmos.circle_2d(iso, outer_r, cfg.glow.with_alpha(0.10));

    for i in 0..n {
        let base = i as f32 / n as f32;
        let ang = base * two_pi + t * cfg.speed * 0.7;
        let dir = Vec2::from_angle(ang);

        let wobble = (t * cfg.speed * 2.3 + i as f32 * 1.37).sin();
        let outer = outer_r + 0.5 * variance + 0.5 * variance * wobble;

        let p0 = center + dir * inner_r;
        let p1 = center + dir * outer;

        gizmos.line_2d(p0, p1, cfg.glow.with_alpha(0.35));

        let p_mid = center + dir * (0.5 * (cfg.inner_r + outer));
        gizmos.line_2d(p0, p_mid, cfg.core.with_alpha(0.85));
    }
}


#[derive(Clone, Copy)]
enum SwallowStyle {
    LavaSplash,       // localized splash at entry point on sun surface
}

#[derive(Component)]
struct SwallowFx {
    start: Vec2,
    elapsed: f32,
    duration: f32,
    style: SwallowStyle,
    scale: f32,
}

fn on_fatal_collision_swallow(
    ev: On<FatalCollisionEvent>,
    mut commands: Commands,
    q_t: Query<&Transform>,
    q_sun: Query<(), With<Sun>>,
    q_sat: Query<&Level, With<Satellite>>
) {
    // Only for satellite swallowed by sun
    if q_sun.get(ev.other).is_err() { return; }
    let Ok(level) = q_sat.get(ev.destroyed) else { return; };

    let Ok(t) = q_t.get(ev.destroyed) else { return; };
    // Use localized lava splash at the sun entry point
    let style = SwallowStyle::LavaSplash;
    // Scale splash based on satellite level (moderate growth)
    let lvl = level.level.max(1.0);
    let scale = 0.8 + 0.4 * lvl; // 1->1.2, 2->1.6, 3->2.0
    commands.spawn((
        SwallowFx { start: t.translation.xy(), elapsed: 0.0, duration: 0.5, style, scale },
        Name::new("SwallowFx"),
    ));
}

fn update_and_render_swallow_fx(
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut q_fx: Query<(Entity, &mut SwallowFx)>,
    q_sun_t: Query<(&GlobalTransform, &HitBox), With<Sun>>,
    time: Res<Time>,
) {
    let Ok((gt, hb)) = q_sun_t.single() else { return; };
    let center = gt.translation().xy();
    let sun_r = hb.radius;
    let effect_scale = sun_r / 20.0; // 20.0 was the original base

    for (e, mut fx) in q_fx.iter_mut() {
        fx.elapsed += time.delta_secs();
        let t = (fx.elapsed / fx.duration).clamp(0.0, 1.0);
        let ease_out = 1.0 - (1.0 - t).powf(3.0);
        let ease_in = t.powf(2.0);

        let core = Color::srgb(1.00, 0.65, 0.15).with_alpha(0.9 * (1.0 - t));
        let glow = Color::srgb(1.00, 0.30, 0.05).with_alpha(0.6 * (1.0 - t));

        match fx.style {
            SwallowStyle::LavaSplash => {
                // Localized splash that splatters OUTWARDS from the sun surface at impact
                let to_c = center - fx.start;
                let inward = if to_c.length_squared() > 0.0001 { to_c.normalize() } else { Vec2::X };
                let outward = -inward;
                let impact = center + outward * sun_r; // surface impact point outside the sun
                let tangent = Vec2::new(-outward.y, outward.x);
                let s = fx.scale;
                let es = effect_scale;

                // Crown splatter: a compact set of outward jets with slight angular spread
                let crown_strength = (ease_out * (1.0 - ease_out)).mul_add(4.0, 0.0); // bell curve 0..1..0
                for i in -3..=3 {
                    let spread = i as f32 * 0.18;
                    let dir = (outward + tangent * spread).normalize_or_zero();
                    let len = (9.0 * es) * s * crown_strength * (1.0 - 0.12 * (i as f32).abs());
                    let tip = impact + dir * len;
                    gizmos.line_2d(impact, tip, glow.with_alpha(0.75 * (1.0 - t)));
                    // brighter core near the tips
                    let mid = impact + dir * (0.55 * len);
                    gizmos.line_2d(mid, tip, core.with_alpha(0.9 * (1.0 - t)));
                }

                // Minimal rim churn: a few short ticks right at the entry lobe only
                for j in -3..=3 {
                    let spread = j as f32 * 0.12;
                    let rim_dir = (outward + tangent * spread).normalize_or_zero();
                    let p_rim = impact;
                    let out = p_rim + rim_dir * ((1.2 * es) * s * (1.0 - t));
                    gizmos.line_2d(p_rim, out, core.with_alpha(0.5 * (1.0 - t)));
                }

                // Droplets (sparks) flung outward; fewer for perf
                for i in 0..4 {
                    let f = i as f32 / 3.0 - 0.5; // -0.5..0.5
                    let dir = (outward + tangent * (0.6 * f)).normalize_or_zero();
                    let pos_d = impact + dir * ((10.0 * es) * s * ease_out);
                    let r = ((1.1 * es) * s * (1.0 - t)).max(0.2);
                    let iso = Isometry2d::from_translation(pos_d);
                    gizmos.circle_2d(iso, r, core.with_alpha(0.55 * (1.0 - t)));
                }
            }
        }

        if fx.elapsed >= fx.duration {
            commands.entity(e).despawn();
        }
    }
}


