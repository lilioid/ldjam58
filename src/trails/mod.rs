use bevy::prelude::*;
use crate::GameplaySystem;
use crate::sun_system::{Level, Satellite};
use crate::physics::velocity::Velocity;

pub struct TrailsPlugin;

impl Plugin for TrailsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (record_history, render_trails).in_set(GameplaySystem));
        app.add_systems(PostUpdate, attach_trails_to_satellites);
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub enum TrailStyle {

    Ribbon,
}

#[derive(Component)]
pub struct Trail {
    pub style: TrailStyle,
    pub color: Color,
    pub max_points: usize,   // stored tail length
    pub spacing: f32,        // world units between samples
}

impl Default for Trail {
    fn default() -> Self {
        Self {
            style: TrailStyle::Ribbon,
            // muted teal fits retro NASA; tweak per style if desired
            color: Color::srgb(0.35, 0.9, 0.95),
            max_points: 17,
            spacing: 4.0,
        }
    }
}

#[derive(Component, Default)]
pub struct TrailHistory {
    points: Vec<Vec2>,
    dist_accum: f32,
}

fn attach_trails_to_satellites(
    mut commands: Commands,
    q: Query<(Entity, &Level), (With<Satellite>, Without<Trail>, With<Velocity>)>,
) {

    for (e, level) in q.iter() {
        // Skip level-1 satellites (where despawns are most likely in your setup)
        if level.level <= 1.0 {
            continue;
        }
        // Safe, deferred insert: only inserts if entity still exists at apply-time
        commands.entity(e).insert((Trail::default(), TrailHistory::default()));
        
    }
}

fn record_history(
    time: Res<Time>,
    mut q: Query<(&Transform, &Velocity, &Trail, &mut TrailHistory)>,
) {
    for (t, vel, trail, mut hist) in q.iter_mut() {
        let p = t.translation.xy();
        hist.dist_accum += vel.0.length() * time.delta_secs();
        if hist.points.is_empty() || hist.dist_accum >= trail.spacing {
            hist.points.push(p);
            hist.dist_accum = 0.0;
            if hist.points.len() > trail.max_points {
                let overflow = hist.points.len() - trail.max_points;
                hist.points.drain(0..overflow);
            }
        }
    }
}

fn render_trails(
    mut gizmos: Gizmos,
    q: Query<( &Trail, &TrailHistory)>,
) {
    for ( trail, hist) in q.iter() {
        let c = trail.color;
        match trail.style {

            TrailStyle::Ribbon => {
                let nseg = hist.points.len().saturating_sub(1).max(1);
                for (i, w) in hist.points.windows(2).enumerate() {
                    let t = i as f32 / nseg as f32; // 0 = tail, 1 = head
                    let a_side = 0.25 * t;          // fade outer ribbons
                    let a_center = 0.05 + 0.70 * t; // faint tail -> bright head
                    gizmos.line_2d(w[0] + Vec2::Y * 1.6, w[1] + Vec2::Y * 1.6, c.with_alpha(a_side));
                    gizmos.line_2d(w[0] - Vec2::Y * 1.6, w[1] - Vec2::Y * 1.6, c.with_alpha(a_side));
                    gizmos.line_2d(w[0], w[1], c.with_alpha(a_center));
                }
            }
        }
    }
}