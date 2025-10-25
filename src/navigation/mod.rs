// Navigation module - Fuzzy logic controller for vehicle navigation

use crate::fuzzy_system::{
    triangular, trapezoidal, Antecedent, Consequent, FuzzyRule, FuzzySet, FuzzySystem,
    LinguisticVariable, RuleOperator,
};
use crate::vehicle::VehicleCharacteristics;
use std::collections::HashMap;
use std::f64::consts::PI;

/// Navigation controller using fuzzy logic
pub struct NavigationController {
    fuzzy_system: FuzzySystem,
    _maneuverability: f64,  // Reserved for future use
    _max_acceleration: f64,  // Reserved for future use
}

impl NavigationController {
    /// Create a simplified navigation controller for a vehicle
    ///
    /// Inputs:
    /// - distancia_al_objetivo: [0, 1000]
    /// - error_angular: [-180°, 180°]
    /// - velocidad_relativa: [0, 1]
    ///
    /// Outputs:
    /// - ajuste_angular: [-maneuverability, +maneuverability]
    /// - ajuste_velocidad: [-max_accel, +max_accel] (not used - constant velocity)
    ///
    /// Rules: 10 rules covering all distance-angle combinations
    pub fn new(characteristics: &VehicleCharacteristics) -> Self {
        let mut system = FuzzySystem::new("Navigation Controller");

        let maneuverability = characteristics.maneuverability;
        let max_accel = characteristics.max_acceleration;

        // INPUT 1: distancia_al_objetivo [0, 1000]
        let mut dist_var = LinguisticVariable::new("distancia_al_objetivo", (0.0, 1000.0));
        dist_var.add_set(FuzzySet::new("muy_cerca", trapezoidal(0.0, 0.0, 50.0, 100.0)));
        dist_var.add_set(FuzzySet::new("media", triangular(80.0, 200.0, 400.0)));
        dist_var.add_set(FuzzySet::new("lejos", trapezoidal(350.0, 500.0, 1000.0, 1000.0)));
        system.add_input(dist_var);

        // INPUT 2: error_angular [-180°, 180°]
        // Negative angles = target is to the left, need to turn left
        // Positive angles = target is to the right, need to turn right
        let mut error_var = LinguisticVariable::new("error_angular", (-PI, PI));
        error_var.add_set(FuzzySet::new(
            "alineado",
            trapezoidal(-10f64.to_radians(), -5f64.to_radians(), 5f64.to_radians(), 10f64.to_radians()),
        ));
        error_var.add_set(FuzzySet::new(
            "desviado_izq",
            triangular(-90f64.to_radians(), -45f64.to_radians(), -10f64.to_radians()),
        ));
        error_var.add_set(FuzzySet::new(
            "desviado_der",
            triangular(10f64.to_radians(), 45f64.to_radians(), 90f64.to_radians()),
        ));
        // Very deviated: covers angles beyond ±90°
        error_var.add_set(FuzzySet::new(
            "muy_desviado_izq",
            trapezoidal(-PI, -150f64.to_radians(), -120f64.to_radians(), -70f64.to_radians()),
        ));
        error_var.add_set(FuzzySet::new(
            "muy_desviado_der",
            trapezoidal(70f64.to_radians(), 120f64.to_radians(), 150f64.to_radians(), PI),
        ));
        system.add_input(error_var);

        // INPUT 3: velocidad_relativa [0, 1] (normalized)
        let mut vel_var = LinguisticVariable::new("velocidad_relativa", (0.0, 1.0));
        vel_var.add_set(FuzzySet::new("lenta", triangular(0.0, 0.0, 0.3)));
        vel_var.add_set(FuzzySet::new("media", triangular(0.2, 0.5, 0.8)));
        vel_var.add_set(FuzzySet::new("rapida", trapezoidal(0.7, 1.0, 1.0, 1.0)));
        system.add_input(vel_var);

        // OUTPUT 1: ajuste_angular [-maneuverability, +maneuverability]
        let mut ang_out_var = LinguisticVariable::new("ajuste_angular", (-maneuverability, maneuverability));
        ang_out_var.add_set(FuzzySet::new(
            "girar_izq",
            triangular(-maneuverability, -0.7 * maneuverability, -0.3 * maneuverability),
        ));
        ang_out_var.add_set(FuzzySet::new(
            "leve_izq",
            triangular(-0.4 * maneuverability, -0.2 * maneuverability, 0.0),
        ));
        ang_out_var.add_set(FuzzySet::new(
            "mantener",
            triangular(-0.1 * maneuverability, 0.0, 0.1 * maneuverability),
        ));
        ang_out_var.add_set(FuzzySet::new(
            "leve_der",
            triangular(0.0, 0.2 * maneuverability, 0.4 * maneuverability),
        ));
        ang_out_var.add_set(FuzzySet::new(
            "girar_der",
            triangular(0.3 * maneuverability, 0.7 * maneuverability, maneuverability),
        ));
        system.set_output(ang_out_var);

        // OUTPUT 2: ajuste_velocidad [-max_accel, +max_accel]
        // Note: Using a separate system would be cleaner, but for simplicity we'll use
        // a single system with two outputs by encoding velocity rules similarly

        // RULES (simplified version)

        // R1: SI lejos Y alineado ENTONCES mantener_rumbo Y acelerar_fuerte
        system.add_rule(FuzzyRule::new(
            vec![
                Antecedent::new("lejos", "distancia_al_objetivo"),
                Antecedent::new("alineado", "error_angular"),
            ],
            vec![Consequent::new("mantener", "ajuste_angular")],
            RuleOperator::And,
        ));

        // R2: SI lejos Y desviado_der ENTONCES girar_der
        system.add_rule(FuzzyRule::new(
            vec![
                Antecedent::new("lejos", "distancia_al_objetivo"),
                Antecedent::new("desviado_der", "error_angular"),
            ],
            vec![Consequent::new("girar_der", "ajuste_angular")],
            RuleOperator::And,
        ));

        // R3: SI lejos Y desviado_izq ENTONCES girar_izq
        system.add_rule(FuzzyRule::new(
            vec![
                Antecedent::new("lejos", "distancia_al_objetivo"),
                Antecedent::new("desviado_izq", "error_angular"),
            ],
            vec![Consequent::new("girar_izq", "ajuste_angular")],
            RuleOperator::And,
        ));

        // R4: SI media Y alineado ENTONCES mantener
        system.add_rule(FuzzyRule::new(
            vec![
                Antecedent::new("media", "distancia_al_objetivo"),
                Antecedent::new("alineado", "error_angular"),
            ],
            vec![Consequent::new("mantener", "ajuste_angular")],
            RuleOperator::And,
        ));

        // R5: SI media Y desviado_der ENTONCES leve_der
        system.add_rule(FuzzyRule::new(
            vec![
                Antecedent::new("media", "distancia_al_objetivo"),
                Antecedent::new("desviado_der", "error_angular"),
            ],
            vec![Consequent::new("leve_der", "ajuste_angular")],
            RuleOperator::And,
        ));

        // R6: SI media Y desviado_izq ENTONCES leve_izq
        system.add_rule(FuzzyRule::new(
            vec![
                Antecedent::new("media", "distancia_al_objetivo"),
                Antecedent::new("desviado_izq", "error_angular"),
            ],
            vec![Consequent::new("leve_izq", "ajuste_angular")],
            RuleOperator::And,
        ));

        // R7: SI muy_cerca Y alineado ENTONCES mantener
        system.add_rule(FuzzyRule::new(
            vec![
                Antecedent::new("muy_cerca", "distancia_al_objetivo"),
                Antecedent::new("alineado", "error_angular"),
            ],
            vec![Consequent::new("mantener", "ajuste_angular")],
            RuleOperator::And,
        ));

        // R8a: SI muy_desviado_izq ENTONCES girar fuerte izquierda
        system.add_rule(FuzzyRule::new(
            vec![Antecedent::new("muy_desviado_izq", "error_angular")],
            vec![Consequent::new("girar_izq", "ajuste_angular")],
            RuleOperator::And,
        ));

        // R8b: SI muy_desviado_der ENTONCES girar fuerte derecha
        system.add_rule(FuzzyRule::new(
            vec![Antecedent::new("muy_desviado_der", "error_angular")],
            vec![Consequent::new("girar_der", "ajuste_angular")],
            RuleOperator::And,
        ));

        // R9: SI muy_cerca Y desviado_izq ENTONCES leve_izq
        system.add_rule(FuzzyRule::new(
            vec![
                Antecedent::new("muy_cerca", "distancia_al_objetivo"),
                Antecedent::new("desviado_izq", "error_angular"),
            ],
            vec![Consequent::new("leve_izq", "ajuste_angular")],
            RuleOperator::And,
        ));

        // R10: SI muy_cerca Y desviado_der ENTONCES leve_der
        system.add_rule(FuzzyRule::new(
            vec![
                Antecedent::new("muy_cerca", "distancia_al_objetivo"),
                Antecedent::new("desviado_der", "error_angular"),
            ],
            vec![Consequent::new("leve_der", "ajuste_angular")],
            RuleOperator::And,
        ));

        Self {
            fuzzy_system: system,
            _maneuverability: maneuverability,
            _max_acceleration: max_accel,
        }
    }

    /// Compute control output for angular adjustment
    ///
    /// Velocity is kept constant for simplicity - only the steering angle is controlled
    pub fn compute_control(
        &self,
        distance_to_target: f64,
        angular_error: f64,
        velocity_relative: f64,
    ) -> (f64, f64) {
        // Evaluate fuzzy system for angular adjustment
        let mut inputs = HashMap::new();
        inputs.insert("distancia_al_objetivo".to_string(), distance_to_target);
        inputs.insert("error_angular".to_string(), angular_error);
        inputs.insert("velocidad_relativa".to_string(), velocity_relative);

        let (_, angular_adjustment) = self.fuzzy_system.evaluate(&inputs);

        // Velocity is constant - no adjustment
        let velocity_adjustment = 0.0;

        (angular_adjustment, velocity_adjustment)
    }
}
