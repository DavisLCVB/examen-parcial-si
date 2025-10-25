use std::collections::HashMap;



pub struct Antecedent {
    pub set: String,
    pub variable: String,
}

pub struct Consequent {
    pub set: String,
    pub variable: String,
}

impl Antecedent {
    pub fn new(set: &str, variable: &str) -> Self {
        Self {
            set: set.to_string(),
            variable: variable.to_string(),
        }
    }
}

impl Consequent {
    pub fn new(set: &str, variable: &str) -> Self {
        Self {
            set: set.to_string(),
            variable: variable.to_string(),
        }
    }
}

pub enum RuleOperator {
    And,
    Or,
}

pub struct FuzzyRule {
    pub antecedents: Vec<Antecedent>,
    pub consequents: Vec<Consequent>,
    pub operator: RuleOperator,
}

impl FuzzyRule {
    pub fn new(
        antecedents: Vec<Antecedent>,
        consequents: Vec<Consequent>,
        operator: RuleOperator,
    ) -> Self {
        Self {
            antecedents,
            consequents,
            operator,
        }
    }

    pub fn evaluate(&self, inputs: &HashMap<String, HashMap<String, f64>>) -> f64 {
        let mut degrees = Vec::new();

        for antecedent in &self.antecedents {
            if let Some(var_membership) = inputs.get(&antecedent.variable) {
                if let Some(degree) = var_membership.get(&antecedent.set) {
                    degrees.push(*degree);
                }
            }
        }

        if degrees.is_empty() {
            return 0.0;
        }

        match self.operator {
            RuleOperator::And => {
                // For AND, start with first element and apply min with rest
                degrees.into_iter().reduce(|acc, x| acc.min(x)).unwrap_or(0.0)
            }
            RuleOperator::Or => {
                // For OR, start with first element and apply max with rest
                degrees.into_iter().reduce(|acc, x| acc.max(x)).unwrap_or(0.0)
            }
        }
    }   
}