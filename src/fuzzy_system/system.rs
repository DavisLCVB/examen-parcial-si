use std::{collections::HashMap, fmt::Display};
use std::fmt;

use crate::fuzzy_system::{DefuzzificationMethod, FuzzyRule, LinguisticVariable};

// Conditional printing macro - only prints when CLI feature is enabled
#[cfg(feature = "cli")]
macro_rules! fuzzy_eprintln {
    ($($arg:tt)*) => {
        eprintln!($($arg)*)
    };
}

#[cfg(not(feature = "cli"))]
macro_rules! fuzzy_eprintln {
    ($($arg:tt)*) => {};
}

pub struct FuzzySystem{
    pub name: String,
    pub input_variables: Vec<LinguisticVariable>,
    pub output_variable: LinguisticVariable,
    pub rules: Vec<FuzzyRule>,
    pub defuzzification_method: DefuzzificationMethod,
}

impl FuzzySystem {
    pub fn new<N: Into<String>>(
        name: N,
    ) -> Self {
        FuzzySystem {
            name: name.into(),
            input_variables: Vec::new(),
            output_variable: LinguisticVariable::new("output", (0.0, 1.0)),
            rules: Vec::new(),
            defuzzification_method: DefuzzificationMethod::Centroid,
        }
    }

    pub fn add_input(&mut self, variable: LinguisticVariable) {
        self.input_variables.push(variable);
    }

    pub fn add_rule(&mut self, rule: FuzzyRule) {
        self.rules.push(rule);
    }

    pub fn set_output(&mut self, variable: LinguisticVariable) {
        self.output_variable = variable;
    }

    pub fn evaluate(&self, inputs: &HashMap<String, f64>) -> (String, f64){
        // Validate that all required input variables are present
        for var in &self.input_variables {
            if !inputs.contains_key(&var.name) {
                fuzzy_eprintln!("Warning: Input variable '{}' not found in inputs. Using default value 0.0", var.name);
            }
        }

        // Fuzzification phase
        let mut fuzzyfied_inputs = HashMap::new();
        for var in &self.input_variables {
            if let Some(&value) = inputs.get(&var.name) {
                // Validate input is within expected range
                if value < var.range.0 || value > var.range.1 {
                    fuzzy_eprintln!("Warning: Input '{}' = {} is outside expected range {:?}",
                             var.name, value, var.range);
                }
                fuzzyfied_inputs.insert(var.name.clone(), var.fuzzify(value));
            }
        }

        // Rule evaluation and aggregation phase
        let mut activated_outputs: HashMap<String, f64> = HashMap::new();
        let mut any_rule_fired = false;

        for rule in &self.rules {
            let degree = rule.evaluate(&fuzzyfied_inputs);
            if degree > f64::EPSILON {
                any_rule_fired = true;
            }
            for consequent in &rule.consequents {
                // Validate consequent references valid output set
                if !self.output_variable.fuzzy_sets.iter().any(|s| s.name == consequent.set) {
                    fuzzy_eprintln!("Warning: Consequent set '{}' not found in output variable '{}'",
                             consequent.set, self.output_variable.name);
                    continue;
                }
                let entry = activated_outputs.entry(consequent.set.clone()).or_insert(0.0);
                *entry = entry.max(degree);
            }
        }

        if !any_rule_fired {
            fuzzy_eprintln!("Warning: No rules were activated for inputs {:?}", inputs);
        }

        // Defuzzification phase
        let defuzzified_value = match self.defuzzification_method {
            DefuzzificationMethod::Centroid => {
                crate::fuzzy_system::Defuzzifier::centroid(&self.output_variable, &activated_outputs)
            }
        };
        (self.output_variable.name.clone(), defuzzified_value)
    }
}

impl Display for FuzzySystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "FuzzySystem: {}", self.name)?;

        writeln!(f, "Input variables:")?;
        for var in &self.input_variables {
            writeln!(f, "  - {} (range: {:?})", var.name, var.range)?;
            for set in &var.fuzzy_sets {
                writeln!(f, "      · {}", set.name)?;
            }
        }

        writeln!(f, "Output variable:")?;
        writeln!(f, "  - {} (range: {:?})", self.output_variable.name, self.output_variable.range)?;
        for set in &self.output_variable.fuzzy_sets {
            writeln!(f, "      · {}", set.name)?;
        }

        writeln!(f, "Rules:")?;
        for (i, rule) in self.rules.iter().enumerate() {
            let antecedents: Vec<String> = rule
                .antecedents
                .iter()
                .map(|a| format!("{} is {}", a.variable, a.set))
                .collect();
            let consequents: Vec<String> = rule
                .consequents
                .iter()
                .map(|c| format!("{} is {}", c.variable, c.set))
                .collect();
            let op = match rule.operator {
                crate::fuzzy_system::RuleOperator::And => "AND",
                crate::fuzzy_system::RuleOperator::Or => "OR",
            };

            writeln!(f, "  {}: if {} {} then {}", i + 1, antecedents.join(" "), op, consequents.join(", "))?;
        }

        writeln!(f, "Defuzzification: {:?}", self.defuzzification_method)
    }
}


