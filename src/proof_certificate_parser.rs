use sexp::{parse, Atom, Sexp}; // ensure the `sexp` crate is included in Cargo.toml: sexp = "0.5"
use crate::presburger::{PresburgerSet, QuantifiedSet, Constraint};


/// Expression tree representing logical formula structure
#[derive(Debug, Clone)]
pub enum ExprTree {
    And(Vec<ExprTree>),
    Or(Vec<ExprTree>),
    Not(Box<ExprTree>),
    Implies(Box<ExprTree>, Box<ExprTree>),
    Leaf(Sexp), // an atomic constraint like (= ...) or (>= ...)
}

/// Parses and processes an SMT-LIB-style formula string, extracting variables and building an expression tree.
/// assume the input starts with exists (...)
pub fn process_formula(input: &str) {
    let parsed = parse(input).expect("Failed to parse S-expression");
    let list = match parsed {
        Sexp::List(l) => l,
        _ => panic!("Expected list at top level"),
    };
    assert_eq!(list[0], Sexp::Atom(Atom::S("exists".into())));

    let decls = &list[1];
    let body = &list[2];

    let ti_vars = extract_vars(decls);
    println!("Variables: {:?}", ti_vars);

    let expr_tree = parse_expr_tree(body);
    println!("Parsed expression tree: {:#?}", expr_tree);

    println!("TODO: add forward/backward analysis");
    println!("TODO: intersect between sets of different disjuncts");
}

/// Extracts variable names from a list of SMT-style declarations.
fn extract_vars(decls: &Sexp) -> Vec<String> {
    match decls {
        Sexp::List(decl_list) => decl_list.iter().map(|d| {
            let v = match d {
                Sexp::List(inner) => inner,
                _ => panic!("Expected (var Int) pair"),
            };
            assert_eq!(v[1], Sexp::Atom(Atom::S("Int".into())));
            match &v[0] {
                Sexp::Atom(Atom::S(s)) => s.clone(),
                _ => panic!("Expected symbol for variable name"),
            }
        }).collect(),
        _ => panic!("Expected list of declarations"),
    }
}

/// Attempts to extract an integer constant from a Sexp atom.
fn extract_constant(sexp: &Sexp) -> Option<i64> {
    match sexp {
        Sexp::Atom(Atom::S(s)) => {
            println!("booya");
            s.parse::<i64>().ok()
        }
        Sexp::Atom(Atom::I(i)) => Some(*i),
        _ => None,
    }
}

/// Evaluates a Sexp representing (+ c1 c2 ...) where all arguments are constants.
fn try_eval_const_sum(expr: &Sexp) -> Option<Sexp> {
    if let Sexp::List(list) = expr {
        if let Some(Sexp::Atom(Atom::S(op))) = list.get(0) {
            if op == "+" {
                let mut total = 0i64;
                for item in &list[1..] {
                    if let Some(val) = extract_constant(item) {
                        total += val;
                    } else {
                        return None;
                    }
                }
                return Some(Sexp::Atom(Atom::I(total)));
            }
        }
    }
    None
}

/// Rewrites expressions of the form (> lhs rhs) to:
/// - (>= lhs c+1) if rhs is a constant c
/// - (>= lhs (+ rhs 1)) otherwise
fn normalize_gt_to_ge(expr: &Sexp) -> Sexp {
    if let Sexp::List(list) = expr {
        if list.len() == 3 {
            if let Sexp::Atom(Atom::S(op)) = &list[0] {
                if op == ">" {
                    let lhs = &list[1];
                    let rhs = &list[2];

                    if let Some(c) = extract_constant(rhs) {
                        return Sexp::List(vec![
                            Sexp::Atom(Atom::S(">=".into())),
                            lhs.clone(),
                            Sexp::Atom(Atom::I(c + 1)),
                        ]);
                    }

                    let plus_expr = Sexp::List(vec![
                        Sexp::Atom(Atom::S("+".into())),
                        rhs.clone(),
                        Sexp::Atom(Atom::I(1)),
                    ]);

                    return Sexp::List(vec![
                        Sexp::Atom(Atom::S(">=".into())),
                        lhs.clone(),
                        simplify_sum_expr(&plus_expr),
                    ]);
                }
            }
        }
    }

    expr.clone()
}

