use std::vec;

use examen_parcial::fuzzy_system::{triangular, Antecedent, Consequent, FuzzyRule, FuzzySet, FuzzySystem, LinguisticVariable, RuleOperator};


fn main() {
    // Air conditioner control system
    // Note: This AC system considers both cooling AND dehumidification
    // High humidity requires more AC power even at lower temperatures for dehumidification
    let mut system = FuzzySystem::new("Air conditioner system");

    // Input variables
    let mut temp_var = LinguisticVariable::new("Temperature", (0.0, 100.0));
    temp_var.add_set(FuzzySet::new("very cold", triangular(0.0, 0.0, 20.0)));
    temp_var.add_set(FuzzySet::new("cold", triangular(10.0, 25.0, 40.0)));
    temp_var.add_set(FuzzySet::new("warm", triangular(30.0, 50.0, 70.0)));
    temp_var.add_set(FuzzySet::new("hot", triangular(60.0, 80.0, 100.0)));
    system.add_input(temp_var);

    let mut humidity_var = LinguisticVariable::new("Humidity", (0.0, 100.0));
    humidity_var.add_set(FuzzySet::new("low", triangular(0.0, 0.0, 40.0)));
    humidity_var.add_set(FuzzySet::new("medium", triangular(30.0, 50.0, 70.0)));
    humidity_var.add_set(FuzzySet::new("high", triangular(60.0, 100.0, 100.0)));
    system.add_input(humidity_var);

    // Output variable
    let mut ac_power_var = LinguisticVariable::new("AC Power", (0.0, 100.0));
    ac_power_var.add_set(FuzzySet::new("low", triangular(0.0, 0.0, 40.0)));
    ac_power_var.add_set(FuzzySet::new("medium", triangular(30.0, 50.0, 70.0)));
    ac_power_var.add_set(FuzzySet::new("high", triangular(60.0, 100.0, 100.0)));
    system.set_output(ac_power_var);

    // Define rules
    // Rule 1: High humidity requires dehumidification even in cold conditions
    let rule1 = FuzzyRule::new(
        vec![Antecedent::new("very cold", "Temperature"), Antecedent::new("high", "Humidity")],
        vec![Consequent::new("high", "AC Power")],
        RuleOperator::And
    );
    system.add_rule(rule1);

    // Rule 2: Moderate conditions need moderate AC power
    let rule2 = FuzzyRule::new(
        vec![Antecedent::new("cold", "Temperature"), Antecedent::new("medium", "Humidity")],
        vec![Consequent::new("medium", "AC Power")],
        RuleOperator::And
    );
    system.add_rule(rule2);

    // Rule 3: Comfortable conditions need minimal AC
    let rule3 = FuzzyRule::new(
        vec![Antecedent::new("warm", "Temperature"), Antecedent::new("low", "Humidity")],
        vec![Consequent::new("low", "AC Power")],
        RuleOperator::And
    );
    system.add_rule(rule3);

    // Rule 4: Hot and humid conditions need maximum AC power
    let rule4 = FuzzyRule::new(
        vec![Antecedent::new("hot", "Temperature"), Antecedent::new("high", "Humidity")],
        vec![Consequent::new("high", "AC Power")],
        RuleOperator::And
    );
    system.add_rule(rule4);

    // Rule 5: Hot but dry conditions need moderate cooling
    let rule5 = FuzzyRule::new(
        vec![Antecedent::new("hot", "Temperature"), Antecedent::new("low", "Humidity")],
        vec![Consequent::new("medium", "AC Power")],
        RuleOperator::And
    );
    system.add_rule(rule5);

    // low AC power when very cold or cold test
    let inputs = vec![
        ("Temperature".to_string(), 15.0),
        ("Humidity".to_string(), 80.0),
    ].into_iter().collect();
    let (output_var, output_value) = system.evaluate(&inputs);
    println!("For inputs {:?}, the output '{}' is {:.2}", inputs, output_var, output_value);

    // high AC power when hot and high humidity test
    let inputs = vec![
        ("Temperature".to_string(), 90.0),
        ("Humidity".to_string(), 85.0),
    ].into_iter().collect();
    let (output_var, output_value) = system.evaluate(&inputs);
    println!("For inputs {:?}, the output '{}' is {:.2}", inputs, output_var, output_value);

    // medium AC power when warm and low humidity test
    let inputs = vec![
        ("Temperature".to_string(), 55.0),
        ("Humidity".to_string(), 35.0),
    ].into_iter().collect();
    let (output_var, output_value) = system.evaluate(&inputs);
    println!("For inputs {:?}, the output '{}' is {:.2}", inputs, output_var, output_value);

    println!("Fuzzy system evaluation completed.");
    println!("-----------------------------------");
    println!("System details:\n{}", system);

}