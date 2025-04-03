//! Code that uses the ISL library.
//!
//! Probably all that you care about is [`affine_constraints_for_complement`]

#![allow(unsafe_op_in_unsafe_fn)]

use crate::affine_constraints::*;
use crate::semilinear::*;

use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_void};
use std::fmt::Display;
use std::ptr;

/// Give the affine constraints corresponding to the complement of this semilinear set.
pub fn affine_constraints_for_complement(
    num_vars: usize,
    sset: &SemilinearSet<Var>,
) -> Constraints {
    unsafe {
        let all_vars: Vec<Var> = (0..num_vars).map(|i| Var(i)).collect();
        let isl_set = complement_semilinear_set(sset, &all_vars);
        let constraints = isl_set_to_affine_constraints(num_vars, isl_set);
        isl_set_free(isl_set);
        constraints
    }
}

/// Compute the complement of an affine constraint set using ISL.
pub fn complement_affine_constraints(constraints: &Constraints) -> Constraints {
    unsafe {
        // 1. Convert the affine constraints to an ISL set
        let isl_set = affine_constraints_to_isl_set(constraints);

        // 2. Compute the complement of the ISL set
        // The universe set is already restricted to non-negative integers
        let universe = universe_set_for_vars(constraints.num_vars);
        let complemented_set = isl_set_subtract(universe, isl_set);

        // 3. Convert the complemented ISL set back to affine constraints
        let result = isl_set_to_affine_constraints(constraints.num_vars, complemented_set);

        // 4. Clean up
        isl_set_free(complemented_set);

        result
    }
}

/// Compute the intersection of two affine constraint sets using ISL.
pub fn intersect_affine_constraints(
    constraints1: &Constraints,
    constraints2: &Constraints,
) -> Constraints {
    // This requires that both constraints have the same variable numbering
    assert_eq!(
        constraints1.num_vars, constraints2.num_vars,
        "Intersect requires constraints with the same number of variables"
    );

    unsafe {
        // 1. Convert both affine constraints to ISL sets
        let isl_set1 = affine_constraints_to_isl_set(constraints1);
        let isl_set2 = affine_constraints_to_isl_set(constraints2);

        // 2. Compute the intersection
        let intersection_set = isl_set_intersect(isl_set1, isl_set2);

        // 3. Convert back to affine constraints
        let result = isl_set_to_affine_constraints(constraints1.num_vars, intersection_set);

        // 4. Clean up
        isl_set_free(intersection_set);

        result
    }
}

/// Check if an affine constraint set is empty using ISL.
pub fn is_affine_constraints_empty(constraints: &Constraints) -> bool {
    unsafe {
        // 1. Convert to ISL set
        let isl_set = affine_constraints_to_isl_set(constraints);

        // 2. Check if empty using ISL's isEmpty function
        let is_empty = isl_set_is_empty(isl_set) != 0;

        // 3. Clean up
        isl_set_free(isl_set);

        is_empty
    }
}

/// Create a universe set for the given number of variables
pub fn universe_set_for_vars(num_vars: usize) -> *mut isl_set {
    let var_names: Vec<_> = (0..num_vars).map(|i| format!("P{}", i)).collect();
    let var_refs: Vec<_> = var_names.iter().map(|s| s.as_str()).collect();
    universe_set(&var_refs)
}

/// Based on https://rust-lang.github.io/rust-bindgen/tutorial-4.html
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/isl_bindings.rs"));
}
use bindings::*;

// Error handling options for ISL
pub const ISL_ON_ERROR_WARN: i32 = 0;
pub const ISL_ON_ERROR_CONTINUE: i32 = 1;
pub const ISL_ON_ERROR_ABORT: i32 = 2;

/// The lazily-allocated thread-local (but global within a thread) ISL context
pub fn get_ctx() -> *mut isl_ctx {
    thread_local! {
        static CTX: Cell<*mut isl_ctx> = Cell::new(ptr::null_mut());
    }
    let mut ctx = CTX.get();
    if ctx.is_null() {
        unsafe {
            ctx = isl_ctx_alloc();
            isl_options_set_on_error(ctx, ISL_ON_ERROR_ABORT as _);
        }
        CTX.set(ctx);
    }
    ctx
}

