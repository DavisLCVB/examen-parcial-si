use std::{collections::HashMap, fmt::Debug};

use crate::fuzzy_system::FuzzySet;


pub struct LinguisticVariable {
    pub name: String,
    pub fuzzy_sets: Vec<FuzzySet>,
    pub range: (f64, f64),
}

impl LinguisticVariable {
    pub fn new(name: &str, range: (f64, f64)) -> Self {
        Self {
            name: name.to_string(),
            fuzzy_sets: Vec::new(),
            range,
        }
    }

    pub fn add_set(&mut self, fuzzy_set: FuzzySet) {
        self.fuzzy_sets.push(fuzzy_set);
    }

    pub fn fuzzify(&self, value: f64) -> HashMap<String, f64> {
        self.fuzzy_sets.iter().map(|set| (set.name.clone(), set.evaluate(value))).collect()
    }
}

pub enum DefuzzificationMethod {
    Centroid,
}

impl Debug for DefuzzificationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefuzzificationMethod::Centroid => write!(f, "Centroid"),
        }
    }
}

pub struct Defuzzifier;

impl Defuzzifier {
    /// Centroid defuzzification method using numerical integration
    /// Computes: ∫ x·μ(x) dx / ∫ μ(x) dx
    /// where μ(x) is the aggregated membership function (max of all activated sets)
    pub fn centroid(output_var: &LinguisticVariable, activated: &HashMap<String, f64>) -> f64 {
        let steps = 1000; // Increased resolution for better accuracy
        let step_size = (output_var.range.1 - output_var.range.0) / steps as f64;
        let mut numerator = 0.0;
        let mut denominator = 0.0;

        // Numerical integration using trapezoidal rule
        for i in 0..=steps {
            let x = output_var.range.0 + i as f64 * step_size;

            // Compute aggregated membership at point x (max over all activated sets)
            let mut aggregated_membership: f64 = 0.0;
            for set in &output_var.fuzzy_sets {
                if let Some(&activation_degree) = activated.get(&set.name) {
                    // Apply implication (min between activation degree and membership)
                    let membership_at_x = set.evaluate(x);
                    let clipped_membership = membership_at_x.min(activation_degree);
                    aggregated_membership = aggregated_membership.max(clipped_membership);
                }
            }

            numerator += x * aggregated_membership;
            denominator += aggregated_membership;
        }

        if denominator < f64::EPSILON {
            // No rules activated, return midpoint of range
            return (output_var.range.0 + output_var.range.1) / 2.0;
        }

        numerator / denominator
    }

}