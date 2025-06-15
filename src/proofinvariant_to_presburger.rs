use crate::kleene::Kleene;
use crate::presburger::{PresburgerSet, QuantifiedSet};
use crate::proof_parser::{Constraint as ProofConstraint, Formula, ProofInvariant};
use either::Either;
use std::hash::Hash;

/// Convert a single affine constraint to a PresburgerSet
/// Note: This only works when T is String since that's what the proof parser uses
pub fn from_affine_constraint(
    constraint: &ProofConstraint<String>,
    mapping: Vec<String>,
) -> PresburgerSet<String> {
    // Convert the proof constraint to a presburger constraint
    let p_constraint = crate::proof_parser::to_presburger_constraint(constraint);

    // Wrap in QuantifiedSet
    let qs = QuantifiedSet::new(vec![p_constraint]);

    // Use existing from_quantified_sets
    PresburgerSet::from_quantified_sets(&[qs], mapping)
}

/// Convert a Formula to PresburgerSet
pub fn formula_to_presburger(formula: &Formula<String>, mapping: &[String]) -> PresburgerSet<String> {
    match formula {
        Formula::Constraint(constraint) => {
            // Use from_affine_constraint for single constraints
            from_affine_constraint(constraint, mapping.to_vec())
        }

        Formula::And(formulas) => {
            // AND = intersection of all subformulas
            formulas
                .iter()
                .map(|f| formula_to_presburger(f, mapping))
                .reduce(|a, b| a.intersection(&b))
                .unwrap_or_else(|| PresburgerSet::universe(mapping.to_vec()))
        }

        Formula::Or(formulas) => {
            // OR = union of all subformulas
            formulas
                .iter()
                .map(|f| formula_to_presburger(f, mapping))
                .reduce(|a, b| a.union(&b))
                .unwrap_or_else(|| PresburgerSet::<String>::zero())
        }

        Formula::Exists(_, _) => {
            unreachable!("Existential quantification not supported in PresburgerSet conversion")
        }

        Formula::Forall(_, _) => {
            unreachable!("Universal quantification not supported in PresburgerSet conversion")
        }
    }
}

/// Convert a ProofInvariant to PresburgerSet
pub fn proof_invariant_to_presburger(
    proof_invariant: &ProofInvariant<String>,
    mapping: Vec<String>,
) -> PresburgerSet<String> {
    formula_to_presburger(&proof_invariant.formula, &mapping)
}

/// Eliminate places forward by constraining them to be zero
/// This adds the places to the variable list and ANDs the formula with (place = 0) for each place
pub fn eliminate_forward<T>(proof_invariant: &ProofInvariant<T>, places: &[T]) -> ProofInvariant<T>
where
    T: Clone + PartialEq + Eq + Hash + std::fmt::Display,
{
    use crate::proof_parser::{AffineExpr, CompOp};

    // Check that none of the places are already in the variable list
    for place in places {
        assert!(
            !proof_invariant.variables.contains(place),
            "Place {} is already in the variable list",
            place
        );
    }

    // Create new variable list with places added
    let mut new_variables = proof_invariant.variables.clone();
    new_variables.extend(places.iter().cloned());

    // Create constraints for each place = 0
    let mut place_constraints = Vec::new();
    for place in places {
        let expr = AffineExpr::from_var(place.clone());
        let constraint = ProofConstraint::new(expr, CompOp::Eq);
        place_constraints.push(Formula::Constraint(constraint));
    }

    // AND the original formula with all place = 0 constraints
    let mut all_formulas = vec![proof_invariant.formula.clone()];
    all_formulas.extend(place_constraints);

    let new_formula = Formula::And(all_formulas);

    ProofInvariant {
        variables: new_variables,
        formula: new_formula,
    }
}