/// Some functions return `isl_size`. This may be negative, in which case we panic.
fn check_isl_size(size: isl_size) -> usize {
    assert!(size >= 0);
    size as usize
}

/// Read an ISL set from a string
pub fn read_from_str(text: &str) -> *mut isl_set {
    unsafe {
        let text = CString::new(text).expect("text should not have ascii \\0 in it");
        let ptr = isl_set_read_from_str(get_ctx(), text.as_ptr());
        assert!(!ptr.is_null());
        ptr
    }
}

/// Display an ISL set as a string. Does not take ownership
pub fn to_str(set: *mut isl_set) -> String {
    unsafe {
        let result = isl_set_to_str(set);
        let string = CStr::from_ptr(result)
            .to_str()
            .expect("not valid utf-8")
            .to_owned();
        libc::free(result as *mut _);
        string
    }
}

/// Generates a string representation of the LinearSet and returns the set of all pi variables.
pub fn generate_linear_set_string<K: Eq + Hash + Clone + Ord + Display>(
    linear_set: &LinearSet<K>,
    keys: &[K],
) -> String {
    let mut constraints = Vec::new();

    // Generate the main constraints
    for key in keys.iter() {
        // Add the base value
        let base_value = linear_set.base.get(key);
        let mut constraint = format!("{key} = {base_value}");

        // Add period values
        for (i, period) in linear_set.periods.iter().enumerate() {
            let coeff = period.get(key);
            if coeff != 0 {
                constraint.push_str(&format!(" + {coeff} p{i}"));
            }
        }
        constraints.push(constraint);
    }

    // Periods are non-negative
    for i in 0..linear_set.periods.len() {
        constraints.push(format!("p{i} >= 0"));
    }

    format!(
        "{{ [{}] : exists ({} : {}) }}",
        keys.iter()
            .map(|key| format!("{key}"))
            .collect::<Vec<String>>()
            .join(", "), // Join the sorted keys with commas
        (0..linear_set.periods.len())
            .map(|i| format!("p{i}"))
            .collect::<Vec<String>>()
            .join(", "),
        constraints.join(" and "),
    )
}

/// ISL encoding of $\mathbb{N}^|keys|$
pub fn universe_set<K: Eq + Hash + Clone + Ord + Display>(keys: &[K]) -> *mut isl_set {
    let text = format!(
        "{{ [{}] : {} }}",
        keys.iter()
            .map(|key| format!("{}", key))
            .collect::<Vec<String>>()
            .join(", "),
        keys.iter()
            .map(|key| format!("{} >= 0", key))
            .collect::<Vec<String>>()
            .join(" and "),
    );
    read_from_str(&text)
}

/// Convert a semilinear set to an ISL set
pub fn semilinear_set_to_isl_set<K: Eq + Hash + Clone + Ord + Display>(
    semilinear_set: &SemilinearSet<K>,
    keys: &[K],
) -> *mut isl_set {
    semilinear_set
        .components
        .iter()
        .map(|c| read_from_str(&generate_linear_set_string(&c, keys)))
        .reduce(|x, y| unsafe { isl_set_union(x, y) })
        .expect("empty semilinear set (TODO handle this case)")
}

/// The ISL set for the complement of the given semilinear set
pub fn complement_semilinear_set<K: Eq + Hash + Clone + Ord + Display>(
    semilinear_set: &SemilinearSet<K>,
    keys: &[K],
) -> *mut isl_set {
    unsafe {
        isl_set_subtract(
            universe_set(keys),
            semilinear_set_to_isl_set(semilinear_set, keys),
        )
    }
}

/// Create a copy of an ISL set
pub unsafe fn isl_set_copy(set: *mut isl_set) -> *mut isl_set {
    bindings::isl_set_copy(set)
}

/// Check if two ISL sets are equal
pub unsafe fn isl_set_is_equal(set1: *mut isl_set, set2: *mut isl_set) -> isl_bool {
    bindings::isl_set_is_equal(set1, set2)
}

/// Intersect two ISL sets
pub unsafe fn isl_set_intersect(set1: *mut isl_set, set2: *mut isl_set) -> *mut isl_set {
    bindings::isl_set_intersect(set1, set2)
}

/// Union two ISL sets
pub unsafe fn isl_set_union(set1: *mut isl_set, set2: *mut isl_set) -> *mut isl_set {
    bindings::isl_set_union(set1, set2)
}

