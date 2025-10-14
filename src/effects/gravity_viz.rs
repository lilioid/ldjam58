use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::GameplaySystem;
use crate::sun_system::Sun;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GravityViz>();
    app.add_systems(
        Update,
        cycle_mode.run_if(input_just_pressed(KeyCode::KeyU)).in_set(GameplaySystem),
    );
    app.add_systems(Update, draw_viz.in_set(GameplaySystem));
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum VizMode {
    #[default]
    Off,
    Equipotential,
    VectorField,
    GravityWell,
    SlingshotCues,
    Streamlines,
    ParticleFlow,
    LensingGrid,
    Heatmap,
    RadialTicks,
    Scanlines,
    Hatches,
    Spirals,
    PolarRulers,
    Isochrones,
}

#[derive(Resource, Debug)]
struct GravityViz {
    mode: VizMode,
}

impl Default for GravityViz {
    fn default() -> Self { Self { mode: VizMode::Equipotential } }
}

fn cycle_mode(mut viz: ResMut<GravityViz>) {
    viz.mode = match viz.mode {
        VizMode::Off => VizMode::Equipotential,
        VizMode::Equipotential => VizMode::VectorField,
        VizMode::VectorField => VizMode::GravityWell,
        VizMode::GravityWell => VizMode::SlingshotCues,
        VizMode::SlingshotCues => VizMode::Streamlines,
        VizMode::Streamlines => VizMode::ParticleFlow,
        VizMode::ParticleFlow => VizMode::LensingGrid,
        VizMode::LensingGrid => VizMode::Heatmap,
        VizMode::Heatmap => VizMode::RadialTicks,
        VizMode::RadialTicks => VizMode::Scanlines,
        VizMode::Scanlines => VizMode::Hatches,
        VizMode::Hatches => VizMode::Spirals,
        VizMode::Spirals => VizMode::PolarRulers,
        VizMode::PolarRulers => VizMode::Isochrones,
        VizMode::Isochrones => VizMode::Off,
    };
    info!("Gravity viz mode: {:?}", viz.mode);
}

fn draw_viz(
    mut gizmos: Gizmos,
    viz: Res<GravityViz>,
    sun_q: Query<&GlobalTransform, With<Sun>>,
    cam_q: Query<(&Camera, &GlobalTransform, &Transform)>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    if viz.mode == VizMode::Off { return; }
    let Some(gt) = sun_q.iter().next() else { return; };
    let center = gt.translation().xy();

    // Compute current view bounds to fill the screen regardless of zoom
    let Some(window) = window_q.iter().next() else { return; };
    let Some((_, cam_gt, cam_t)) = cam_q.iter().next() else { return; };
    let scale = cam_t.scale.x.max(0.0001);
    let half_w = window.width() * 0.5 / scale;
    let half_h = window.height() * 0.5 / scale;
    let view_center = cam_gt.translation().xy();
    let start = view_center - Vec2::new(half_w, half_h);
    let end = view_center + Vec2::new(half_w, half_h);

    match viz.mode {
        VizMode::Equipotential => draw_iso_potentials(&mut gizmos, center, end),
        VizMode::VectorField => draw_vector_field(&mut gizmos, center, start, end),
        VizMode::GravityWell => draw_well_grid(&mut gizmos, center, end),
        VizMode::SlingshotCues => draw_slingshot_cues(&mut gizmos, center),
        VizMode::Streamlines => draw_streamlines(&mut gizmos, center, start, end),
        VizMode::ParticleFlow => draw_particle_flow(&mut gizmos, center, start, end),
        VizMode::LensingGrid => draw_lensing_grid(&mut gizmos, center, start, end),
        VizMode::Heatmap => draw_heatmap(&mut gizmos, center, end),
        VizMode::RadialTicks => draw_radial_ticks(&mut gizmos, center, end),
        VizMode::Scanlines => draw_scanlines(&mut gizmos, start, end),
        VizMode::Hatches => draw_hatches(&mut gizmos, start, end),
        VizMode::Spirals => draw_spirals(&mut gizmos, center, end),
        VizMode::PolarRulers => draw_polar_rulers(&mut gizmos, center, end),
        VizMode::Isochrones => draw_isochrones(&mut gizmos, center, end),
        VizMode::Off => {}
    }
}

