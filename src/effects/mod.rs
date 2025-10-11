use bevy::prelude::*;
use crate::GameplaySystem;
use crate::sun_system::{Sun, Satellite, Level};
use crate::collision::FatalCollisionEvent;

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
    app.init_resource::<SwallowStyleCycler>();
    app.add_systems(Update, (update_and_render_swallow_fx).in_set(GameplaySystem));
}

fn draw_sun_flames(
    mut gizmos: Gizmos,
    sun: Query<&GlobalTransform, With<Sun>>,
    time: Res<Time>,
    cfg: Res<SunFlameConfig>,
) {
    let Ok(gt) = sun.single() else { return; };
    let center = gt.translation().xy();
    let iso = Isometry2d::from_translation(center);

    let t = time.elapsed_secs();
    let n = cfg.spikes.max(3) as i32;
    let two_pi = std::f32::consts::TAU;

    gizmos.circle_2d(iso, cfg.outer_r, cfg.glow.with_alpha(0.10));

    for i in 0..n {
        let base = i as f32 / n as f32;
        let ang = base * two_pi + t * cfg.speed * 0.7;
        let dir = Vec2::from_angle(ang);

        let wobble = (t * cfg.speed * 2.3 + i as f32 * 1.37).sin();
        let outer = cfg.outer_r + 0.5 * cfg.variance + 0.5 * cfg.variance * wobble;

        let p0 = center + dir * cfg.inner_r;
        let p1 = center + dir * outer;

        gizmos.line_2d(p0, p1, cfg.glow.with_alpha(0.35));

        let p_mid = center + dir * (0.5 * (cfg.inner_r + outer));
        gizmos.line_2d(p0, p_mid, cfg.core.with_alpha(0.85));
    }
}


#[derive(Clone, Copy)]
enum SwallowStyle { 
    IngestBeam,       // current line + ring + rim ripple
    SpiralVortex,     // spiral into sun
    EmberSpray,       // sparks flying and fading into sun
    ShockwaveSink,    // concentric shockwaves leading inward
    FilamentArcs,     // arcing filaments that curl toward sun
    LavaSplash,       // localized splash at entry point on sun surface
}