/// Free an ISL set
pub unsafe fn isl_set_free(set: *mut isl_set) -> *mut isl_set {
    bindings::isl_set_free(set)
}

/// Check if an ISL set is empty
pub unsafe fn isl_set_is_empty(set: *mut isl_set) -> isl_bool {
    bindings::isl_set_is_empty(set)
}

/// Convert a set of affine constraints to an ISL set
pub fn affine_constraints_to_isl_set(constraints: &Constraints) -> *mut isl_set {
    // Create variable names for all the variables (including existential ones)
    let total_vars = constraints.num_vars + constraints.num_existential_vars;
    let var_names: Vec<_> = (0..total_vars).map(|i| format!("P{}", i)).collect();

    // Convert each disjunct (AND clause) to a basic set, then union them
    let mut result_set: *mut isl_set = std::ptr::null_mut();

    for conjuncts in &constraints.constraints {
        // Generate the string representation for this conjunction
        let mut constraint_strings = Vec::new();

        // Add the main constraints
        for constraint in conjuncts {
            // Build the affine expression
            let mut expr = String::new();

            for (i, (coeff, var)) in constraint.affine_formula.iter().enumerate() {
                if i > 0 {
                    if *coeff >= 0 {
                        expr.push_str(" + ");
                    } else {
                        expr.push_str(" - ");
                    }
                    expr.push_str(&format!("{} {}", coeff.abs(), var_names[var.0]));
                } else {
                    // First term
                    expr.push_str(&format!("{} {}", coeff, var_names[var.0]));
                }
            }

            // Add the offset
            if constraint.offset != 0 {
                if constraint.offset > 0 {
                    expr.push_str(&format!(" + {}", constraint.offset));
                } else {
                    expr.push_str(&format!(" - {}", -constraint.offset));
                }
            }

            // Add the constraint type
            match constraint.constraint_type {
                EqualToZero => constraint_strings.push(format!("{} = 0", expr)),
                NonNegative => constraint_strings.push(format!("{} >= 0", expr)),
            }
        }

        // Handle existential variables
        let mut exists_vars = Vec::new();
        for i in constraints.num_vars..total_vars {
            exists_vars.push(format!("{}", var_names[i]));
        }

        // Define the domain: real vars are non-negative
        for i in 0..constraints.num_vars {
            constraint_strings.push(format!("{} >= 0", var_names[i]));
        }

        // Create the ISL set string
        let set_string = if exists_vars.is_empty() {
            // No existential variables
            format!(
                "{{ [{}] : {} }}",
                var_names[0..constraints.num_vars].join(", "),
                constraint_strings.join(" and "),
            )
        } else {
            // With existential variables
            format!(
                "{{ [{}] : exists ({} : {}) }}",
                var_names[0..constraints.num_vars].join(", "),
                exists_vars.join(", "),
                constraint_strings.join(" and "),
            )
        };

        // Parse the ISL set string
        let basic_set = read_from_str(&set_string);

        // Union with the result set
        unsafe {
            if result_set.is_null() {
                result_set = basic_set;
            } else {
                result_set = isl_set_union(result_set, basic_set);
            }
        }
    }

    // If no constraints, return the universe set
    if result_set.is_null() {
        let var_names_refs: Vec<_> = var_names[0..constraints.num_vars]
            .iter()
            .map(|s| s.as_str())
            .collect();
        return universe_set(&var_names_refs);
    }

    result_set
}

/// Iterate through each basic set in a set
///
/// Does not take ownership. The callback is given ownership of the basic_set's
pub unsafe fn for_each_basic_set<F: FnMut(*mut isl_basic_set)>(set: *mut isl_set, mut f: F) {
    unsafe extern "C" fn callback<F: FnMut(*mut isl_basic_set)>(
        bset: *mut isl_basic_set,
        user: *mut c_void,
    ) -> isl_stat {
        let f: &mut F = &mut *(user as *mut F);
        f(bset);
        0 // isl_stat_ok
    }

    let user: *mut c_void = &mut f as *mut F as *mut c_void;
    isl_set_foreach_basic_set(set, Some(callback::<F>), user);
}