fn draw_iso_potentials(gizmos: &mut Gizmos, center: Vec2, end: Vec2) {
    let iso = Isometry2d::from_translation(center);
    // Expand circles to fill current view
    let max_r = center.distance(end) * 1.1;
    let mut r = 16.0;
    while r <= max_r {
        let alpha = (1.0 / (1.0 + r * 0.04)).min(0.6);
        let color = Color::srgb(0.9, 0.8, 0.4).with_alpha(alpha * 0.4);
        gizmos.circle_2d(iso, r, color);
        r += 24.0;
    }
}

fn draw_vector_field(gizmos: &mut Gizmos, center: Vec2, start: Vec2, end: Vec2) {
    let step = 64.0;
    let color = Color::srgb(0.6, 0.9, 0.6).with_alpha(0.55);
    let mut y = start.y;
    while y <= end.y {
        let mut x = start.x;
        while x <= end.x {
            let p = Vec2::new(x, y);
            let dir = center - p;
            let dist = dir.length().max(1.0);
            let n = dir / dist;
            let len = (120.0 / dist).clamp(3.0, 14.0);
            gizmos.arrow_2d(p, p + n * len, color);
            x += step;
        }
        y += step;
    }
}

fn draw_well_grid(gizmos: &mut Gizmos, center: Vec2, end: Vec2) {
    // Radial lines
    let spokes = 24;
    let radius = center.distance(end) * 1.1;
    for i in 0..spokes {
        let a = i as f32 / spokes as f32 * std::f32::consts::TAU;
        let dir = Vec2::from_angle(a);
        let p0 = center + dir * 16.0;
        let p1 = center + dir * radius;
        let color = Color::srgb(0.3, 0.9, 0.9).with_alpha(0.25);
        gizmos.line_2d(p0, p1, color);
    }
    // Circular bands
    let mut r = 24.0;
    let color = Color::srgb(0.3, 0.9, 0.9).with_alpha(0.18);
    let iso = Isometry2d::from_translation(center);
    while r <= radius {
        gizmos.circle_2d(iso, r, color);
        r += 36.0;
    }
}

fn draw_slingshot_cues(gizmos: &mut Gizmos, center: Vec2) {
    // Minimal: a faint ring at the sun to suggest curvature intensity
    let iso = Isometry2d::from_translation(center);
    let color = Color::srgb(0.2, 0.8, 1.0).with_alpha(0.2);
    gizmos.circle_2d(iso, 34.0, color);
}

fn draw_streamlines(gizmos: &mut Gizmos, center: Vec2, start: Vec2, end: Vec2) {
    let step = 72.0;
    let h = 10;
    let color = Color::srgb(0.7, 0.9, 1.0).with_alpha(0.36);
    let mut y = start.y;
    while y <= end.y {
        let mut x = start.x;
        while x <= end.x {
            let mut p = Vec2::new(x, y);
            for _ in 0..h {
                let dir = center - p;
                let dist = dir.length().max(1.0);
                let n = dir / dist;
                let next = p + n * 20.0;
                gizmos.line_2d(p, next, color);
                p = next;
            }
            x += step;
        }
        y += step;
    }
}

fn draw_particle_flow(gizmos: &mut Gizmos, center: Vec2, start: Vec2, end: Vec2) {
    let color = Color::srgb(0.5, 0.9, 1.0).with_alpha(0.25);
    let mut y = start.y;
    while y <= end.y {
        let mut x = start.x;
        while x <= end.x {
            let p = Vec2::new(x, y);
            let r = (p.distance(center) * 0.02).clamp(0.5, 2.5);
            let iso = Isometry2d::from_translation(p);
            gizmos.circle_2d(iso, r, color);
            x += 64.0;
        }
        y += 64.0;
    }
}

fn draw_lensing_grid(gizmos: &mut Gizmos, center: Vec2, start: Vec2, end: Vec2) {
    let color = Color::srgb(0.6, 0.9, 1.0).with_alpha(0.25);
    // Vertical curves
    let mut x = start.x;
    while x <= end.x {
        let mut y = start.y;
        let mut last = Vec2::new(x, y);
        while y <= end.y {
            let p = Vec2::new(x, y);
            let v = p - center;
            let bend = (1000.0 / (1.0 + v.length())).min(40.0);
            let n = v.perp().normalize_or_zero();
            let cur = p + n * bend * 0.02;
            gizmos.line_2d(last, cur, color);
            last = cur;
            y += 28.0;
        }
        x += 56.0;
    }
    // Horizontal curves
    let mut y = start.y;
    while y <= end.y {
        let mut x = start.x;
        let mut last = Vec2::new(x, y);
        while x <= end.x {
            let p = Vec2::new(x, y);
            let v = p - center;
            let bend = (1000.0 / (1.0 + v.length())).min(40.0);
            let n = v.perp().normalize_or_zero();
            let cur = p - n * bend * 0.02;
            gizmos.line_2d(last, cur, color);
            last = cur;
            x += 56.0;
        }
        y += 56.0;
    }
}

