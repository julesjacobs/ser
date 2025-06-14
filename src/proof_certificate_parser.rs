use regex::Regex;
use sexp::{parse, Atom, Sexp}; // ensure the `sexp` crate is included in Cargo.toml: sexp = "0.5"
use crate::presburger::{Variable, PresburgerSet, QuantifiedSet, Constraint, ConstraintType};

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
/// assume the input starts with (exists (...))
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

/// Wraps standalone variables as (* var 1) only at the leaves,
/// and avoids double-wrapping inside expressions like (* t0 -1).
fn wrap_vars_in_mul(expr: &Sexp) -> Sexp {
    match expr {
        // If it's an atom and not an operator, wrap it as (* var 1)
        Sexp::Atom(Atom::S(s)) if !["+", "*", "=", ">=", ">", "<", "<=", "and", "or", "not", "=>"].contains(&s.as_str()) => {
            Sexp::List(vec![
                Sexp::Atom(Atom::S("*".into())),
                Sexp::Atom(Atom::S(s.clone())),
                Sexp::Atom(Atom::I(1)),
            ])
        }
        // Lists: process recursively, but don't re-wrap what's already in (* ...)
        Sexp::List(list) if !list.is_empty() => {
            if let Sexp::Atom(Atom::S(op)) = &list[0] {
                // If already a `(* ...)` application, leave as-is
                if op == "*" {
                    return Sexp::List(list.clone());
                }
            }

            let new_list: Vec<Sexp> = list.iter().map(wrap_vars_in_mul).collect();
            Sexp::List(new_list)
        }
        _ => expr.clone(), // Numbers and other atoms: leave unchanged
    }
}


/// Recursively parses and simplifies an S-expression into an expression tree.
/// Also simplifies leaf expressions involving constant addition.
fn parse_expr_tree(expr: &Sexp) -> ExprTree {
    let wrapped = wrap_vars_in_mul(expr);                      // <- Insert here
    let normalized = normalize_gt_to_ge(&wrapped);
    let maybe_const_sum = try_eval_const_sum(&normalized);
    let simplified = maybe_const_sum.unwrap_or(normalized.clone());
    let simplified_with_const = ensure_constants_in_plus(&simplified);
    let final_simplified = ensure_plus_rhs_for_equals(&simplified_with_const);

    match &final_simplified {
        Sexp::List(list) if !list.is_empty() => {
            if let Sexp::Atom(Atom::S(op)) = &list[0] {
                match op.as_str() {
                    "and" => ExprTree::And(
                        list[1..].iter().map(parse_expr_tree).collect()
                    ),
                    "or" => ExprTree::Or(
                        list[1..].iter().map(parse_expr_tree).collect()
                    ),
                    "not" => ExprTree::Not(
                        Box::new(parse_expr_tree(&list[1]))
                    ),
                    "=>" => ExprTree::Implies(
                        Box::new(parse_expr_tree(&list[1])),
                        Box::new(parse_expr_tree(&list[2])),
                    ),
                    _ => ExprTree::Leaf(final_simplified.clone()),
                }
            } else {
                ExprTree::Leaf(final_simplified.clone())
            }
        }
        _ => ExprTree::Leaf(final_simplified.clone()),
    }
}


// todo new - start

pub fn from_single_constraint_string(input: &str) -> PresburgerSet<String> {

    let input = input.trim();

    let re_eq = Regex::new(r"\(= ([^\s]+) \((.+)\)\)").unwrap();
    let re_ge = Regex::new(r"\(>= ([^\s]+) (-?\d+)\)").unwrap();

    let (lhs, linear_terms, constant_term, constraint_type) = if let Some(caps) = re_eq.captures(input) {
        let lhs = caps[1].to_string();
        let rhs = caps[2].trim();
        let mut linear_terms: Vec<(i32, String)> = vec![];

        let atom_re = Regex::new(r"\(t(\d+)\)").unwrap();
        let mul_re = Regex::new(r"\(\* t(\d+) (-?\d+)\)").unwrap();
        let add_re = Regex::new(r"\+").unwrap();

        // Handle (+ t3 (* t4 -1) (* t5 -1))
        for token in rhs.split_whitespace() {
            if token == "+" {
                continue;
            } else if let Some(cap) = mul_re.captures(token) {
                let var = format!("t{}", &cap[1]);
                let coef = cap[2].parse::<i32>().unwrap();
                linear_terms.push((coef, var));
            } else if let Some(cap) = atom_re.captures(token) {
                let var = format!("t{}", &cap[1]);
                linear_terms.push((1, var));
            }
        }

        linear_terms.push((-1, lhs.clone()));
        (lhs, linear_terms, 0, ConstraintType::EqualToZero)
    } else if let Some(caps) = re_ge.captures(input) {
        let lhs = caps[1].to_string();
        let c = caps[2].parse::<i32>().unwrap();
        let terms = vec![(1, lhs.clone())];
        (lhs, terms, -c, ConstraintType::NonNegative)
    } else {
        panic!("Unsupported constraint format: {}", input);
    };

    let mapping: Vec<String> = linear_terms
        .iter()
        .map(|(_, v)| v.clone())
        .chain(Some(lhs.clone()))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    let constraint = Constraint::new(
        linear_terms
            .into_iter()
            .map(|(c, v)| (c, Variable::Var(v)))
            .collect(),
        constant_term,
        constraint_type,
    );

    let qs = QuantifiedSet::new(vec![constraint]);
    PresburgerSet::from_quantified_sets(&[qs], mapping)
}



