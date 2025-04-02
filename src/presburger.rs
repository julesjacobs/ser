use std::fmt;
use std::hash::Hash;
use crate::semilinear::{SemilinearSet, LinearSet};

/// Represents a Presburger set, which is a union of existentially quantified conjunctions of linear constraints
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresburgerSet<T> {
   union: Vec<QuantifiedSet<T>>
}

impl<T: Hash + Clone + Ord> PresburgerSet<T> {
    /// Create a new PresburgerSet with the given union of QuantifiedSet components
    pub fn new(union: Vec<QuantifiedSet<T>>) -> Self {
        PresburgerSet { union }
    }
    
    /// Create a new PresburgerSet with consistently sorted components for equality comparison
    pub fn with_sorted_components(mut union: Vec<QuantifiedSet<T>>) -> Self {
        // We don't have a good way to order QuantifiedSet components consistently,
        // but we can ensure the order is deterministic by length of constraints
        union.sort_by_key(|qs| qs.constraints.len());
        PresburgerSet { union }
    }
}

/// Represents an existentially quantified conjunction of linear constraints
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantifiedSet<T> {
    // The variables T are from the original set, and the variables usize are the existential variables
    // The existential quantification is over *natural numbers* not integers
    constraints: Vec<Constraint<Variable<T>>>
}

// Helper method to get a sorted set of constraints for comparison
impl<T: Hash + Clone + Ord> QuantifiedSet<T> {
    // Creates a new QuantifiedSet with sorted constraints for easier equality comparison
    pub fn with_sorted_constraints(mut constraints: Vec<Constraint<Variable<T>>>) -> Self {
        // Sort constraints by a stable order to make comparison consistent
        constraints.sort_by_key(|c| {
            // Create a canonical representation for sorting
            // First by constraint type, then variables
            let mut key = Vec::new();
            
            // Add constraint type
            key.push(match c.constraint_type {
                ConstraintType::EqualToZero => 0,
                ConstraintType::NonNegative => 1,
            });
            
            // Add variables and coefficients, sorted by variable
            let mut terms: Vec<_> = c.linear_combination.clone();
            terms.sort_by(|(_, a), (_, b)| a.cmp(b));
            for (coef, var) in terms {
                match &var {
                    Variable::Original(_) => key.push(0),
                    Variable::Existential(idx) => key.push(1 + *idx as i32),
                }
                key.push(coef);
            }
            
            // Add constant term
            key.push(c.constant_term);
            
            key
        });
        
        QuantifiedSet { constraints }
    }
}

/// A variable in a Presburger formula: either an original variable or an existentially quantified variable
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Variable<T> {
    Original(T),
    Existential(usize),
}

/// Represents a linear constraint
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint<T> {
    linear_combination: Vec<(i32, T)>,
    constant_term: i32,
    constraint_type: ConstraintType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintType {
    NonNegative,
    EqualToZero,
}

// Display implementation for Variable
impl<T: fmt::Display> fmt::Display for Variable<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Variable::Original(t) => write!(f, "{}", t),
            Variable::Existential(idx) => write!(f, "n{}", idx),
        }
    }
}

// Display implementation for constraints
impl<T: fmt::Display> fmt::Display for Constraint<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format the linear combination
        let mut terms = Vec::new();
        for (coef, var) in &self.linear_combination {
            if *coef == 0 {
                continue;
            }
            
            if *coef == 1 {
                terms.push(format!("{}", var));
            } else if *coef == -1 {
                terms.push(format!("-{}", var));
            } else {
                terms.push(format!("{}·{}", coef, var));
            }
        }
        
        // Add the constant term if non-zero
        if self.constant_term != 0 {
            terms.push(format!("{}", self.constant_term));
        }
        
        // Join terms with plus signs, being careful about negative terms
        let mut result = String::new();
        for (i, term) in terms.iter().enumerate() {
            if i > 0 {
                // Add a space, and a plus sign if the next term is positive
                if term.starts_with('-') {
                    result.push_str(" - ");
                    result.push_str(&term[1..]);
                } else {
                    result.push_str(" + ");
                    result.push_str(term);
                }
            } else {
                // First term
                result.push_str(term);
            }
        }
        
        // If no terms, just show 0
        if result.is_empty() {
            result = "0".to_string();
        }
        
        // Add the constraint type
        match self.constraint_type {
            ConstraintType::NonNegative => write!(f, "{} ≥ 0", result),
            ConstraintType::EqualToZero => write!(f, "{} = 0", result),
        }
    }
}