/// Eliminate places backward by requiring at least one to be non-zero
/// This adds the places to the variable list and ORs the formula with (place != 0) for each place
pub fn eliminate_backward<T>(proof_invariant: &ProofInvariant<T>, places: &[T]) -> ProofInvariant<T>
where
    T: Clone + PartialEq + Eq + Hash + std::fmt::Display,
{
    use crate::proof_parser::{AffineExpr, CompOp};

    // Check that none of the places are already in the variable list
    for place in places {
        assert!(
            !proof_invariant.variables.contains(place),
            "Place {} is already in the variable list",
            place
        );
    }

    // Create new variable list with places added
    let mut new_variables = proof_invariant.variables.clone();
    new_variables.extend(places.iter().cloned());

    // Create constraints for each place != 0
    // Since we can only express >= and =, we'll use (place >= 1) for natural numbers
    let mut place_constraints = Vec::new();
    for place in places {
        let mut expr = AffineExpr::from_var(place.clone());
        expr = expr.sub(&AffineExpr::from_const(1)); // place - 1 >= 0 means place >= 1
        let constraint = ProofConstraint::new(expr, CompOp::Geq);
        place_constraints.push(Formula::Constraint(constraint));
    }

    // OR all the non-zero constraints (at least one place must be non-zero)
    let places_nonzero = Formula::Or(place_constraints);

    // OR the original formula with the places_nonzero formula
    let new_formula = Formula::Or(vec![proof_invariant.formula.clone(), places_nonzero]);

    ProofInvariant {
        variables: new_variables,
        formula: new_formula,
    }
}

/// Create a universe proof invariant (true for all values)
pub fn universe_proof<T>(variables: Vec<T>) -> ProofInvariant<T>
where
    T: Clone + Eq + Hash,
{
    ProofInvariant {
        variables,
        formula: Formula::And(vec![]), // Empty AND = true
    }
}


/// Existentially quantify over the given variables 
/// This function wraps the formula in existential quantifiers but keeps the Either type
/// to avoid type mismatches. The actual projection happens later.
pub fn existentially_quantify_keep_either<T>(
    proof: ProofInvariant<Either<usize, T>>,
    existential_vars: &[usize],
) -> ProofInvariant<Either<usize, T>>
where
    T: Clone + PartialEq + Eq + Hash + std::fmt::Display,
{
    // Separate variables into existential (Left) and regular (Right)
    let mut existential_in_proof = Vec::new();
    let mut remaining_vars = Vec::new();
    
    for var in proof.variables {
        match &var {
            Either::Left(i) => {
                if existential_vars.contains(i) {
                    existential_in_proof.push(var);
                } else {
                    // This shouldn't happen - Left variables should all be existential
                    panic!("Found Left({}) variable that's not in existential_vars list", i);
                }
            }
            Either::Right(_) => {
                remaining_vars.push(var);
            }
        }
    }
    
    // Wrap the formula with existential quantifiers for each Left(i) variable
    let mut formula = proof.formula;
    for ex_var in existential_in_proof.into_iter().rev() {
        // Extract the usize from Either::Left
        match ex_var {
            Either::Left(idx) => {
                formula = Formula::Exists(idx, Box::new(formula));
            }
            Either::Right(_) => {
                panic!("Expected Left variant for existential variable");
            }
        }
    }
    
    ProofInvariant {
        variables: remaining_vars,
        formula,
    }
}

