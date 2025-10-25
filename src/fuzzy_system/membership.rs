
pub trait MembershipFunction {
    fn evaluate(&self, input: f64) -> f64;
}

pub struct TriangularMembershipFunction {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl MembershipFunction for TriangularMembershipFunction {
    fn evaluate(&self, input: f64) -> f64 {
        if input < self.a || input > self.c {
            0.0
        } else if (input - self.b).abs() < f64::EPSILON {
            1.0
        } else if input < self.b {
            let denominator = self.b - self.a;
            if denominator.abs() < f64::EPSILON {
                0.0
            } else {
                (input - self.a) / denominator
            }
        } else {
            let denominator = self.c - self.b;
            if denominator.abs() < f64::EPSILON {
                0.0
            } else {
                (self.c - input) / denominator
            }
        }
    }
}

pub struct TrapezoidalMembershipFunction {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

impl MembershipFunction for TrapezoidalMembershipFunction {
    fn evaluate(&self, input: f64) -> f64 {
        if input < self.a || input > self.d {
            0.0
        } else if input >= self.b && input <= self.c {
            1.0
        } else if input < self.b {
            let denominator = self.b - self.a;
            if denominator.abs() < f64::EPSILON {
                0.0
            } else {
                (input - self.a) / denominator
            }
        } else {
            let denominator = self.d - self.c;
            if denominator.abs() < f64::EPSILON {
                0.0
            } else {
                (self.d - input) / denominator
            }
        }
    }
}

pub struct GaussianMembershipFunction {
    pub mean: f64,
    pub sigma: f64,
}

impl MembershipFunction for GaussianMembershipFunction {
    fn evaluate(&self, input: f64) -> f64 {
        let exponent = -((input - self.mean).powi(2)) / (2.0 * self.sigma.powi(2));
        exponent.exp()
    }
}

pub struct SigmoidalMembershipFunction {
    pub a: f64,
    pub c: f64,
}

impl MembershipFunction for SigmoidalMembershipFunction {
    fn evaluate(&self, input: f64) -> f64 {
        1.0 / (1.0 + (-self.a * (input - self.c)).exp())
    }
}

//helpers

pub fn triangular(a: f64, b: f64, c: f64) -> Box<TriangularMembershipFunction> {
    assert!(a <= b && b <= c, "Triangular membership function requires a <= b <= c");
    Box::new(TriangularMembershipFunction { a, b, c })
}

pub fn trapezoidal(a: f64, b: f64, c: f64, d: f64) -> Box<TrapezoidalMembershipFunction> {
    assert!(a <= b && b <= c && c <= d, "Trapezoidal membership function requires a <= b <= c <= d");
    Box::new(TrapezoidalMembershipFunction { a, b, c, d })
}

pub fn gaussian(mean: f64, sigma: f64) -> Box<GaussianMembershipFunction> {
    assert!(sigma > 0.0, "Gaussian membership function requires sigma > 0");
    Box::new(GaussianMembershipFunction { mean, sigma })
}

pub fn sigmoidal(a: f64, c: f64) -> Box<SigmoidalMembershipFunction> {
    assert!(a.abs() > f64::EPSILON, "Sigmoidal membership function requires a != 0");
    Box::new(SigmoidalMembershipFunction { a, c })
}