#[derive(Resource, Default)]
struct SwallowStyleCycler { idx: usize }

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
    q_sat: Query<&Level, With<Satellite>>,
    mut cycler: ResMut<SwallowStyleCycler>,
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
    q_sun_t: Query<&GlobalTransform, With<Sun>>,
    time: Res<Time>,
) {
    let Ok(gt) = q_sun_t.single() else { return; };
    let center = gt.translation().xy();

    for (e, mut fx) in q_fx.iter_mut() {
        fx.elapsed += time.delta_secs();
        let t = (fx.elapsed / fx.duration).clamp(0.0, 1.0);
        let ease_out = 1.0 - (1.0 - t).powf(3.0);
        let ease_in = t.powf(2.0);

        let core = Color::srgb(1.00, 0.65, 0.15).with_alpha(0.9 * (1.0 - t));
        let glow = Color::srgb(1.00, 0.30, 0.05).with_alpha(0.6 * (1.0 - t));

        match fx.style {
            SwallowStyle::IngestBeam => {
                let pos = fx.start.lerp(center, ease_out);
                let dir = (center - pos).normalize_or_zero();
                let orth = Vec2::new(-dir.y, dir.x);
                let head = center - dir * 8.0;
                gizmos.line_2d(pos, head, core);
                gizmos.line_2d(pos + orth * 1.2, head + orth * 1.2, glow);
                gizmos.line_2d(pos - orth * 1.2, head - orth * 1.2, glow);
                let ring_r = 6.0 * (1.0 - t);
                let iso = Isometry2d::from_translation(pos);
                gizmos.circle_2d(iso, ring_r.max(0.5), core.with_alpha(0.5 * (1.0 - t)));
                let rim = Isometry2d::from_translation(center);
                let rim_r = 20.0 + 3.0 * (0.5 + 0.5 * (t * 8.0).sin()) * (1.0 - t);
                gizmos.circle_2d(rim, rim_r, glow.with_alpha(0.25 * (1.0 - t)));
            }
            SwallowStyle::SpiralVortex => {
                let turns = 1.5;
                let angle = turns * std::f32::consts::TAU * ease_out;
                let radius = (fx.start - center).length() * (1.0 - ease_out);
                let pos = center + Vec2::from_angle(angle) * radius;
                let orth = Vec2::from_angle(angle + std::f32::consts::FRAC_PI_2);
                gizmos.line_2d(pos, pos + orth * 10.0 * (1.0 - t), glow);
                for k in [-1.5, -0.5, 0.5, 1.5] {
                    let seg = pos + orth * (k as f32) * 2.0;
                    gizmos.line_2d(seg, seg + (center - seg).normalize_or_zero() * 6.0, core);
                }
                let rim = Isometry2d::from_translation(center);
                gizmos.circle_2d(rim, 20.0 + 2.5 * (1.0 - t), glow.with_alpha(0.25 * (1.0 - t)));
            }
            SwallowStyle::EmberSpray => {
                let pos = fx.start.lerp(center, ease_out);
                let dir = (center - pos).normalize_or_zero();
                let orth = Vec2::new(-dir.y, dir.x);
                for i in 0..7 {
                    let f = i as f32 / 6.0;
                    let spread = (f - 0.5) * 6.0;
                    let p = pos + orth * spread * (1.0 - t) * 2.0;
                    let q = p + dir * (8.0 + 6.0 * f) * (1.0 - t);
                    gizmos.line_2d(p, q, core.with_alpha(0.8 * (1.0 - t)));
                }
                let iso = Isometry2d::from_translation(pos);
                gizmos.circle_2d(iso, 4.0 * (1.0 - t), glow.with_alpha(0.4 * (1.0 - t)));
            }
            SwallowStyle::ShockwaveSink => {
                let pos = fx.start.lerp(center, ease_out);
                let iso = Isometry2d::from_translation(pos);
                let r_main = 8.0 * (1.0 - t);
                gizmos.circle_2d(iso, r_main, core.with_alpha(0.6 * (1.0 - t)));
                for j in 0..3 {
                    let f = j as f32 / 3.0;
                    let r = r_main + 4.0 * f;
                    gizmos.circle_2d(iso, r, glow.with_alpha(0.25 * (1.0 - t)));
                }
                let dir = (center - pos).normalize_or_zero();
                let head = center - dir * 10.0 * (1.0 - t);
                gizmos.line_2d(pos, head, core);
            }
            SwallowStyle::FilamentArcs => {
                let pos = fx.start.lerp(center, ease_in);
                let to_c = center - pos;
                let base_ang = to_c.to_angle();
                for i in [-2, -1, 0, 1, 2] {
                    let a = base_ang + (i as f32) * 0.25;
                    let p = pos + Vec2::from_angle(a) * (8.0 * (1.0 - t));
                    let mid = pos + Vec2::from_angle((a + base_ang) * 0.5) * (to_c.length() * 0.4);
                    gizmos.line_2d(p, mid, glow.with_alpha(0.7 * (1.0 - t)));
                    gizmos.line_2d(mid, center - Vec2::from_angle(a) * 6.0, core.with_alpha(0.9 * (1.0 - t)));
                }
            }
            SwallowStyle::LavaSplash => {
                // Localized splash that splatters OUTWARDS from the sun surface at impact
                let sun_r = 20.0; // matches HitBox radius of the sun
                let to_c = center - fx.start;
                let inward = if to_c.length_squared() > 0.0001 { to_c.normalize() } else { Vec2::X };
                let outward = -inward;
                let impact = center + outward * sun_r; // surface impact point outside the sun
                let tangent = Vec2::new(-outward.y, outward.x);
                let s = fx.scale;

                // Crown splatter: a compact set of outward jets with slight angular spread
                let crown_strength = (ease_out * (1.0 - ease_out)).mul_add(4.0, 0.0); // bell curve 0..1..0
                for i in -3..=3 {
                    let spread = i as f32 * 0.18;
                    let dir = (outward + tangent * spread).normalize_or_zero();
                    let len = 9.0 * s * crown_strength * (1.0 - 0.12 * (i as f32).abs());
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
                    let out = p_rim + rim_dir * (1.2 * s * (1.0 - t));
                    gizmos.line_2d(p_rim, out, core.with_alpha(0.5 * (1.0 - t)));
                }

                // Droplets (sparks) flung outward; fewer for perf
                for i in 0..4 {
                    let f = i as f32 / 3.0 - 0.5; // -0.5..0.5
                    let dir = (outward + tangent * (0.6 * f)).normalize_or_zero();
                    let pos_d = impact + dir * (10.0 * s * ease_out);
                    let r = (1.1 * s * (1.0 - t)).max(0.2);
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


