//! Affine constraints, like might be output from ISL
//!
//! Variables are normalized to be v0, v1, v2, ...

use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Var(usize);

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

pub struct Constraint {
    /// Linear combination of variables: (coeff, var) pairs
    affine_formula: Vec<(i32, Var)>,
    offset: i32,
    /// What kind of constraint (inequality or equality)
    constraint_type: ConstraintType,
}

/// Variables 0...N-1 are the real variables.
/// Variables N...N+k-1 are the newly introduced existential variables
pub struct Constraints {
    num_vars: usize,             // N
    num_existential_vars: usize, // k

    /// A big OR over a bunch of big ANDs of constraints
    constraints: Vec<Vec<Constraint>>,
}