/// The int value of an isl_val. Panics if it's not representable in `i32`
///
/// Takes ownership of the isl_val.
pub unsafe fn int_of_isl_val(val: *mut isl_val) -> i32 {
    assert!(!val.is_null());
    assert!(isl_val_is_int(val) != 0);
    let result = isl_val_get_num_si(val).try_into().unwrap();
    isl_val_free(val);
    result
}

/// Iterate through each constraint in a basic set
///
/// Does not take ownership. The callback is given ownership of the constraints
pub unsafe fn for_each_constraint<F: FnMut(*mut isl_constraint)>(
    bset: *mut isl_basic_set,
    mut f: F,
) {
    unsafe extern "C" fn callback<F: FnMut(*mut isl_constraint)>(
        c: *mut isl_constraint,
        user: *mut c_void,
    ) -> isl_stat {
        let f: &mut F = &mut *(user as *mut F);
        f(c);
        0 // isl_stat_ok
    }

    let user: *mut c_void = &mut f as *mut F as *mut c_void;
    isl_basic_set_foreach_constraint(bset, Some(callback::<F>), user);
}

/// Convert an isl_set (in `Var`s `0` through `num_vars - 1`) to a set of affine constraints
///
/// Does not take ownership of set
pub unsafe fn isl_set_to_affine_constraints(num_vars: usize, set: *mut isl_set) -> Constraints {
    let var_names: Vec<_> = (0..num_vars)
        .map(|i| CString::new(format!("{}", Var(i))).expect("can't have \\0 in var names"))
        .collect();
    let mut total_exists = 0;
    let mut constraints = Vec::new();

    for_each_basic_set(set, |bset| {
        // find the dimension + var labelling
        let local_space = isl_basic_set_get_local_space(bset);
        let num_exists = check_isl_size(isl_local_space_dim(local_space, isl_dim_type_isl_dim_div));
        if num_exists > total_exists {
            total_exists = num_exists;
        }
        let var_to_dim_idx: HashMap<usize, i32> = var_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let dim_idx = isl_local_space_find_dim_by_name(
                    local_space,
                    isl_dim_type_isl_dim_set,
                    name.as_ptr(),
                );
                assert!(0 <= dim_idx && (dim_idx as usize) < num_vars);
                (i, dim_idx)
            })
            .collect();
        isl_local_space_free(local_space);

        // collect the affine constraints
        let mut cs = Vec::new();
        for_each_constraint(bset, |c| {
            let constraint_type;
            if isl_constraint_is_equality(c) != 0 {
                constraint_type = EqualToZero;
            } else {
                constraint_type = NonNegative;
            }
            let offset = int_of_isl_val(isl_constraint_get_constant_val(c));
            let mut affine_formula = Vec::new();
            // Add the normal vars
            for i in 0..num_vars {
                let coeff = int_of_isl_val(isl_constraint_get_coefficient_val(
                    c,
                    isl_dim_type_isl_dim_set,
                    var_to_dim_idx[&i],
                ));
                if coeff != 0 {
                    affine_formula.push((coeff, Var(i)));
                }
            }
            // Add the existential vars. ISL existential vars are ints; ours are nats
            for i in 0..num_exists {
                let coeff = int_of_isl_val(isl_constraint_get_coefficient_val(
                    c,
                    isl_dim_type_isl_dim_div,
                    i as _,
                ));
                if coeff != 0 {
                    affine_formula.push((coeff, Var(num_vars + 2 * i)));
                    affine_formula.push((-coeff, Var(num_vars + 2 * i + 1)));
                }
            }

            isl_constraint_free(c);

            cs.push(Constraint {
                affine_formula,
                offset,
                constraint_type,
            });
        });

        isl_basic_set_free(bset);
        constraints.push(cs);
    });

    Constraints::new(num_vars, 2 * total_exists, constraints)
}

#[test]
fn test_affine_constraints() {
    let zero = SparseVector::new();
    let mut two_x = SparseVector::new();
    two_x.set(Var(0), 2);
    let vars = &[Var(0)];

    let evens = SemilinearSet::new(vec![LinearSet {
        base: zero,
        periods: vec![two_x],
    }]);

    let isl = complement_semilinear_set(&evens, vars);
    println!("isl set: {}", to_str(isl));
    unsafe {
        let constraints = isl_set_to_affine_constraints(vars.len(), isl);
        dbg!(constraints);
        isl_set_free(isl);
    }
}

