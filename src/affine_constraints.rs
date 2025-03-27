//! Affine constraints, like might be output from ISL
//!
//! Variables are normalized to be v0, v1, v2, ...

use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Var(pub usize);

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "v{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ConstraintType {
    NonNegative,
    EqualToZero,
}
pub use ConstraintType::*;

#[derive(Debug, Clone)]
pub struct Constraint {
    /// Linear combination of variables: (coeff, var) pairs
    pub affine_formula: Vec<(i32, Var)>,
    pub offset: i32,
    /// What kind of constraint (inequality or equality)
    pub constraint_type: ConstraintType,
}

/// Variables 0...N-1 are the real variables.
/// Variables N...N+k-1 are the newly introduced existential variables
#[derive(Debug, Clone)]
pub struct Constraints {
    pub num_vars: usize,             // N
    pub num_existential_vars: usize, // k

    /// A big OR over a bunch of big ANDs of constraints
    pub constraints: Vec<Vec<Constraint>>,
}