/// Ensures that any (+ ...) expression includes a constant term.
/// Traverses recursively through any S-expression.
fn ensure_constants_in_plus(expr: &Sexp) -> Sexp {
    match expr {
        Sexp::List(list) if !list.is_empty() => {
            if let Sexp::Atom(Atom::S(op)) = &list[0] {
                if op == "+" {
                    let mut new_list = list.clone();
                    let has_const = new_list.iter().skip(1).any(|item| extract_constant(item).is_some());
                    if !has_const {
                        new_list.push(Sexp::Atom(Atom::I(0)));
                    }
                    return Sexp::List(new_list);
                }
            }

            // Recurse into children
            let new_children: Vec<Sexp> = list.iter().map(ensure_constants_in_plus).collect();
            Sexp::List(new_children)
        }
        _ => expr.clone(),
    }
}


/// Ensures that the RHS of any (= lhs rhs) expression is a (+ ...) expression.
/// If the RHS is not already a (+ ...), it wraps it as (+ rhs 0).
fn ensure_plus_rhs_for_equals(expr: &Sexp) -> Sexp {
    if let Sexp::List(list) = expr {
        if list.len() == 3 {
            if let Sexp::Atom(Atom::S(op)) = &list[0] {
                if op == "=" {
                    let lhs = &list[1];
                    let rhs = &list[2];

                    let needs_wrapping = match rhs {
                        Sexp::List(rhs_list) => {
                            match rhs_list.first() {
                                Some(Sexp::Atom(Atom::S(s))) => s != "+",
                                _ => true,
                            }
                        }
                        _ => true,
                    };

                    if needs_wrapping {
                        let wrapped_rhs = Sexp::List(vec![
                            Sexp::Atom(Atom::S("+".into())),
                            rhs.clone(),
                            Sexp::Atom(Atom::I(0)),
                        ]);
                        return Sexp::List(vec![
                            Sexp::Atom(Atom::S("=".into())),
                            lhs.clone(),
                            wrapped_rhs,
                        ]);
                    }
                }
            }
        }
    }

    expr.clone()
}