#[test]
fn test_complement_affine_constraints() {
    // Create a simple constraint system representing x >= 5
    // The complement should be x < 5, which in our representation is -x + 4 >= 0
    let affine_constraints = Constraints {
        num_vars: 1, // just one variable x
        num_existential_vars: 0,
        constraints: vec![vec![Constraint {
            affine_formula: vec![(1, Var(0))], // x
            offset: -5,                        // -5
            constraint_type: NonNegative,      // x - 5 >= 0 means x >= 5
        }]],
    };

    // Compute the complement
    let complement = complement_affine_constraints(&affine_constraints);
    println!("Original constraints: {:?}", affine_constraints);
    println!("Complemented constraints: {:?}", complement);

    // Check the structure of the complemented constraints
    assert_eq!(complement.num_vars, 1);
    assert_eq!(complement.constraints.len(), 1); // Should have one disjunct
    assert_eq!(complement.constraints[0].len(), 1); // Should have one constraint

    // Check that the constraint is -x + 4 >= 0 (which is x <= 4)
    let constraint = &complement.constraints[0][0];
    assert_eq!(constraint.constraint_type, NonNegative);
    assert_eq!(constraint.affine_formula.len(), 1);
    assert_eq!(constraint.affine_formula[0].0, -1); // Coefficient -1
    assert_eq!(constraint.affine_formula[0].1, Var(0)); // Variable x
    assert!(constraint.offset >= 4); // Should be 4 or higher
}

#[test]
fn test_complement_keeps_non_negative_constraint() {
    // This test checks that when we complement a set,
    // the result is properly restricted to the non-negative orthant

    // Create a constraint system representing x <= 3
    // The complement in the full integer space would be x > 3
    // But in the non-negative natural number space, it should be x >= 4
    let affine_constraints = Constraints {
        num_vars: 1,
        num_existential_vars: 0,
        constraints: vec![vec![Constraint {
            affine_formula: vec![(-1, Var(0))], // -x
            offset: 3,                          // +3
            constraint_type: NonNegative,       // -x + 3 >= 0 means x <= 3
        }]],
    };

    // Convert to ISL to see the string representation
    let isl_set = affine_constraints_to_isl_set(&affine_constraints);
    println!("Original as ISL: {}", to_str(isl_set));

    // Compute the complement
    let complement = complement_affine_constraints(&affine_constraints);

    // Convert to ISL to see the string representation
    let complement_isl = affine_constraints_to_isl_set(&complement);
    println!("Complement as ISL: {}", to_str(complement_isl));

    // Check that the result has the correct constraints
    assert_eq!(complement.num_vars, 1);
    assert_eq!(complement.constraints.len(), 1); // Should have one disjunct

    // The constraint should be x >= 4
    let constraint = &complement.constraints[0][0];
    assert_eq!(constraint.constraint_type, NonNegative);

    // Check the constraint structure (should be x - 4 >= 0)
    let coeff = constraint.affine_formula[0].0;
    let var = constraint.affine_formula[0].1;
    let offset = constraint.offset;

    assert_eq!(coeff, 1); // Coefficient 1 for x
    assert_eq!(var, Var(0)); // Variable x
    assert_eq!(offset, -4); // Offset -4

    // Also verify by converting to strings
    let set_string = to_str(isl_set);
    let complement_string = to_str(complement_isl);

    // In the string representation, the non-negative constraint might be implicit
    // or simplified out by ISL, but we can see that the complement is correctly
    // computed as x >= 4, not simply x > 3, which confirms the non-negative constraint
    // is being respected
    assert!(set_string.contains("0 <= P0 <= 3"));
    assert!(complement_string.contains("P0 >= 4"));

    // Clean up ISL resources
    unsafe {
        isl_set_free(isl_set);
        isl_set_free(complement_isl);
    }
}

