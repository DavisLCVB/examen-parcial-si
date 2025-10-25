use std::fmt::Debug;

use crate::fuzzy_system::MembershipFunction;


pub struct FuzzySet{
    pub name: String,
    pub membership_function: Box<dyn MembershipFunction + Send + Sync>,
}

impl Debug for FuzzySet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FuzzySet")
            .field("name", &self.name)
            .finish()
    }
}


impl FuzzySet {
    pub fn new<N: Into<String>>(name: N, membership_function: Box<dyn MembershipFunction + Send + Sync>) -> Self {
        FuzzySet {
            name: name.into(),
            membership_function,
        }
    }

    pub fn evaluate(&self, input: f64) -> f64 {
        self.membership_function.evaluate(input)
    }
}

pub struct FuzzyOperation;

impl FuzzyOperation{
    pub fn and(a: &f64, b: &f64) -> f64 {
        a.min(*b)
    }

    pub fn or(a: &f64, b: &f64) -> f64 {
        a.max(*b)
    }

    pub fn not(a: &f64) -> f64 {
        1.0 - *a
    }
}