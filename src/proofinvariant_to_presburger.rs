use crate::presburger::{PresburgerSet, QuantifiedSet};
use crate::proof_parser::{ProofInvariant, Formula, Constraint as ProofConstraint};
use crate::kleene::Kleene;

/// Convert a single affine constraint to a PresburgerSet
/// Note: This only works when T is String since that's what the proof parser uses
pub fn from_affine_constraint(
    constraint: &ProofConstraint,
    mapping: Vec<String>
) -> PresburgerSet<String> {
    // Convert the proof constraint to a presburger constraint
    let p_constraint = crate::proof_parser::to_presburger_constraint(constraint);
    
    // Wrap in QuantifiedSet
    let qs = QuantifiedSet::new(vec![p_constraint]);
    
    // Use existing from_quantified_sets
    PresburgerSet::from_quantified_sets(&[qs], mapping)
}

/// Convert a Formula to PresburgerSet
pub fn formula_to_presburger(
    formula: &Formula,
    mapping: &[String]
) -> PresburgerSet<String> {
    match formula {
        Formula::Constraint(constraint) => {
            // Use from_affine_constraint for single constraints
            from_affine_constraint(constraint, mapping.to_vec())
        }
        
        Formula::And(formulas) => {
            // AND = intersection of all subformulas
            formulas.iter()
                .map(|f| formula_to_presburger(f, mapping))
                .reduce(|a, b| a.intersection(&b))
                .unwrap_or_else(|| PresburgerSet::universe(mapping.to_vec()))
        }
        
        Formula::Or(formulas) => {
            // OR = union of all subformulas
            formulas.iter()
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
    proof_invariant: &ProofInvariant,
    mapping: Vec<String>
) -> PresburgerSet<String> {
    formula_to_presburger(&proof_invariant.formula, &mapping)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_parser::{AffineExpr, CompOp};

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
        let constraint1 = ProofConstraint::new(
            AffineExpr::from_var("x".to_string()),
            CompOp::Geq
        );
        
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
        let constraint1 = ProofConstraint::new(
            AffineExpr::from_var("x".to_string()),
            CompOp::Eq
        );
        
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
        let x_geq_0 = ProofConstraint::new(
            AffineExpr::from_var("x".to_string()),
            CompOp::Geq
        );
        
        let y_geq_0 = ProofConstraint::new(
            AffineExpr::from_var("y".to_string()),
            CompOp::Geq
        );
        
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
    #[should_panic(expected = "Existential quantification not supported")]
    fn test_exists_formula_panics() {
        let formula = Formula::Exists(
            "x".to_string(),
            Box::new(Formula::Constraint(ProofConstraint::new(
                AffineExpr::from_var("x".to_string()),
                CompOp::Eq
            )))
        );
        
        let mapping = vec!["x".to_string()];
        let _ = formula_to_presburger(&formula, &mapping);
    }

    #[test]
    #[should_panic(expected = "Universal quantification not supported")]
    fn test_forall_formula_panics() {
        let formula = Formula::Forall(
            "x".to_string(),
            Box::new(Formula::Constraint(ProofConstraint::new(
                AffineExpr::from_var("x".to_string()),
                CompOp::Geq
            )))
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
}