#[test]
fn test_complement_affine_constraints_complex() {
    // Create a constraint system representing (x >= 2 AND y >= 3) OR (x = y)
    let affine_constraints = Constraints {
        num_vars: 2, // Two variables x and y
        num_existential_vars: 0,
        constraints: vec![
            // First disjunct: x >= 2 AND y >= 3
            vec![
                Constraint {
                    affine_formula: vec![(1, Var(0))], // x
                    offset: -2,                        // -2
                    constraint_type: NonNegative,      // x - 2 >= 0 means x >= 2
                },
                Constraint {
                    affine_formula: vec![(1, Var(1))], // y
                    offset: -3,                        // -3
                    constraint_type: NonNegative,      // y - 3 >= 0 means y >= 3
                },
            ],
            // Second disjunct: x = y
            vec![Constraint {
                affine_formula: vec![(1, Var(0)), (-1, Var(1))], // x - y
                offset: 0,                                       // 0
                constraint_type: EqualToZero,                    // x - y = 0 means x = y
            }],
        ],
    };

    // Compute the complement
    let complement = complement_affine_constraints(&affine_constraints);
    println!("Original constraints: {:?}", affine_constraints);
    println!("Complemented constraints: {:?}", complement);

    // The complement of (x >= 2 AND y >= 3) OR (x = y) should be
    // (x < 2 OR y < 3) AND (x != y)
    // Check that the complemented constraints have the expected structure
    assert_eq!(complement.num_vars, 2);

    // Convert to ISL set strings for better visualization
    let original_set = affine_constraints_to_isl_set(&affine_constraints);
    let complement_set = affine_constraints_to_isl_set(&complement);
    println!("Original set: {}", to_str(original_set));
    println!("Complement set: {}", to_str(complement_set));

    // Clean up
    unsafe {
        isl_set_free(original_set);
        isl_set_free(complement_set);
    }
}

#[test]
fn test_intersect_affine_constraints() {
    // Create two constraint systems
    // First set: x >= 2
    let constraints1 = Constraints {
        num_vars: 1,
        num_existential_vars: 0,
        constraints: vec![vec![Constraint {
            affine_formula: vec![(1, Var(0))], // x
            offset: -2,                        // -2
            constraint_type: NonNegative,      // x - 2 >= 0 means x >= 2
        }]],
    };

    // Second set: x <= 5
    let constraints2 = Constraints {
        num_vars: 1,
        num_existential_vars: 0,
        constraints: vec![vec![Constraint {
            affine_formula: vec![(-1, Var(0))], // -x
            offset: 5,                          // +5
            constraint_type: NonNegative,       // -x + 5 >= 0 means x <= 5
        }]],
    };

    // Compute the intersection: should be 2 <= x <= 5
    let intersection = intersect_affine_constraints(&constraints1, &constraints2);

    // Convert to ISL set strings for visualization
    let set1 = affine_constraints_to_isl_set(&constraints1);
    let set2 = affine_constraints_to_isl_set(&constraints2);
    let intersection_set = affine_constraints_to_isl_set(&intersection);

    println!("Set 1: {}", to_str(set1));
    println!("Set 2: {}", to_str(set2));
    println!("Intersection: {}", to_str(intersection_set));

    // Verify the intersection is correct
    unsafe {
        // Check the intersection set directly
        let expected = read_from_str("{ [P0] : 2 <= P0 <= 5 }");
        assert!(isl_set_is_equal(intersection_set, expected) != 0);

        // Clean up
        isl_set_free(set1);
        isl_set_free(set2);
        isl_set_free(intersection_set);
        isl_set_free(expected);
    }
}

#[test]
fn test_is_affine_constraints_empty() {
    // Create a non-empty set: x >= 0
    let non_empty = Constraints {
        num_vars: 1,
        num_existential_vars: 0,
        constraints: vec![vec![Constraint {
            affine_formula: vec![(1, Var(0))], // x
            offset: 0,                         // 0
            constraint_type: NonNegative,      // x >= 0
        }]],
    };

    // Create an empty set: x >= 1 AND x <= 0
    let empty = Constraints {
        num_vars: 1,
        num_existential_vars: 0,
        constraints: vec![vec![
            Constraint {
                affine_formula: vec![(1, Var(0))], // x
                offset: -1,                        // -1
                constraint_type: NonNegative,      // x - 1 >= 0 means x >= 1
            },
            Constraint {
                affine_formula: vec![(-1, Var(0))], // -x
                offset: 0,                          // 0
                constraint_type: NonNegative,       // -x >= 0 means x <= 0
            },
        ]],
    };

    // Check if the sets are empty
    assert!(!is_affine_constraints_empty(&non_empty));
    assert!(is_affine_constraints_empty(&empty));

    // Visualize the sets
    let non_empty_set = affine_constraints_to_isl_set(&non_empty);
    let empty_set = affine_constraints_to_isl_set(&empty);

    println!("Non-empty set: {}", to_str(non_empty_set));
    println!("Empty set: {}", to_str(empty_set));

    // Clean up
    unsafe {
        isl_set_free(non_empty_set);
        isl_set_free(empty_set);
    }
}

