//! Hidden synthetic worlds and their protected evaluator (T-003).
//!
//! This crate is deliberately **not** a dependency of the `hephaestus`
//! control-plane crate: the permission scope separating workers/control-plane
//! from hidden ground truth is the Cargo dependency graph itself, and
//! `crates/hephaestus/tests/evaluator_access.rs` denies any attempt to add
//! it.
//!
//! What invention workers (or the control plane) may ever see is
//! [`Observation`]: input/output pairs only. Ground truth — mechanism form,
//! parameters, confounders, feasibility, regime breaks — lives in [`World`],
//! which this crate never serializes into an observation stream.
//!
//! Determinism: inputs are a stratified grid (no RNG), noise is Gaussian via
//! a 12-uniform Irwin–Hall sum driven by SplitMix64 — basic IEEE operations
//! only, so observation receipts are bit-stable across platforms.

use serde::{Deserialize, Serialize};

mod corpus;
mod evaluate;
mod rng;

mod trace_fixture;
pub use corpus::{CorpusEntry, Probe, load_corpus, observation_receipt};
pub use evaluate::{AMBIGUITY_BAND, Assessment, PARAM_TOL, Verdict, evaluate};
pub use trace_fixture::trace_fixture;

/// Scenario classes the fixture corpus must cover (IMPLEMENTATION_PLAN T-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scenario {
    TrueMechanism,
    FalseMechanism,
    Confounder,
    ImpossibleConstraint,
    AmbiguousEvidence,
    ChangedBoundary,
}

/// Parametric forms worlds and candidates share.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// y = a
    Constant,
    /// y = a + b·x
    Linear,
    /// y = a + b/x  (domains exclude 0)
    Inverse,
}

impl Form {
    pub fn predict(self, a: f64, b: f64, x: f64) -> f64 {
        match self {
            Form::Constant => a,
            Form::Linear => a + b * x,
            Form::Inverse => a + b / x,
        }
    }

    /// Whether this form claims an x-dependent (causal-looking) effect.
    pub fn claims_input_effect(self, b: f64) -> bool {
        match self {
            Form::Constant => false,
            Form::Linear | Form::Inverse => b != 0.0,
        }
    }
}

/// A piecewise regime start: for x >= break_at, this regime's form applies.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Regime {
    pub break_at: f64,
    pub form: Form,
    pub a: f64,
    pub b: f64,
}

/// An impossible output constraint: the world can never reach
/// output <= output_max (T-003 impossible constraints).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub output_max: f64,
}

/// Hidden ground truth of one synthetic world. Never serialized into
/// observation streams.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub id: String,
    pub scenario: Scenario,
    /// Sample domain [x_min, x_max], excluding 0 for inverse forms.
    pub domain: [f64; 2],
    /// Observation noise standard deviation.
    pub noise_sigma: f64,
    /// RNG seed for noise (inputs are a deterministic grid).
    pub seed: u64,
    /// Data-generating form below the first regime break (or whole domain).
    pub truth_form: Form,
    pub truth_a: f64,
    pub truth_b: f64,
    /// False in confounder worlds: the curve is driven by an unobserved
    /// confounder, so the input has no causal effect on the output.
    pub causal_effect_of_input: bool,
    /// Empty for single-regime worlds.
    #[serde(default)]
    pub regimes: Vec<Regime>,
    #[serde(default)]
    pub requirement: Option<Requirement>,
}

/// What workers see: input/output pairs only. The deny tests assert this
/// projection can never carry ground truth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub input: f64,
    pub output: f64,
}

/// A candidate explanation offered by an invention worker.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub form: Form,
    pub a: f64,
    pub b: f64,
}

impl Candidate {
    pub fn predict(&self, x: f64) -> f64 {
        self.form.predict(self.a, self.b, x)
    }
}

impl World {
    /// The active truth form at input x (piecewise across regime breaks).
    pub fn truth_at(&self, x: f64) -> (Form, f64, f64) {
        let mut active = (self.truth_form, self.truth_a, self.truth_b);
        for regime in &self.regimes {
            if x >= regime.break_at {
                active = (regime.form, regime.a, regime.b);
            }
        }
        active
    }