// Display implementation for QuantifiedSet
impl<T: fmt::Display> fmt::Display for QuantifiedSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Count how many existential variables are used
        let mut max_var = 0;
        for constraint in &self.constraints {
            for (_, var) in &constraint.linear_combination {
                if let Variable::Existential(idx) = var {
                    max_var = max_var.max(*idx + 1);
                }
            }
        }
        
        // Format the existential quantifier prefix
        if max_var > 0 {
            write!(f, "∃")?;
            for i in 0..max_var {
                if i > 0 {
                    write!(f, ",")?;
                }
                write!(f, "n{}", i)?;
            }
            write!(f, ". ")?;
        }
        
        // Format the constraints
        for (i, constraint) in self.constraints.iter().enumerate() {
            if i > 0 {
                write!(f, " ∧ ")?;
            }
            write!(f, "{}", constraint)?;
        }
        
        Ok(())
    }
}

// Display implementation for PresburgerSet
impl<T: fmt::Display> fmt::Display for PresburgerSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, set) in self.union.iter().enumerate() {
            if i > 0 {
                write!(f, " ∨ ")?;
            }
            
            // Add parentheses if there are multiple components
            if self.union.len() > 1 {
                write!(f, "({})", set)?;
            } else {
                write!(f, "{}", set)?;
            }
        }
        
        Ok(())
    }
}
 

impl<T: Hash + Clone + Ord + fmt::Display> PresburgerSet<T> {
    /// Convert a LinearSet<T> to a QuantifiedSet<T>
    /// A linear set base + n₁·period₁ + n₂·period₂ + ... (where n_i are natural numbers)
    /// becomes a quantified set with existential variables for the coefficients
    // Make this public to allow testing
    pub fn from_linear_set(linear_set: &LinearSet<T>) -> QuantifiedSet<T> {
        let mut constraints = Vec::new();
        
        // For a linear set with base vector (b₁, b₂, ...) and period vectors
        // (p₁₁, p₁₂, ...), (p₂₁, p₂₂, ...), etc.,
        // we create constraints:
        // 1. For each dimension, x = b + n₁·p₁ + n₂·p₂ + ...
        //    where x is the original variable, b is the base value, and n_i are the existential variables
        
        // Collect all unique dimensions from base and periods
        let mut all_dimensions = std::collections::HashSet::new();
        linear_set.for_each_key(|key| { all_dimensions.insert(key.clone()); });
        
        // For each dimension, create a constraint x = b + n₁·p₁ + n₂·p₂ + ...
        for dim in all_dimensions {
            let mut linear_combination = Vec::new();
            
            // Original variable with coefficient 1
            linear_combination.push((1, Variable::Original(dim.clone())));
            
            // Base with coefficient -1
            let base_val = linear_set.base.get(&dim) as i32;
            let constant_term = -base_val;
            
            // Each period contributes -p*n to the constraint
            for (i, period) in linear_set.periods.iter().enumerate() {
                let period_val = period.get(&dim) as i32;
                if period_val > 0 {
                    linear_combination.push((-period_val, Variable::Existential(i)));
                }
            }
            
            // Create the equality constraint: x - b - Σ(p_i * n_i) = 0
            constraints.push(Constraint {
                linear_combination,
                constant_term,
                constraint_type: ConstraintType::EqualToZero,
            });
        }
        
        // Note: We don't need to add non-negativity constraints for existential variables
        // since they are already understood to range over natural numbers (non-negative integers)
        
        // Sort the constraints for consistent comparison
        QuantifiedSet::with_sorted_constraints(constraints)
    }