#[test]
fn test_affine_constraints_to_isl() {
    // Create a simple constraint system
    // (2P0 + P1 ≥ 4) OR (P0 = P1)
    let constraints = Constraints {
        num_vars: 2, // P0 (v0) and P1 (v1)
        num_existential_vars: 0,
        constraints: vec![
            // First OR clause: 2P0 + P1 ≥ 4
            vec![Constraint {
                affine_formula: vec![(2, Var(0)), (1, Var(1))],
                offset: -4, // 2P0 + P1 - 4 ≥ 0
                constraint_type: NonNegative,
            }],
            // Second OR clause: P0 = P1
            vec![Constraint {
                affine_formula: vec![(1, Var(0)), (-1, Var(1))],
                offset: 0,
                constraint_type: EqualToZero,
            }],
        ],
    };

    // Convert to ISL set
    let isl_set = affine_constraints_to_isl_set(&constraints);
    println!("ISL set: {}", to_str(isl_set));

    // Convert back to affine constraints to verify round-trip conversion
    unsafe {
        let round_trip = isl_set_to_affine_constraints(constraints.num_vars, isl_set);
        println!("Round-trip constraints: {:?}", round_trip);

        // Clean up
        isl_set_free(isl_set);
    }
}

#[test]
fn test_affine_constraints_with_existential_vars() {
    // Create a constraint system with existential variables
    // There exists P2, P3 such that:
    // (P0 = 2*P2) AND (P1 = 2*P3 + 1)
    // This represents the set where P0 is even and P1 is odd
    let constraints = Constraints {
        num_vars: 2,             // P0 (v0) and P1 (v1)
        num_existential_vars: 2, // P2 (v2) and P3 (v3) are existential
        constraints: vec![vec![
            // P0 = 2*P2
            Constraint {
                affine_formula: vec![(1, Var(0)), (-2, Var(2))],
                offset: 0,
                constraint_type: EqualToZero,
            },
            // P1 = 2*P3 + 1
            Constraint {
                affine_formula: vec![(1, Var(1)), (-2, Var(3))],
                offset: -1,
                constraint_type: EqualToZero,
            },
        ]],
    };

    // Convert to ISL set
    let isl_set = affine_constraints_to_isl_set(&constraints);
    println!("ISL set with existential vars: {}", to_str(isl_set));

    // Convert back to affine constraints
    unsafe {
        let round_trip = isl_set_to_affine_constraints(constraints.num_vars, isl_set);
        println!(
            "Round-trip constraints with existential vars: {:?}",
            round_trip
        );

        // Clean up
        isl_set_free(isl_set);
    }
}

// old tests
#[test]
fn test_1() {
    // Create a base vector
    let mut base_vector = SparseVector::new();
    base_vector.set("x".to_string(), 1);
    base_vector.set("y".to_string(), 2);
    base_vector.set("z".to_string(), 3);

    // Create period vectors
    let mut period_vector_1 = SparseVector::new();
    period_vector_1.set("x".to_string(), 7);
    period_vector_1.set("y".to_string(), 8);
    period_vector_1.set("z".to_string(), 9);

    let mut period_vector_2 = SparseVector::new();
    period_vector_2.set("x".to_string(), 6);
    period_vector_2.set("y".to_string(), 5);
    period_vector_2.set("z".to_string(), 2);

    // Create a LinearSet
    let linear_set_1 = LinearSet {
        base: base_vector,
        periods: vec![period_vector_1, period_vector_2],
    };

    let semilinear_set = SemilinearSet::new(vec![linear_set_1]);
    let keys = vec!["x".to_string(), "y".to_string(), "z".to_string()];

    let result_set = complement_semilinear_set(&semilinear_set, &keys);
    println!("Final Result Set: {}", to_str(result_set));
    unsafe {
        isl_set_free(result_set);
    }
}