    fn stratified_inputs(&self) -> Vec<f64> {
        const CELLS: usize = 64;
        let (lo, hi) = (self.domain[0], self.domain[1]);
        let step = (hi - lo) / CELLS as f64;
        (0..CELLS).map(|i| lo + (i as f64 + 0.5) * step).collect()
    }

    /// Deterministic observation stream: stratified grid inputs, Gaussian
    /// noise from the world's seed. Two calls with the same arguments are
    /// bit-identical.
    pub fn sample_observations(&self) -> Vec<Observation> {
        let mut noise = rng::NoiseSource::new(self.seed);
        self.stratified_inputs()
            .into_iter()
            .map(|input| {
                let (form, a, b) = self.truth_at(input);
                let clean = form.predict(a, b, input);
                Observation {
                    input,
                    output: clean + self.noise_sigma * noise.next_gaussian(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(scenario: Scenario) -> World {
        World {
            id: "W-TEST".to_string(),
            scenario,
            domain: [10.0, 100.0],
            noise_sigma: 0.3,
            seed: 42,
            truth_form: Form::Inverse,
            truth_a: 20.0,
            truth_b: 500.0,
            causal_effect_of_input: true,
            regimes: Vec::new(),
            requirement: None,
        }
    }

    #[test]
    fn sampling_is_bit_deterministic() {
        let w = world(Scenario::TrueMechanism);
        let a = w.sample_observations();
        let b = w.sample_observations();
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn different_seeds_produce_different_noise() {
        let mut w = world(Scenario::TrueMechanism);
        let first = w.sample_observations();
        w.seed = 43;
        let second = w.sample_observations();
        assert_ne!(first, second);
        for (x, y) in first.iter().zip(&second) {
            assert_eq!(x.input, y.input, "inputs are seed-independent");
        }
    }

    #[test]
    fn grid_covers_the_domain() {
        let w = world(Scenario::TrueMechanism);
        let obs = w.sample_observations();
        assert!(obs.iter().all(|o| o.input > 10.0 && o.input < 100.0));
        let inputs: Vec<f64> = obs.iter().map(|o| o.input).collect();
        assert!(
            inputs.windows(2).all(|w| w[0] < w[1]),
            "strictly increasing"
        );
    }

    #[test]
    fn regime_table_selects_piecewise_truth() {
        let mut w = world(Scenario::ChangedBoundary);
        w.regimes = vec![Regime {
            break_at: 55.0,
            form: Form::Linear,
            a: 30.0,
            b: 0.2,
        }];
        assert_eq!(w.truth_at(30.0), (Form::Inverse, 20.0, 500.0));
        assert_eq!(w.truth_at(80.0), (Form::Linear, 30.0, 0.2));
        // Sampling respects the piecewise generator.
        let obs = w.sample_observations();
        assert_eq!(obs.len(), 64);
    }

    #[test]
    fn observations_serialize_to_input_output_only() {
        let w = world(Scenario::Confounder);
        let obs = w.sample_observations();
        let value = serde_json::to_value(&obs).unwrap();
        let array = value.as_array().unwrap();
        let keys: std::collections::HashSet<String> = array
            .iter()
            .flat_map(|o| o.as_object().unwrap().keys().cloned())
            .collect();
        let allowed: std::collections::HashSet<String> =
            ["input", "output"].into_iter().map(String::from).collect();
        assert_eq!(keys, allowed, "observation projection must stay minimal");
    }

    #[test]
    fn gaussian_noise_is_centered_enough_for_fixtures() {
        let mut noise = rng::NoiseSource::new(7);
        let samples: Vec<f64> = (0..4096).map(|_| noise.next_gaussian()).collect();
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let var =
            samples.iter().map(|s| (s - mean) * (s - mean)).sum::<f64>() / samples.len() as f64;
        assert!(mean.abs() < 0.05, "mean {mean}");
        assert!((0.85..1.15).contains(&var), "variance {var}");
    }
}
