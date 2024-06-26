extern crate rand;
extern crate wasm_bindgen;
extern crate web_sys;

mod utils;
use std::f64::consts::PI;
use std::string::FromUtf8Error;
use std::vec;

use wasm_bindgen::prelude::*;
use web_sys::js_sys::Math::cos;
use web_sys::js_sys::Math::sin;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

/// Represents a Swarmalator system with agents.
///
/// # Fields
/// - `agents`: Number of agents.
/// - `A`, `B`: Coefficients for velocity contributions.
/// - `K`, `J`: Coupling constants.
/// - `chiral`: Boolean indicating if the system is chiral.
/// - `target`: Optional target positions.
/// - `inherent_velocities`: Inherent velocities of the agents.
/// - `natural_frequencies`: Natural frequencies of the agents.
/// - `c`: Additional constant values.
/// - `velocities`: Current velocities of the agents.
/// - `phases`: Current phases of the agents.
/// - `delta_phases`: Changes in phases.
/// - `positions`: Current positions of the agents.
#[wasm_bindgen]
pub struct Swarmalator {
    agents: usize,
    A: f64,
    B: f64,
    K: f64,
    J: f64,
    target: Option<Vec<f64>>,
    natural_frequencies: Vec<f64>,
    chiral: Option<Vec<f64>>,
    velocities: Vec<f64>,
    phases: Vec<f64>,
    delta_phases: Vec<f64>,
    positions: Vec<f64>,
}

#[wasm_bindgen]
impl Swarmalator {
    /// Creates a new Swarmalator instance.
    ///
    /// # Arguments
    /// - `agents`: Number of agents.
    /// - `positions`: Initial positions of the agents.
    /// - `phases`: Initial phases of the agents.
    /// - `natural_frequencies`: Natural frequencies of the agents.
    /// - `K`: Phase coupling coefficient
    /// - `J`: Spatial-phase interaction coefficient
    /// - `chiral`: Optional chiral values
    /// - `target`: Optional target positions.
    ///
    /// # Panics
    /// Panics if the length of `positions` is not equal to `2 * agents`.
    #[wasm_bindgen(constructor)]
    pub fn new(
        agents: usize,
        positions: Vec<f64>,
        phases: Vec<f64>,
        natural_frequencies: Vec<f64>,
        K: f64,
        J: f64,
        chiral: Option<Vec<f64>>,
        target: Option<Vec<f64>>,
    ) -> Swarmalator {
        utils::set_panic_hook();

        // Check the length of the arrays
        if positions.len() != agents * 2 {
            panic!("Positions array must have 2 * agents elements")
        }

        if phases.len() != agents {
            panic!("Phases array must have agents elements")
        }

        if natural_frequencies.len() != agents {
            panic!("Natural frequencies array must have agents elements")
        }

        // All agents start stationary
        let velocities: Vec<f64> = vec![0.0; agents * 2];

        // We store delta_phase so we get the dt from update
        let delta_phases: Vec<f64> = vec![0.0; agents];

        // If we have a target, set it
        let target: Option<Vec<f64>> = match target {
            Some(t) => Some(t.clone()),
            None => None,
        };

        Swarmalator {
            agents,
            A: 1.0,
            B: 1.0,
            K,
            J,
            chiral,
            target,
            natural_frequencies,
            velocities,
            phases,
            delta_phases,
            positions: positions.clone(),
        }
    }

