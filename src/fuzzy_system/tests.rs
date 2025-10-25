#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_triangular_membership() {
        let tri = triangular(0.0, 5.0, 10.0);

        // Test boundaries
        assert_eq!(tri.evaluate(-1.0), 0.0);
        assert_eq!(tri.evaluate(11.0), 0.0);

        // Test peak
        assert!((tri.evaluate(5.0) - 1.0).abs() < f64::EPSILON);

        // Test slopes
        assert!((tri.evaluate(2.5) - 0.5).abs() < 0.01);
        assert!((tri.evaluate(7.5) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_trapezoidal_membership() {
        let trap = trapezoidal(0.0, 3.0, 7.0, 10.0);

        // Test boundaries
        assert_eq!(trap.evaluate(-1.0), 0.0);
        assert_eq!(trap.evaluate(11.0), 0.0);

        // Test plateau
        assert_eq!(trap.evaluate(5.0), 1.0);
        assert_eq!(trap.evaluate(3.0), 1.0);
        assert_eq!(trap.evaluate(7.0), 1.0);

        // Test slopes
        assert!((trap.evaluate(1.5) - 0.5).abs() < 0.01);
        assert!((trap.evaluate(8.5) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_gaussian_membership() {
        let gauss = gaussian(5.0, 1.0);

        // Test peak at mean
        assert!((gauss.evaluate(5.0) - 1.0).abs() < f64::EPSILON);

        // Test symmetry
        let left = gauss.evaluate(3.0);
        let right = gauss.evaluate(7.0);
        assert!((left - right).abs() < 0.01);

        // Test that it decreases away from mean
        assert!(gauss.evaluate(5.0) > gauss.evaluate(6.0));
        assert!(gauss.evaluate(6.0) > gauss.evaluate(7.0));
    }

    #[test]
    fn test_sigmoidal_membership() {
        let sig = sigmoidal(1.0, 5.0);

        // Test that it's approximately 0.5 at center
        assert!((sig.evaluate(5.0) - 0.5).abs() < 0.01);

        // Test monotonic increase (for positive a)
        assert!(sig.evaluate(3.0) < sig.evaluate(5.0));
        assert!(sig.evaluate(5.0) < sig.evaluate(7.0));

        // Test bounds
        assert!(sig.evaluate(0.0) > 0.0 && sig.evaluate(0.0) < 1.0);
        assert!(sig.evaluate(10.0) > 0.0 && sig.evaluate(10.0) < 1.0);
    }

    #[test]
    #[should_panic(expected = "a <= b <= c")]
    fn test_triangular_validation() {
        triangular(10.0, 5.0, 0.0); // Invalid: decreasing order
    }

    #[test]
    #[should_panic(expected = "a <= b <= c <= d")]
    fn test_trapezoidal_validation() {
        trapezoidal(10.0, 5.0, 3.0, 0.0); // Invalid: decreasing order
    }

    #[test]
    #[should_panic(expected = "sigma > 0")]
    fn test_gaussian_validation() {
        gaussian(5.0, -1.0); // Invalid: negative sigma
    }

    #[test]
    #[should_panic(expected = "a != 0")]
    fn test_sigmoidal_validation() {
        sigmoidal(0.0, 5.0); // Invalid: a = 0
    }

    #[test]
    fn test_fuzzy_and_operation() {
        assert_eq!(FuzzyOperation::and(&0.3, &0.7), 0.3);
        assert_eq!(FuzzyOperation::and(&0.8, &0.4), 0.4);
        assert_eq!(FuzzyOperation::and(&1.0, &0.5), 0.5);
    }

    #[test]
    fn test_fuzzy_or_operation() {
        assert_eq!(FuzzyOperation::or(&0.3, &0.7), 0.7);
        assert_eq!(FuzzyOperation::or(&0.8, &0.4), 0.8);
        assert_eq!(FuzzyOperation::or(&0.0, &0.5), 0.5);
    }

    #[test]
    fn test_fuzzy_not_operation() {
        assert!((FuzzyOperation::not(&0.3) - 0.7).abs() < f64::EPSILON);
        assert!((FuzzyOperation::not(&1.0) - 0.0).abs() < f64::EPSILON);
        assert!((FuzzyOperation::not(&0.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rule_evaluation_and() {
        let mut inputs = HashMap::new();
        let mut var1_membership = HashMap::new();
        var1_membership.insert("low".to_string(), 0.7);
        var1_membership.insert("high".to_string(), 0.3);

        let mut var2_membership = HashMap::new();
        var2_membership.insert("cold".to_string(), 0.8);
        var2_membership.insert("hot".to_string(), 0.2);

        inputs.insert("var1".to_string(), var1_membership);
        inputs.insert("var2".to_string(), var2_membership);

        let rule = FuzzyRule::new(
            vec![
                Antecedent::new("low", "var1"),
                Antecedent::new("cold", "var2"),
            ],
            vec![Consequent::new("output_low", "output")],
            RuleOperator::And,
        );

        let result = rule.evaluate(&inputs);
        assert!((result - 0.7).abs() < f64::EPSILON); // min(0.7, 0.8) = 0.7
    }

    #[test]
    fn test_rule_evaluation_or() {
        let mut inputs = HashMap::new();
        let mut var1_membership = HashMap::new();
        var1_membership.insert("low".to_string(), 0.3);
        var1_membership.insert("high".to_string(), 0.7);

        let mut var2_membership = HashMap::new();
        var2_membership.insert("cold".to_string(), 0.2);
        var2_membership.insert("hot".to_string(), 0.8);

        inputs.insert("var1".to_string(), var1_membership);
        inputs.insert("var2".to_string(), var2_membership);

        let rule = FuzzyRule::new(
            vec![
                Antecedent::new("low", "var1"),
                Antecedent::new("cold", "var2"),
            ],
            vec![Consequent::new("output_low", "output")],
            RuleOperator::Or,
        );

        let result = rule.evaluate(&inputs);
        assert!((result - 0.3).abs() < f64::EPSILON); // max(0.3, 0.2) = 0.3
    }

    #[test]
    fn test_fuzzification() {
        let mut var = LinguisticVariable::new("temperature", (0.0, 100.0));
        var.add_set(FuzzySet::new("cold", triangular(0.0, 0.0, 50.0)));
        var.add_set(FuzzySet::new("hot", triangular(50.0, 100.0, 100.0)));

        let memberships = var.fuzzify(25.0);

        assert!(memberships.contains_key("cold"));
        assert!(memberships.contains_key("hot"));
        assert!(memberships["cold"] > 0.4 && memberships["cold"] < 0.6);
        assert!(memberships["hot"] < 0.1);
    }

    #[test]
    fn test_complete_fuzzy_system() {
        let mut system = FuzzySystem::new("Test System");

        // Input variable
        let mut temp_var = LinguisticVariable::new("temperature", (0.0, 100.0));
        temp_var.add_set(FuzzySet::new("cold", triangular(0.0, 0.0, 50.0)));
        temp_var.add_set(FuzzySet::new("hot", triangular(50.0, 100.0, 100.0)));
        system.add_input(temp_var);

        // Output variable
        let mut fan_var = LinguisticVariable::new("fan_speed", (0.0, 100.0));
        fan_var.add_set(FuzzySet::new("low", triangular(0.0, 0.0, 50.0)));
        fan_var.add_set(FuzzySet::new("high", triangular(50.0, 100.0, 100.0)));
        system.set_output(fan_var);

        // Rules
        let rule1 = FuzzyRule::new(
            vec![Antecedent::new("cold", "temperature")],
            vec![Consequent::new("low", "fan_speed")],
            RuleOperator::And,
        );
        system.add_rule(rule1);

        let rule2 = FuzzyRule::new(
            vec![Antecedent::new("hot", "temperature")],
            vec![Consequent::new("high", "fan_speed")],
            RuleOperator::And,
        );
        system.add_rule(rule2);

        // Test cold temperature
        let mut inputs = HashMap::new();
        inputs.insert("temperature".to_string(), 25.0);
        let (output_name, output_value) = system.evaluate(&inputs);

        assert_eq!(output_name, "fan_speed");
        assert!(output_value < 50.0); // Should be in low range

        // Test hot temperature
        inputs.insert("temperature".to_string(), 75.0);
        let (_, output_value) = system.evaluate(&inputs);
        assert!(output_value > 50.0); // Should be in high range
    }

    #[test]
    fn test_defuzzification_centroid() {
        let mut output_var = LinguisticVariable::new("output", (0.0, 100.0));
        output_var.add_set(FuzzySet::new("low", triangular(0.0, 25.0, 50.0)));
        output_var.add_set(FuzzySet::new("high", triangular(50.0, 75.0, 100.0)));

        let mut activated = HashMap::new();
        activated.insert("low".to_string(), 0.5);
        activated.insert("high".to_string(), 0.5);

        let result = Defuzzifier::centroid(&output_var, &activated);

        // With equal activation, result should be near center
        assert!(result > 40.0 && result < 60.0);
    }

    #[test]
    fn test_defuzzification_no_activation() {
        let mut output_var = LinguisticVariable::new("output", (0.0, 100.0));
        output_var.add_set(FuzzySet::new("low", triangular(0.0, 25.0, 50.0)));

        let activated = HashMap::new(); // No activation

        let result = Defuzzifier::centroid(&output_var, &activated);

        // Should return midpoint
        assert!((result - 50.0).abs() < f64::EPSILON);
    }
}
