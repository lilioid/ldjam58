use bevy::prelude::*;
use bevy::math::primitives::Circle;
use bevy::prelude::Mesh;
// Mesh2d and MeshMaterial2d are re-exported by the Bevy prelude in 0.17

use crate::screens::Screen;
use crate::sun_system::Sun;
use crate::AppSystems;

/// Marks particles spawned by the sun emitter
#[derive(Component)]
struct SunParticle {
    velocity: Vec2,
    lifetime: f32,
    age: f32,
    start_color: Color,
    end_color: Color,
    start_size: f32,
    max_distance: f32,
    material: Handle<ColorMaterial>,
}

/// Component attached to the `Sun` entity to emit particles
#[derive(Component)]
struct SunEmitter {
    radius: f32,
    spawn_per_tick: usize,
    timer: Timer,
    base_angle: f32,
}

#[derive(Resource)]
struct SunParticleResources {
	circle_mesh: Handle<Mesh>,
}

impl FromWorld for SunParticleResources {
	fn from_world(world: &mut World) -> Self {
		let mut meshes = world.resource_mut::<Assets<Mesh>>();
		let circle_mesh = meshes.add(Mesh::from(Circle::new(1.0)));
		Self { circle_mesh }
	}
}

pub(super) fn plugin(app: &mut App) {
	app.init_resource::<SunParticleResources>();
	app.add_systems(
		OnEnter(Screen::Gameplay),
		attach_emitter_on_enter,
	);

	app.add_systems(
		Update,
		attach_emitter_on_added.in_set(AppSystems::Update),
	);

	app.add_systems(
		Update,
		(emit_particles, update_particles)
			.run_if(in_state(Screen::Gameplay))
			.in_set(AppSystems::Update),
	);
}

/// Attach an emitter to any existing `Sun` when gameplay starts
fn attach_emitter_on_enter(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    suns: Query<(Entity, &Transform, &Sprite), (With<Sun>, Without<SunEmitter>)>,
) {
    for (e, transform, sprite) in &suns {
        let radius = images
            .get(&sprite.image)
            .map(|img| {
                // Use half the displayed width as the edge of the sprite
                let px_size = img.size();
                0.5 * px_size.x as f32 * transform.scale.x
            })
            .unwrap_or(90.0);

        commands.entity(e).insert(SunEmitter {
            radius,
            spawn_per_tick: 12,
            timer: Timer::from_seconds(0.02, TimerMode::Repeating),
            base_angle: 0.0,
        });
    }
}

/// If a `Sun` is spawned later, attach the emitter then
fn attach_emitter_on_added(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    suns: Query<(Entity, &Transform, &Sprite), Added<Sun>>,
) {
    for (e, transform, sprite) in &suns {
        let radius = images
            .get(&sprite.image)
            .map(|img| {
                let px_size = img.size();
                0.5 * px_size.x as f32 * transform.scale.x
            })
            .unwrap_or(90.0);

        commands.entity(e).insert(SunEmitter {
            radius,
            spawn_per_tick: 12,
            timer: Timer::from_seconds(0.02, TimerMode::Repeating),
            base_angle: 0.0,
        });
    }
}

fn emit_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(&Transform, &mut SunEmitter), With<Sun>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (transform, mut emitter) in &mut q {
        emitter.timer.tick(time.delta());
        if !emitter.timer.finished() {
            continue;
        }

        // Advance the emission angle slowly so the ring shimmers over time
        emitter.base_angle = (emitter.base_angle + 0.35) % std::f32::consts::TAU;

        let center = transform.translation.truncate();
        let count = emitter.spawn_per_tick.max(1) as f32;
        let angle_step = std::f32::consts::TAU / count;

        for i in 0..emitter.spawn_per_tick {
            let i_f = i as f32;
            let angle = emitter.base_angle + i_f * angle_step;
            let dir = Vec2::from_angle(angle);
            let pos = center + dir * emitter.radius;

            // Deterministic variation using trigs (avoid external RNG dep)
            let speed = 160.0 + 50.0 * (angle * 1.7).sin().abs();
            let swirl_amt = 25.0 * (angle * 2.3).cos();
            let tangent = Vec2::new(-dir.y, dir.x);
            let velocity = dir * speed + tangent * swirl_amt;

            let lifetime = 0.9 + 0.3 * (angle * 0.9).cos().abs();
            let start_size = 14.0 + 6.0 * (angle * 1.1).sin().abs();

            let start_color = Color::srgba(1.0, 0.9, 0.4, 0.95);
            let end_color = Color::srgba(1.0, 0.2, 0.0, 0.0);

            let mesh = meshes.add(Mesh::from(Circle::new(1.0)));
            let material = materials.add(ColorMaterial::from(start_color));

            commands.spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(pos.x, pos.y, 0.01).with_scale(Vec3::splat(start_size)),
                SunParticle {
                    velocity,
                    lifetime,
                    age: 0.0,
                    start_color,
                    end_color,
                    start_size,
                    max_distance: emitter.radius + 30.0,
                    material,
                },
            ));
        }
    }
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    sun_q: Query<&Transform, (With<Sun>, Without<SunParticle>)>,
    mut q: Query<(Entity, &mut Transform, &mut SunParticle), Without<Sun>>,
) {
    let dt = time.delta().as_secs_f32();
    let sun_center = sun_q.single().map(|t| t.translation.truncate()).unwrap_or(Vec2::ZERO);
	for (e, mut transform, mut p) in &mut q {
		p.age += dt;
		if p.age >= p.lifetime {
			commands.entity(e).despawn();
			continue;
		}

		let drag = 0.98_f32.powf(60.0 * dt);
		p.velocity *= drag;
		transform.translation.x += p.velocity.x * dt;
		transform.translation.y += p.velocity.y * dt;

		let dist = transform.translation.truncate().distance(sun_center);
		if dist > p.max_distance {
			commands.entity(e).despawn();
			continue;
		}

		let t = (p.age / p.lifetime).clamp(0.0, 1.0);
		let bell = 1.0 + 0.4 * (1.0 - (2.0 * t - 1.0).abs());
		let shrink = 1.0 - 0.6 * t;
		let size = (p.start_size * bell * shrink).max(1.0);
		transform.scale = Vec3::splat(size);

		let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
		let sc = p.start_color.to_linear();
		let ec = p.end_color.to_linear();
		let r = lerp(sc.red,   ec.red,   t);
		let g = lerp(sc.green, ec.green, t);
		let b = lerp(sc.blue,  ec.blue,  t);
		let a = lerp(sc.alpha, ec.alpha, t);
		if let Some(mat) = materials.get_mut(&p.material) {
			mat.color = Color::from(bevy::color::LinearRgba { red: r, green: g, blue: b, alpha: a });
		}
	}
}