/// Simplifies addition expressions by evaluating and folding constant terms.
/// For example, (+ x 3 4) becomes (+ x 7), and (+ 2 3) becomes 5.
fn simplify_sum_expr(expr: &Sexp) -> Sexp {
    match expr {
        Sexp::Atom(_) => expr.clone(),
        Sexp::List(list) if list.is_empty() => expr.clone(),
        Sexp::List(list) => {
            let simplified: Vec<Sexp> = list.iter().map(simplify_sum_expr).collect();

            if let Sexp::Atom(Atom::S(op)) = &simplified[0] {
                if op == "+" {
                    let mut sum = 0i64;
                    let mut non_consts = Vec::new();

                    for item in &simplified[1..] {
                        if let Some(val) = extract_constant(item) {
                            sum += val;
                        } else {
                            non_consts.push(item.clone());
                        }
                    }

                    let mut new_expr = vec![Sexp::Atom(Atom::S("+".into()))];
                    new_expr.extend(non_consts);

                    if sum != 0 || new_expr.len() == 1 {
                        new_expr.push(Sexp::Atom(Atom::I(sum)));
                    }

                    if new_expr.len() == 2 {
                        return new_expr[1].clone();
                    }

                    return Sexp::List(new_expr);
                }
            }

            Sexp::List(simplified)
        }
    }
}
/// Recursively parses and simplifies an S-expression into an expression tree.
/// Also simplifies leaf expressions involving constant addition.
fn parse_expr_tree(expr: &Sexp) -> ExprTree {
    let normalized = normalize_gt_to_ge(expr);
    let maybe_const_sum = try_eval_const_sum(&normalized);
    let simplified = maybe_const_sum.unwrap_or(normalized.clone());

    match &simplified {
        Sexp::List(list) if !list.is_empty() => {
            if let Sexp::Atom(Atom::S(op)) = &list[0] {
                match op.as_str() {
                    "and" => ExprTree::And(list[1..].iter().map(parse_expr_tree).collect()),
                    "or" => ExprTree::Or(list[1..].iter().map(parse_expr_tree).collect()),
                    "not" => ExprTree::Not(Box::new(parse_expr_tree(&list[1]))),
                    "=>" => ExprTree::Implies(
                        Box::new(parse_expr_tree(&list[1])),
                        Box::new(parse_expr_tree(&list[2])),
                    ),
                    _ => ExprTree::Leaf(simplified),
                }
            } else {
                ExprTree::Leaf(simplified)
            }
        }
        _ => ExprTree::Leaf(simplified),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expr_tree_simple() {
        let input = "(exists ((t0 Int)(t1 Int)(t2 Int)) (and (>= t0 0) (>= t1 0) (>= t2 0) (= x (+ t0 (* t1 -1)))))";
        process_formula(input);
    }

    #[test]
    fn test_parse_nested_and_or_implies() {
        let input = "(exists ((t0 Int)(t1 Int)) (and (and (>= t0 0) (>= t1 0)) (and (or (= x t0) (not (= y t1))) (=> (> t0 0) (> t1 0)))))";
        process_formula(input);
    }

    #[test]
    fn test_parse_complex_expr() {
        let input = "(exists ((t0 Int)(t1 Int)(t2 Int)(t3 Int)(t4 Int)(t5 Int)(t6 Int))
          (and
            (>= t0 0)(>= t1 0)(>= t2 0)(>= t3 0)(>= t4 0)(>= t5 0)(>= t6 0)
            (and
              (= G___ (+ (* t3 -1) t5 1))
              (= L1 (+ t0 (* t3 -1)))
              (= L2 (+ (* t1 -1) t4))
              (= R0 t1)
              (= L3 (+ (* t2 -1) t5))
              (= R1 t2)
              (= L4 (+ t3 (* t4 -1) (* t5 -1)))
              (= G1 (+ t3 (* t5 -1)))
            )
            (and (=> (> t6 0) (> t0 0)) (=> (> t6 0) (> t3 0)))
            (or (> L1 3) (> G___ 0) (> L4 0) (> L6 L20))
          ))";
        process_formula(input);
    }

    #[test]
    fn test_tree_structure_output() {
        let input = "(exists ((t0 Int)(t1 Int)) (and (>= t0 0) (or (= x t0) (not (= y t1)))))";
        let parsed = parse(input).expect("parse error");
        let list = match parsed {
            Sexp::List(l) => l,
            _ => panic!("expected list"),
        };
        let tree = parse_expr_tree(&list[2]);
        println!("Test tree structure: {:#?}", tree);
    }
}