// todo new - end

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

    #[test]
    fn test_tree_structure_output_2(){
        let input =
            "(exists ((t0 Int)(t1 Int)(t2 Int)(t3 Int)(t4 Int)(t5 Int)(t6 Int)(t7 Int)\
        (t8 Int)(t9 Int)(t10 Int)(t11 Int)(t12 Int)(t13 Int)(t14 Int)(t15 Int)(t16 Int)\
        (t17 Int)(t18 Int)(t19 Int)) \
        (and (>= t0 0)(>= t1 0)(>= t2 0)(>= t3 0)(>= t4 0)(>= t5 0)(>= t6 0)(>= t7 0)\
        (>= t8 0)(>= t9 0)(>= t10 0)(>= t11 0)(>= t12 0)(>= t13 0)(>= t14 0)(>= t15 0)(>= t16 0)\
        (>= t17 0)(>= t18 0)(>= t19 0) \
        (and (= G___ (+ (* t9 -1) t13 (* t15 -1) t19 1))\
        (= L______while_X____2___yield____X____X___1__REQ_incr \
        (+ t0 (* t15 -1) (* t16 -1)))(= L______while_X____0___yield____X____X___1__REQ_decr \
        (+ t1 (* t12 -1) (* t13 -1)))(= L______1__REQ_decr \
        (+ (* t2 -1) t9 t12))(= RESP_decr_REQ_1 t2)(= L______2__REQ_decr (+ (* t3 -1) t10))\
        (= RESP_decr_REQ_2 t3)(= L______0__REQ_decr (+ (* t4 -1) t13))(= RESP_decr_REQ_0 t4)\
        (= L______1__REQ_incr (+ (* t5 -1) t15 t18))(= RESP_incr_REQ_1 t5)(= L______2__REQ_incr \
        (+ (* t6 -1) t16))(= RESP_incr_REQ_2 t6)(= L______0__REQ_incr (+ (* t7 -1) t19))\
        (= RESP_incr_REQ_0 t7)(= L______while_X____2___yield____X____X___1__REQ_decr \
        (+ (* t9 -1) (* t10 -1)))(= G__X_1_ (+ t9 (* t10 -1) t12 (* t13 -1) t15 (* t16 -1) t18 \
        (* t19 -1)))(= G__X_2_ (+ t10 (* t12 -1) t16 (* t18 -1)))\
        (= L______while_X____0___yield____X____X___1__REQ_incr (+ (* t18 -1) (* t19 -1) 7)))))";

        let parsed = parse(input).expect("parse error");
        let list = match parsed {
            Sexp::List(l) => l,
            _ => panic!("expected list"),
        };
        let tree = parse_expr_tree(&list[2]);
        println!("Test tree structure: {:#?}", tree);
    }

    #[test]
    fn test_tree_structure_output_3(){
        let input =
            "(exists ((t0 Int)(t1 Int)(t2 Int)(t3 Int)(t4 Int)(t5 Int)(t6 Int)(t7 Int)(t8 Int)(t9 Int)\
        (t10 Int)(t11 Int)(t12 Int)(t13 Int)(t14 Int)) (and (>= t0 0)(>= t1 0)(>= t2 0)(>= t3 0)\
        (>= t4 0)(>= t5 0)(>= t6 0)(>= t7 0)(>= t8 0)(>= t9 0)(>= t10 0)(>= t11 0)(>= t12 0)\
        (>= t13 0)(>= t14 0) (and (= G___ (+ (* t7 -1) (* t8 -1) 1))\
        (= L_full_main_REQ_10 t3)(= L___s_1_t_1___3__REQ_main (+ (* t4 -1) t9))\
        (= RESP_main_REQ_3 t4)(= L___t_2___4__REQ_main \
        (+ (* t5 -1) t11 t14))(= RESP_main_REQ_4 t5)\
        (= L___c_2_s_1_t_1___19__REQ_main (+ (* t6 -1) t12))\
        (= RESP_main_REQ_19 t6)(= G__T_1_ (+ t7 (* t12 -1) (* t13 -1)))(= G__T_1_X_1_ \
        (+ t8 (* t9 -1) (* t10 -1)))(= G__T_2_X_1_ \
        (+ t9 t10 t13))(= G__T_2_ t12))))";

        let parsed = parse(input).expect("parse error");
        let list = match parsed {
            Sexp::List(l) => l,
            _ => panic!("expected list"),
        };
        let tree = parse_expr_tree(&list[2]);
        println!("Test tree structure: {:#?}", tree);
    }

    #[test]
    fn test_tree_structure_output_4(){
        let input =
            "(exists ((t0 Int)(t1 Int)(t2 Int)(t3 Int)(t4 Int)(t5 Int)(t6 Int)) \
        (and (and (and (>= t0 0)(>= t1 0)(>= t2 0)(>= t3 0)(>= t4 0)(>= t5 0)(>= t6 0) \
        (and (= G___ (+ (* t3 -1) t5 1))\
        (= L___full (+ t0 (* t3 -1)))\
        (= L______0__REQ_foo_with_locks (+ (* t1 -1) t4))\
        (= RESP_foo_with_locks_REQ_0 t1)(= L___y_1___1__REQ_foo_with_locks \
        (+ (* t2 -1) t5))(= RESP_foo_with_locks_REQ_1 t2)\
        (= L_with_locks \
        (+ t3 (* t4 -1) (* t5 -1)))(= G__L_1_X_1_ (+ t3 (* t5 -1))))) \
        (and (=> (> t6 0) (> t0 0))(=> (> t6 0) (> t3 0)))) \
        (or (> L__full2 0)(> G___ 0)(> L__with_locks 0)\
        (= L______1__REQ_decr (+ (* t2 -1) t12))(= RESP_decr_REQ_1 t2))))";

        let parsed = parse(input).expect("parse error");
        let list = match parsed {
            Sexp::List(l) => l,
            _ => panic!("expected list"),
        };
        let tree = parse_expr_tree(&list[2]);
        println!("Test tree structure: {:#?}", tree);
    }




    // #[test]
    // fn test_from_leaf_constraint_to_presburger_set(){
    //     let var_x1 = Variable::Var("x1");
    //     let var_x2 = Variable::Var("x2");
    //     let var_x3 = Variable::Var("x3");
    //     let var_x4 = Variable::Var("x4");
    //
    //     // (= (* x1 1) (+ x2 (* x3 -10) (* x4 +1) 6))
    //     let constraint1 = Constraint {
    //         linear_combination: vec![(1, var_t), (2, var_e)],
    //         constant_term: 5,
    //         constraint_type: ConstraintType::NonNegative,
    //     };
    //
    //     // // Test Constraint<T> Display implementation
    //     // let constraint1 = Constraint {
    //     //     linear_combination: vec![(1, var_t), (2, var_e)],
    //     //     constant_term: 5,
    //     //     constraint_type: ConstraintType::NonNegative,
    //     // };
    //     //
    //     // let constraint2 = Constraint {
    //     //     linear_combination: vec![(-1, var_t), (-1, var_e)],
    //     //     constant_term: 0,
    //     //     constraint_type: ConstraintType::EqualToZero,
    //     // };
    //
    //     assert_eq!(format!("{}", constraint1), "Vx + 2E3 + 5 ≥ 0");
    //     // assert_eq!(format!("{}", constraint2), "-Vx -E3 = 0");
    //
    //     let ps = from_single_constraint_string("(= L4 (+ t3 (* t4 -1) (* t5 -1)))");
    //     println!("{}", ps);
    //
    //     let ps2 = from_single_constraint_string("(>= L6 1)");
    //     println!("{}", ps2);
    //
    // }


}