/// Project a ProofInvariant from Either<usize, T> to T
/// This assumes all Left variables have been existentially quantified
pub fn project_proof_from_either<T>(
    proof: ProofInvariant<Either<usize, T>>,
) -> ProofInvariant<T>
where
    T: Clone + Eq + Hash,
{
    // Use the new project_right method instead of map to avoid infinite recursion
    proof.project_right()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_parser::{AffineExpr, CompOp};
    use either::{Left, Right};

    #[test]
    #[ignore] // TODO: Fix recursion issue with Either types
    fn test_existentially_quantify() {
        // Create a proof invariant with mixed Left/Right variables
        let expr1 = AffineExpr::from_var(Left(0));
        let constraint1 = ProofConstraint::new(expr1, CompOp::Eq);
        
        let expr2 = AffineExpr::from_var(Right("x".to_string()));
        let constraint2 = ProofConstraint::new(expr2, CompOp::Geq);
        
        let formula = Formula::And(vec![
            Formula::Constraint(constraint1),
            Formula::Constraint(constraint2),
        ]);
        
        let proof = ProofInvariant {
            variables: vec![Left(0), Right("x".to_string())],
            formula,
        };
        
        // First, existentially quantify over variable 0 (keeping Either type)
        let quantified = existentially_quantify_keep_either(proof, &[0]);
        
        // Check that only the Right variable remains in the variables list
        assert_eq!(quantified.variables.len(), 1);
        match &quantified.variables[0] {
            Right(v) => assert_eq!(v, "x"),
            Left(_) => panic!("Expected Right variable"),
        }
        
        // Check that the formula is wrapped in an existential quantifier
        match &quantified.formula {
            Formula::Exists(var, _body) => {
                assert_eq!(*var, 0); // Should be the existential variable index 0
            }
            _ => panic!("Expected Exists formula"),
        }
        
        // Now project to remove Either
        let final_proof = project_proof_from_either(quantified);
        assert_eq!(final_proof.variables, vec!["x".to_string()]);
    }

    #[test]
    fn test_single_equality_constraint() {
        // Test: x = 5
        let mut expr = AffineExpr::new();
        expr = expr.add(&AffineExpr::from_var("x".to_string()));
        expr = expr.sub(&AffineExpr::from_const(5));

        let constraint = ProofConstraint::new(expr, CompOp::Eq);
        let mapping = vec!["x".to_string()];

        let ps = from_affine_constraint(&constraint, mapping.clone());

        // The result should be a set containing only the point x=5
        assert!(!ps.is_empty());
        println!("Single equality constraint: {}", ps);
    }

    #[test]
    fn test_single_inequality_constraint() {
        // Test: x >= 3 (or x - 3 >= 0)
        let mut expr = AffineExpr::new();
        expr = expr.add(&AffineExpr::from_var("x".to_string()));
        expr = expr.sub(&AffineExpr::from_const(3));

        let constraint = ProofConstraint::new(expr, CompOp::Geq);
        let mapping = vec!["x".to_string()];

        let ps = from_affine_constraint(&constraint, mapping.clone());

        // The result should be a set containing all x >= 3
        assert!(!ps.is_empty());
        println!("Single inequality constraint: {}", ps);
    }

    #[test]
    fn test_multi_variable_constraint() {
        // Test: 2x + 3y - 10 = 0
        let mut expr = AffineExpr::new();
        expr = expr.add(&AffineExpr::from_var("x".to_string()).mul_by_const(2));
        expr = expr.add(&AffineExpr::from_var("y".to_string()).mul_by_const(3));
        expr = expr.sub(&AffineExpr::from_const(10));

        let constraint = ProofConstraint::new(expr, CompOp::Eq);
        let mapping = vec!["x".to_string(), "y".to_string()];

        let ps = from_affine_constraint(&constraint, mapping.clone());

        assert!(!ps.is_empty());
        println!("Multi-variable constraint: {}", ps);
    }

    #[test]
    fn test_and_formula() {
        // Test: x >= 0 AND x <= 10 (represented as x >= 0 AND -x + 10 >= 0)
        let constraint1 = ProofConstraint::new(AffineExpr::from_var("x".to_string()), CompOp::Geq);

        let mut expr2 = AffineExpr::new();
        expr2 = expr2.add(&AffineExpr::from_const(10));
        expr2 = expr2.sub(&AffineExpr::from_var("x".to_string()));
        let constraint2 = ProofConstraint::new(expr2, CompOp::Geq);

        let formula = Formula::And(vec![
            Formula::Constraint(constraint1),
            Formula::Constraint(constraint2),
        ]);

        let mapping = vec!["x".to_string()];
        let ps = formula_to_presburger(&formula, &mapping);

        // The result should be the interval [0, 10]
        assert!(!ps.is_empty());
        println!("AND formula (0 <= x <= 10): {}", ps);
    }

    #[test]
    fn test_or_formula() {
        // Test: x = 0 OR x = 5
        let constraint1 = ProofConstraint::new(AffineExpr::from_var("x".to_string()), CompOp::Eq);

        let mut expr2 = AffineExpr::new();
        expr2 = expr2.add(&AffineExpr::from_var("x".to_string()));
        expr2 = expr2.sub(&AffineExpr::from_const(5));
        let constraint2 = ProofConstraint::new(expr2, CompOp::Eq);

        let formula = Formula::Or(vec![
            Formula::Constraint(constraint1),
            Formula::Constraint(constraint2),
        ]);

        let mapping = vec!["x".to_string()];
        let ps = formula_to_presburger(&formula, &mapping);

        // The result should contain exactly two points: x=0 and x=5
        assert!(!ps.is_empty());
        println!("OR formula (x=0 OR x=5): {}", ps);
    }

    #[test]
    fn test_complex_formula() {
        // Test: (x >= 0 AND y >= 0) OR (x = 10 AND y = 20)
        let x_geq_0 = ProofConstraint::new(AffineExpr::from_var("x".to_string()), CompOp::Geq);

        let y_geq_0 = ProofConstraint::new(AffineExpr::from_var("y".to_string()), CompOp::Geq);

        let mut x_eq_10_expr = AffineExpr::new();
        x_eq_10_expr = x_eq_10_expr.add(&AffineExpr::from_var("x".to_string()));
        x_eq_10_expr = x_eq_10_expr.sub(&AffineExpr::from_const(10));
        let x_eq_10 = ProofConstraint::new(x_eq_10_expr, CompOp::Eq);

        let mut y_eq_20_expr = AffineExpr::new();
        y_eq_20_expr = y_eq_20_expr.add(&AffineExpr::from_var("y".to_string()));
        y_eq_20_expr = y_eq_20_expr.sub(&AffineExpr::from_const(20));
        let y_eq_20 = ProofConstraint::new(y_eq_20_expr, CompOp::Eq);

        let formula = Formula::Or(vec![
            Formula::And(vec![
                Formula::Constraint(x_geq_0),
                Formula::Constraint(y_geq_0),
            ]),
            Formula::And(vec![
                Formula::Constraint(x_eq_10),
                Formula::Constraint(y_eq_20),
            ]),
        ]);

        let mapping = vec!["x".to_string(), "y".to_string()];
        let ps = formula_to_presburger(&formula, &mapping);

        assert!(!ps.is_empty());
        println!("Complex formula: {}", ps);
    }

    #[test]
    fn test_empty_and() {
        // Empty AND should return universe
        let formula = Formula::And(vec![]);
        let mapping = vec!["x".to_string(), "y".to_string()];
        let ps = formula_to_presburger(&formula, &mapping);

        // Should be the universe set
        assert!(!ps.is_empty());
        println!("Empty AND (universe): {}", ps);
    }

    #[test]
    fn test_empty_or() {
        // Empty OR should return empty set
        let formula = Formula::Or(vec![]);
        let mapping = vec!["x".to_string()];
        let ps = formula_to_presburger(&formula, &mapping);

        // Should be the empty set
        assert!(ps.is_empty());
        println!("Empty OR (empty set): {}", ps);
    }

    #[test]
    fn test_proof_invariant() {
        // Test converting a full ProofInvariant
        let mut expr = AffineExpr::new();
        expr = expr.add(&AffineExpr::from_var("p0".to_string()));
        expr = expr.add(&AffineExpr::from_var("p1".to_string()));
        expr = expr.sub(&AffineExpr::from_const(100));

        let constraint = ProofConstraint::new(expr, CompOp::Geq);
        let formula = Formula::Constraint(constraint);

        let proof_inv = ProofInvariant {
            variables: vec!["p0".to_string(), "p1".to_string()],
            formula,
        };

        let ps = proof_invariant_to_presburger(&proof_inv, proof_inv.variables.clone());

        assert!(!ps.is_empty());
        println!("ProofInvariant (p0 + p1 >= 100): {}", ps);
    }

    #[test]
    #[should_panic(expected = "Existential quantification not supported in PresburgerSet conversion")]
    fn test_exists_formula() {
        // Create a formula with an existential variable
        let formula = Formula::Exists(
            0, // Using index 0 for the existential variable
            Box::new(Formula::Constraint(ProofConstraint::new(
                AffineExpr::from_var("x".to_string()),
                CompOp::Eq,
            ))),
        );

        let mapping = vec!["x".to_string()];
        // This should panic as existential quantification is not supported in formula_to_presburger
        let _ = formula_to_presburger(&formula, &mapping);
    }

    #[test]
    #[should_panic(expected = "Universal quantification not supported in PresburgerSet conversion")]
    fn test_forall_formula_panics() {
        let formula = Formula::Forall(
            0, // Using index 0 for the universal variable
            Box::new(Formula::Constraint(ProofConstraint::new(
                AffineExpr::from_var("x".to_string()),
                CompOp::Geq,
            ))),
        );

        let mapping = vec!["x".to_string()];
        let _ = formula_to_presburger(&formula, &mapping);
    }

    #[test]
    fn test_formula_with_different_variable_order() {
        // Test that variable ordering in mapping matters
        let mut expr = AffineExpr::new();
        expr = expr.add(&AffineExpr::from_var("y".to_string()));
        expr = expr.sub(&AffineExpr::from_var("x".to_string()));

        let constraint = ProofConstraint::new(expr, CompOp::Eq);
        let formula = Formula::Constraint(constraint);

        // Test with different variable orderings
        let mapping1 = vec!["x".to_string(), "y".to_string()];
        let mapping2 = vec!["y".to_string(), "x".to_string()];

        let ps1 = formula_to_presburger(&formula, &mapping1);
        let ps2 = formula_to_presburger(&formula, &mapping2);

        println!("Formula with mapping [x,y]: {}", ps1);
        println!("Formula with mapping [y,x]: {}", ps2);

        assert!(!ps1.is_empty());
        assert!(!ps2.is_empty());
    }

    #[test]
    fn test_eliminate_forward() {
        // Test with simple formula x >= 5
        let mut expr = AffineExpr::new();
        expr = expr.add(&AffineExpr::from_var("x".to_string()));
        expr = expr.sub(&AffineExpr::from_const(5));
        let constraint = ProofConstraint::new(expr, CompOp::Geq);
        let formula = Formula::Constraint(constraint);

        let proof_inv = ProofInvariant {
            variables: vec!["x".to_string()],
            formula,
        };

        let places = vec!["p1".to_string(), "p2".to_string()];
        let result = eliminate_forward(&proof_inv, &places);

        // Check variables were added
        assert_eq!(
            result.variables,
            vec!["x".to_string(), "p1".to_string(), "p2".to_string()]
        );

        // Check formula is AND
        match &result.formula {
            Formula::And(formulas) => {
                assert_eq!(formulas.len(), 3); // original + 2 places

                // Convert to PresburgerSet to verify the result
                let ps = formula_to_presburger(&result.formula, &result.variables);
                println!("eliminate_forward result: {}", ps);
            }
            _ => panic!("Expected AND formula"),
        }
    }

    #[test]
    fn test_eliminate_backward() {
        // Test with simple formula x >= 5
        let mut expr = AffineExpr::new();
        expr = expr.add(&AffineExpr::from_var("x".to_string()));
        expr = expr.sub(&AffineExpr::from_const(5));
        let constraint = ProofConstraint::new(expr, CompOp::Geq);
        let formula = Formula::Constraint(constraint);

        let proof_inv = ProofInvariant {
            variables: vec!["x".to_string()],
            formula,
        };

        let places = vec!["p1".to_string(), "p2".to_string()];
        let result = eliminate_backward(&proof_inv, &places);

        // Check variables were added
        assert_eq!(
            result.variables,
            vec!["x".to_string(), "p1".to_string(), "p2".to_string()]
        );

        // Check formula is OR
        match &result.formula {
            Formula::Or(formulas) => {
                assert_eq!(formulas.len(), 2); // original formula + places_nonzero

                // Convert to PresburgerSet to verify the result
                let ps = formula_to_presburger(&result.formula, &result.variables);
                println!("eliminate_backward result: {}", ps);
            }
            _ => panic!("Expected OR formula"),
        }
    }

    #[test]
    #[should_panic(expected = "Place x is already in the variable list")]
    fn test_eliminate_forward_duplicate_variable() {
        // Test assertion when place is already a variable
        let constraint = ProofConstraint::new(AffineExpr::from_var("x".to_string()), CompOp::Eq);
        let formula = Formula::Constraint(constraint);

        let proof_inv = ProofInvariant {
            variables: vec!["x".to_string(), "y".to_string()],
            formula,
        };

        // Try to add 'x' as a place, which should panic
        let places = vec!["x".to_string()];
        let _ = eliminate_forward(&proof_inv, &places);
    }

    #[test]
    #[should_panic(expected = "Place y is already in the variable list")]
    fn test_eliminate_backward_duplicate_variable() {
        // Test assertion when place is already a variable
        let constraint = ProofConstraint::new(AffineExpr::from_var("x".to_string()), CompOp::Geq);
        let formula = Formula::Constraint(constraint);

        let proof_inv = ProofInvariant {
            variables: vec!["x".to_string(), "y".to_string()],
            formula,
        };

        // Try to add 'y' as a place, which should panic
        let places = vec!["y".to_string(), "z".to_string()];
        let _ = eliminate_backward(&proof_inv, &places);
    }

    #[test]
    fn test_eliminate_forward_empty_places() {
        // Test with empty places list
        let constraint = ProofConstraint::new(
            AffineExpr::from_var("x".to_string()).sub(&AffineExpr::from_const(10)),
            CompOp::Eq,
        );
        let formula = Formula::Constraint(constraint);

        let proof_inv = ProofInvariant {
            variables: vec!["x".to_string()],
            formula: formula.clone(),
        };

        let result = eliminate_forward(&proof_inv, &[]);

        // Variables should be unchanged
        assert_eq!(result.variables, proof_inv.variables);

        // Formula should be AND with single element
        match &result.formula {
            Formula::And(formulas) => {
                assert_eq!(formulas.len(), 1);
                assert_eq!(&formulas[0], &formula);
            }
            _ => panic!("Expected AND formula"),
        }
    }

    #[test]
    fn test_eliminate_backward_empty_places() {
        // Test with empty places list
        let constraint = ProofConstraint::new(AffineExpr::from_var("x".to_string()), CompOp::Geq);
        let formula = Formula::Constraint(constraint);

        let proof_inv = ProofInvariant {
            variables: vec!["x".to_string()],
            formula: formula.clone(),
        };

        let result = eliminate_backward(&proof_inv, &[]);

        // Should still create an OR with the original formula
        assert_eq!(result.variables, proof_inv.variables);
        match &result.formula {
            Formula::Or(formulas) => {
                assert_eq!(formulas.len(), 2);
                assert_eq!(&formulas[0], &formula);
                // Second should be an empty Or (no places)
                match &formulas[1] {
                    Formula::Or(inner) => assert_eq!(inner.len(), 0),
                    _ => panic!("Expected empty OR for places"),
                }
            }
            _ => panic!("Expected OR formula"),
        }
    }

    #[test]
    fn test_true_false_formulas() {
        // Test that true (empty AND) converts to universe
        let true_formula = Formula::And(vec![]);
        let mapping = vec!["x".to_string(), "y".to_string()];
        let ps_true = formula_to_presburger(&true_formula, &mapping);

        println!("True formula as PresburgerSet: {}", ps_true);
        assert!(!ps_true.is_empty());

        // Compare with explicit universe
        let universe = PresburgerSet::universe(mapping.clone());
        assert_eq!(ps_true, universe);

        // Test that false (empty OR) converts to empty set
        let false_formula = Formula::Or(vec![]);
        let ps_false = formula_to_presburger(&false_formula, &mapping);

        println!("False formula as PresburgerSet: {}", ps_false);
        assert!(ps_false.is_empty());

        // Compare with explicit empty set
        let empty = PresburgerSet::<String>::zero();
        assert_eq!(ps_false, empty);
    }
}