fn draw_heatmap(gizmos: &mut Gizmos, center: Vec2, end: Vec2) {
    let iso = Isometry2d::from_translation(center);
    let max_r = center.distance(end) * 1.2;
    let mut r = 12.0;
    while r <= max_r {
        let a = (1.0 / (1.0 + r * 0.02)).clamp(0.05, 0.35);
        let color = Color::srgb(1.0, 0.5, 0.2).with_alpha(a);
        gizmos.circle_2d(iso, r, color);
        r += 8.0;
    }
}

fn draw_radial_ticks(gizmos: &mut Gizmos, center: Vec2, end: Vec2) {
    let max_r = center.distance(end) * 1.1;
    let spokes = 36;
    for s in 0..spokes {
        let a = s as f32 / spokes as f32 * std::f32::consts::TAU;
        let dir = Vec2::from_angle(a);
        let mut r = 24.0;
        while r <= max_r {
            let p = center + dir * r;
            let t = Vec2::new(-dir.y, dir.x);
            gizmos.line_2d(p - t * 2.0, p + t * 2.0, Color::srgb(0.9, 0.8, 0.4).with_alpha(0.4));
            r += 24.0;
        }
    }
}

fn draw_scanlines(gizmos: &mut Gizmos, start: Vec2, end: Vec2) {
    let color = Color::srgb(0.2, 0.8, 1.0).with_alpha(0.10);
    let mut y = start.y;
    while y <= end.y {
        gizmos.line_2d(Vec2::new(start.x, y), Vec2::new(end.x, y), color);
        y += 12.0;
    }
}

fn draw_hatches(gizmos: &mut Gizmos, start: Vec2, end: Vec2) {
    let color = Color::srgb(0.9, 0.8, 0.4).with_alpha(0.12);
    let diag = (end - start).length();
    let mut t = -diag;
    while t <= diag {
        let p0 = Vec2::new(start.x + t, start.y);
        let p1 = Vec2::new(start.x + t + (end.y - start.y), end.y);
        gizmos.line_2d(p0, p1, color);
        t += 22.0;
    }
}

fn draw_spirals(gizmos: &mut Gizmos, center: Vec2, end: Vec2) {
    let color = Color::srgb(0.2, 0.8, 1.0).with_alpha(0.35);
    let max_r = center.distance(end) * 1.0;
    let arms = 3;
    for k in 0..arms {
        let mut a = k as f32 * 2.0 * std::f32::consts::PI / arms as f32;
        let mut r = 8.0;
        let mut last = center + Vec2::from_angle(a) * r;
        while r <= max_r {
            a += 0.20;
            r *= 1.04;
            let cur = center + Vec2::from_angle(a) * r;
            gizmos.line_2d(last, cur, color);
            last = cur;
        }
    }
}

fn draw_polar_rulers(gizmos: &mut Gizmos, center: Vec2, end: Vec2) {
    let iso = Isometry2d::from_translation(center);
    let max_r = center.distance(end) * 1.1;
    let mut r = 20.0;
    while r <= max_r {
        gizmos.circle_2d(iso, r, Color::srgb(0.6, 0.9, 1.0).with_alpha(0.18));
        r += 20.0;
    }
    let spokes = 24;
    for i in 0..spokes {
        let a = i as f32 / spokes as f32 * std::f32::consts::TAU;
        let dir = Vec2::from_angle(a);
        gizmos.line_2d(center, center + dir * max_r, Color::srgb(0.6, 0.9, 1.0).with_alpha(0.08));
    }
}

fn draw_isochrones(gizmos: &mut Gizmos, center: Vec2, end: Vec2) {
    let iso = Isometry2d::from_translation(center);
    let max_r = center.distance(end) * 1.2;
    let mut r = 28.0;
    while r <= max_r {
        gizmos.circle_2d(iso, r, Color::srgb(0.9, 0.8, 0.4).with_alpha(0.22));
        r *= 1.12;
    }
}


