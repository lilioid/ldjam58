use crate::physics::calc_gravity::Attractor;
use crate::physics::directional_forces::ThrustForce;
use crate::physics::velocity::Velocity;
use bevy::prelude::*;
use std::ops::Neg;
use crate::launching::Fuel;

pub const THRUSTER_KEY: KeyCode = KeyCode::Space;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
#[allow(unused)]
pub enum ThrusterDirection {
    /** Towards the velocity vector **/
    Prograde,
    /** Opposite direction of the velocity vector **/
    Retrograde,
    /** Decrease orbit size **/
    RadialIn,
    /** Increase orbit size **/
    RadialOut,
}

#[derive(Component, Debug, PartialEq)]
#[require(ThrustForce, Velocity, Transform)]
pub struct Thruster {
    pub active: bool,
    pub strength: f32,
    pub direction: ThrusterDirection,
}

impl Thruster {
    pub fn new(direction: ThrusterDirection, strength: f32) -> Self {
        Self {
            active: false,
            direction,
            strength,
        }
    }
}

pub fn toggle_thruster(mut query: Query<(&mut Thruster, Option<&Name>)>) {
    query.iter_mut().for_each(|(mut thruster, name)| {
        info!(
            "Toggling thruster of {} {}",
            match name {
                Some(name) => name.as_str(),
                None => "unknown",
            },
            if thruster.active { "off" } else { "on" },
        );

        thruster.active = !thruster.active;
    })
}

pub fn apply_thrust_force(
    mut query: Query<(&Thruster, &Velocity, &Transform, &mut ThrustForce)>,
    attractor: Query<&Transform, With<Attractor>>,
) {
    let Some(attractor) = attractor.iter().next() else { return; };

    query
        .iter_mut()
        .for_each(|(i_thruster, i_velocity, i_trans, mut i_thrust_force)| {
            if i_thruster.active {
                let direction = match i_thruster.direction {
                    ThrusterDirection::Prograde => i_velocity.0,
                    ThrusterDirection::Retrograde => i_velocity.0.neg(),
                    ThrusterDirection::RadialIn => {
                        let center = attractor.translation.xy();
                        let i_pos = i_trans.translation.xy();
                        let relative_position = i_pos - center;
                        let rotation = if relative_position.angle_to(i_velocity.0) < 0.0 {
                            -1.0
                        } else {
                            1.0
                        };
                        (relative_position.perp() * rotation).neg()
                    },
                    ThrusterDirection::RadialOut => {
                        let center = attractor.translation.xy();
                        let i_pos = i_trans.translation.xy();
                        let offset = i_pos - center;
                        let rotation = if offset.angle_to(i_velocity.0) < 0.0 {
                            -1.0
                        } else {
                            1.0
                        };
                        offset.perp() * rotation
                    }
                };

                i_thrust_force.0 = direction.clamp_length(1.0, 1.0) * i_thruster.strength;
            }
        });
}

pub fn thruster_use_fuel(mut thruster_query: Query<(&mut Thruster, &mut Fuel)>, time: Res<Time>) {
    for (mut thruster, mut fuel) in thruster_query.iter_mut() {
        if thruster.active && fuel.amount <= 0.0 {
            thruster.active = false;
        } else if thruster.active && fuel.amount > 0.0 {
            fuel.amount -= time.delta_secs();
            if fuel.amount < 0.0 {
                fuel.amount = 0.0;
            }
        }
    }
}