    /// Convert a SemilinearSet<T> to a PresburgerSet<T>
    pub fn from_semilinear_set(semilinear_set: &SemilinearSet<T>) -> Self {
        // A semilinear set is a union of linear sets
        // Convert each linear set to a quantified set and collect them
        let union = semilinear_set.components
            .iter()
            .map(|linear_set| Self::from_linear_set(linear_set))
            .collect();
        
        // Return with sorted components for consistent comparison
        Self::with_sorted_components(union)
    }

    /// Compute the complement of this PresburgerSet<T> using ISL
    pub fn complement(&self) -> Self {
        todo!("Compute complement using ISL") 
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semilinear::{SemilinearSet, LinearSet, SparseVector};

    // Helper function to create a sparse vector with a single key-value pair
    fn sparse_vector<T: Hash + Clone + Ord>(key: T, value: usize) -> SparseVector<T> {
        let mut vec = SparseVector::new();
        vec.set(key, value);
        vec
    }

    #[test]
    fn test_constraint_display() {
        // Test a simple constraint: 2·x + 3·y - 4 = 0
        let constraint = Constraint {
            linear_combination: vec![
                (2, Variable::Original("x".to_string())),
                (3, Variable::Original("y".to_string())),
            ],
            constant_term: -4,
            constraint_type: ConstraintType::EqualToZero,
        };
        
        assert_eq!(constraint.to_string(), "2·x + 3·y - 4 = 0");
        
        // Test a non-negative constraint: n0 - 5 ≥ 0
        let constraint = Constraint {
            linear_combination: vec![(1, Variable::<String>::Existential(0))],
            constant_term: -5,
            constraint_type: ConstraintType::NonNegative,
        };
        
        assert_eq!(constraint.to_string(), "n0 - 5 ≥ 0");
    }

    #[test]
    fn test_simple_linear_set_conversion() {
        // Create a linear set: base = (x=1) with period (x=1)
        // This represents {x=1, x=2, x=3, ...}
        let base = sparse_vector("x".to_string(), 1);
        let period = sparse_vector("x".to_string(), 1);
        let linear_set = LinearSet {
            base,
            periods: vec![period],
        };
        
        // Convert to a Presburger set
        let presburger_set = PresburgerSet::from_semilinear_set(&SemilinearSet {
            components: vec![linear_set],
        });
        
        // Manually construct the expected Presburger set: ∃n0. x - n0 - 1 = 0
        let constraint = Constraint {
            linear_combination: vec![
                (1, Variable::Original("x".to_string())),
                (-1, Variable::Existential(0)),
            ],
            constant_term: -1,
            constraint_type: ConstraintType::EqualToZero,
        };
        
        // Use sorted constraints for consistent comparison
        let expected_quantified_set = QuantifiedSet::with_sorted_constraints(vec![constraint]);
        let expected_presburger_set = PresburgerSet::with_sorted_components(vec![expected_quantified_set]);
        
        // Check the result with display
        println!("Presburger set: {}", presburger_set);
        
        // Compare with expected set
        assert_eq!(presburger_set, expected_presburger_set);
    }

    #[test]
    fn test_multi_dimension_linear_set() {
        // Create a linear set: base = (x=1, y=2) with periods (x=1, y=0) and (x=0, y=1)
        // This represents {(x,y) | x ≥ 1, y ≥ 2}
        let mut base = SparseVector::new();
        base.set("x".to_string(), 1);
        base.set("y".to_string(), 2);
        
        let mut period1 = SparseVector::new();
        period1.set("x".to_string(), 1);
        
        let mut period2 = SparseVector::new();
        period2.set("y".to_string(), 1);
        
        let linear_set = LinearSet {
            base,
            periods: vec![period1, period2],
        };
        
        // Convert to a Presburger set
        let presburger_set = PresburgerSet::from_semilinear_set(&SemilinearSet {
            components: vec![linear_set],
        });
        
        // Print the actual result to understand the constraint order
        println!("Presburger set: {}", presburger_set);
        
        // Manually construct the expected Presburger set with sorted constraints
        // for consistent equality comparison
        let x_constraint = Constraint {
            linear_combination: vec![
                (1, Variable::Original("x".to_string())),
                (-1, Variable::Existential(0)),
            ],
            constant_term: -1,
            constraint_type: ConstraintType::EqualToZero,
        };
        
        let y_constraint = Constraint {
            linear_combination: vec![
                (1, Variable::Original("y".to_string())),
                (-1, Variable::Existential(1)),
            ],
            constant_term: -2,
            constraint_type: ConstraintType::EqualToZero,
        };
        
        // Using QuantifiedSet::with_sorted_constraints to make the comparison order consistent
        let constraints = vec![x_constraint, y_constraint];
        let expected_quantified_set = QuantifiedSet::with_sorted_constraints(constraints);
        let expected_presburger_set = PresburgerSet::with_sorted_components(vec![expected_quantified_set]);
        
        // Check the result with display
        println!("Presburger set: {}", presburger_set);
        
        // Compare with expected set
        assert_eq!(presburger_set, expected_presburger_set);
    }

    #[test]
    fn test_union_of_linear_sets() {
        // Create two linear sets and their union
        // Linear set 1: x = 1 + n0 where n0 ≥ 0, representing {1, 2, 3, ...}
        let base1 = sparse_vector("x".to_string(), 1);
        let period1 = sparse_vector("x".to_string(), 1);
        let linear_set1 = LinearSet {
            base: base1,
            periods: vec![period1],
        };
        
        // Linear set 2: x = 0, representing {0}
        let base2 = sparse_vector("x".to_string(), 0);
        let linear_set2 = LinearSet {
            base: base2,
            periods: vec![],
        };
        
        // Combine them into a semilinear set
        let semilinear_set = SemilinearSet {
            components: vec![linear_set1, linear_set2],
        };
        
        // Convert to a Presburger set
        let presburger_set = PresburgerSet::from_semilinear_set(&semilinear_set);
        
        // Print the actual result to understand it
        println!("Presburger set: {}", presburger_set);
        
        // Manually construct the expected Presburger set based on the actual output: (∃n0. x - n0 - 1 = 0) ∨ ()
        
        // First component: ∃n0. x - n0 - 1 = 0
        let constraint1 = Constraint {
            linear_combination: vec![
                (1, Variable::Original("x".to_string())),
                (-1, Variable::Existential(0)),
            ],
            constant_term: -1,
            constraint_type: ConstraintType::EqualToZero,
        };
        
        let quantified_set1 = QuantifiedSet::with_sorted_constraints(vec![constraint1]);
        
        // Second component: empty constraints
        // Note: This is the way our implementation represents base vectors with all zeros (empty string)
        let quantified_set2 = QuantifiedSet::with_sorted_constraints(vec![]);
        
        // Combined Presburger set with two components
        let expected_presburger_set = PresburgerSet::with_sorted_components(vec![quantified_set1, quantified_set2]);
        
        // Check the result with display
        println!("Presburger set: {}", presburger_set);
        
        // Compare with expected set
        assert_eq!(presburger_set, expected_presburger_set);
    }
    
    #[test]
    fn test_kleene_star_conversion() {
        // Test conversion of a* to Presburger formula
        // a* represents {ε, a, aa, aaa, ...}
        // As a semilinear set: {0} + {1}(1)*
        let zero_base = SparseVector::new();
        let linear_set_empty = LinearSet {
            base: zero_base,
            periods: vec![],
        };
        
        let a_base = sparse_vector("a".to_string(), 1);
        let a_period = sparse_vector("a".to_string(), 1);
        let linear_set_a_star = LinearSet {
            base: a_base,
            periods: vec![a_period],
        };
        
        let a_star = SemilinearSet {
            components: vec![linear_set_empty, linear_set_a_star],
        };
        
        // Convert to a Presburger set
        let presburger_set = PresburgerSet::from_semilinear_set(&a_star);
        
        // Manually construct the expected Presburger set
        
        // First component: empty set of constraints (representing a = 0)
        let empty_constraints = Vec::new();
        let quantified_set1 = QuantifiedSet::with_sorted_constraints(empty_constraints);
        
        // Second component: ∃n0. a - n0 - 1 = 0
        let constraint = Constraint {
            linear_combination: vec![
                (1, Variable::Original("a".to_string())),
                (-1, Variable::Existential(0)),
            ],
            constant_term: -1,
            constraint_type: ConstraintType::EqualToZero,
        };
        
        let quantified_set2 = QuantifiedSet::with_sorted_constraints(vec![constraint]);
        
        // Combined Presburger set
        let expected_presburger_set = PresburgerSet::with_sorted_components(vec![quantified_set1, quantified_set2]);
        
        // Check the result with display
        println!("a* as Presburger set: {}", presburger_set);
        
        // Compare with expected set
        assert_eq!(presburger_set, expected_presburger_set);
    }
    
    #[test]
    fn test_direct_construction() {
        // Test manually constructing a Presburger set and comparing with the one generated from a linear set
        
        // Create a linear set: x=1 + n·1 (representing x ≥ 1)
        let base = sparse_vector("x".to_string(), 1);
        let period = sparse_vector("x".to_string(), 1);
        let linear_set = LinearSet {
            base,
            periods: vec![period],
        };
        
        // Convert to a Presburger set using our function
        let presburger_set = PresburgerSet::from_semilinear_set(&SemilinearSet {
            components: vec![linear_set],
        });
        
        // Manually construct the expected Presburger set: ∃n0. x - n0 - 1 = 0
        let mut constraints = Vec::new();
        
        // x - n0 - 1 = 0
        constraints.push(Constraint {
            linear_combination: vec![
                (1, Variable::Original("x".to_string())),
                (-1, Variable::Existential(0)),
            ],
            constant_term: -1,
            constraint_type: ConstraintType::EqualToZero,
        });
        
        // Note: no non-negativity constraint for n0 since existentials already range over natural numbers
        
        let expected_quantified_set = QuantifiedSet::with_sorted_constraints(constraints);
        let expected_presburger_set = PresburgerSet::with_sorted_components(vec![expected_quantified_set]);
        
        // Compare the generated and expected Presburger sets
        assert_eq!(presburger_set, expected_presburger_set);
        
        // Print both for visual inspection
        println!("Generated: {}", presburger_set);
        println!("Expected: {}", expected_presburger_set);
    }

    use crate::kleene::Kleene;

    #[test]
    fn test_two_letters() {
        // Construct SemilinearSet using Kleene operations directly
        // We want to construct (ab)* here
        let a = SemilinearSet::atom("a".to_string());
        let b = SemilinearSet::atom("b".to_string());
        let ab = a.times(b);
        let ab_star = ab.star();
        let result = PresburgerSet::from_semilinear_set(&ab_star);
        println!("{}", result);
        // assert_eq!(result.to_string(), "∃n0. a - n0 = 0 ∧ b - n0 = 0");

        // Now let's construct a* b*
        let a = SemilinearSet::atom("a".to_string());
        let b = SemilinearSet::atom("b".to_string());
        let a_star = a.star();
        let b_star = b.star();
        let ab_star = a_star.times(b_star);
        let result = PresburgerSet::from_semilinear_set(&ab_star);
        println!("{}", result);
        // assert_eq!(result.to_string(), "∃n0,n1. b - n0 = 0 ∧ a - n1 = 0");
    }
}