    /// Updates the state of the Swarmalator system.
    ///
    /// # Arguments
    /// - `dt`: Time step for the update.
    pub fn update(&mut self, dt: f64) {
        let mut Js = vec![self.J; self.agents];

        // If we have a target we need to recalculate the J values
        if let Some(target) = self.target.as_ref() {
            let mut dists_to_target = vec![0.0; self.agents];
            for i in 0..self.agents {
                dists_to_target[i] = ((self.positions[i * 2] - target[0]).powi(2)
                    + (self.positions[i * 2 + 1] - target[1]).powi(2))
                .sqrt();
            }

            let max_dist = dists_to_target.iter().fold(0.0 / 0.0, |m, v| v.max(m));
            let min_dist = dists_to_target.iter().fold(0.0 / 0.0, |m, v| v.min(m));

            for i in 0..self.agents {
                Js[i] = self.A * f64::abs(dists_to_target[i] - min_dist) / (max_dist - min_dist);
            }
        }

        for i in 0..self.agents {
            if let Some(chiral) = self.chiral.as_ref() {
                self.velocities[i * 2] = chiral[i] * cos(self.phases[i] + PI / 2.0);
                self.velocities[i * 2 + 1] = chiral[i] * sin(self.phases[i] + PI / 2.0);
            } else {
                self.velocities[i * 2] = 0.0;
                self.velocities[i * 2 + 1] = 0.0;
            }

            // Natural frequnecy always contributes to delta phase
            self.delta_phases[i] = self.natural_frequencies[i];

            for j in 0..self.agents {
                if i == j {
                    continue;
                }

                let dist: f64 = ((self.positions[i * 2] - self.positions[j * 2]).powi(2)
                    + (self.positions[i * 2 + 1] - self.positions[j * 2 + 1]).powi(2))
                .sqrt();

                // We may have frequency coupling
                let mut freq_diff_xy: f64 = 0.0;
                let mut freq_diff_phase: f64 = 0.0;

                if self.chiral.is_some() {
                    freq_diff_xy = (PI / 2.0)
                        * f64::abs(
                            self.natural_frequencies[j] / f64::abs(self.natural_frequencies[j])
                                - self.natural_frequencies[i]
                                    / f64::abs(self.natural_frequencies[i]),
                        );

                    freq_diff_phase = freq_diff_xy / 2.0;
                }

                let velocity_contribution_x: f64 =
                    ((self.positions[j * 2] - self.positions[i * 2]) / dist)
                        * (self.A + Js[i] * cos(self.phases[j] - self.phases[i] - freq_diff_xy))
                        - (self.B * (self.positions[j * 2] - self.positions[i * 2]) / dist.powi(2));

                let velocity_contribution_y: f64 =
                    ((self.positions[j * 2 + 1] - self.positions[i * 2 + 1]) / dist)
                        * (self.A + Js[i] * cos(self.phases[j] - self.phases[i] - freq_diff_xy))
                        - (self.B * (self.positions[j * 2 + 1] - self.positions[i * 2 + 1])
                            / dist.powi(2));

                self.velocities[i * 2] += (1.0 / self.agents as f64) * velocity_contribution_x;
                self.velocities[i * 2 + 1] += (1.0 / self.agents as f64) * velocity_contribution_y;

                self.delta_phases[i] += (self.K / (self.agents as f64))
                    * sin(self.phases[j] - self.phases[i] - freq_diff_phase)
                    / dist;
            }
        }

        for i in 0..self.agents {
            self.phases[i] += self.delta_phases[i] * dt;
            self.phases[i] = self.phases[i] % (2.0 * PI);

            self.positions[i * 2] += self.velocities[i * 2] * dt;
            self.positions[i * 2 + 1] += self.velocities[i * 2 + 1] * dt;
        }
    }

    /// Returns a pointer to the velocities array.
    pub fn velocities(&self) -> *const f64 {
        self.velocities.as_ptr()
    }

    /// Returns a pointer to the phases array.
    pub fn phases(&self) -> *const f64 {
        self.phases.as_ptr()
    }

    /// Returns a pointer to the positions array.
    pub fn positions(&self) -> *const f64 {
        self.positions.as_ptr()
    }

    /// Update the target position.
    /// # Arguments
    /// - `target`: New target position.
    /// # Panics
    /// Panics if the length of `target` is not equal to 2.
    pub fn set_target(&mut self, target: Vec<f64>) {
        if target.len() != 2 {
            panic!("Target array must have 2 elements")
        }

        self.target = Some(target);
    }

    /// Set the phase coupling coefficient.
    /// # Arguments
    /// - `K`: New value for K
    pub fn set_K(&mut self, K: f64) {
        self.K = K;
    }

    /// Set the spatial-phase interaction coefficient.
    /// # Arguments
    /// - `J`: New value for J
    pub fn set_J(&mut self, J: f64) {
        self.J = J;
    }

    /// Set the chiral values.
    /// # Arguments
    /// - `chiral`: New chiral values.
    pub fn set_chiral(&mut self, chiral: Option<Vec<f64>>) {
        self.chiral = chiral;
    }

    /// Set the natural frequencies.
    /// # Arguments
    /// - `natural_frequencies`: New natural frequencies.
    /// # Panics
    /// Panics if the length of `natural_frequencies` is not equal to the number of agents.
    pub fn set_natural_frequencies(&mut self, natural_frequencies: Vec<f64>) {
        if natural_frequencies.len() != self.agents {
            panic!("Natural frequencies array must have agents elements")
        }

        self.natural_frequencies = natural_frequencies;
    }

    /// Set the phases
    /// # Arguments
    /// - `phases`: New phases.
    ///
    /// # Panics
    /// Panics if the length of `phases` is not equal to the number of agents.
    pub fn set_phases(&mut self, phases: Vec<f64>) {
        if phases.len() != self.agents {
            panic!("Phases array must have agents elements")
        }

        self.phases = phases;
    }
}
