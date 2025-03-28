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

/// Based on https://rust-lang.github.io/rust-bindgen/tutorial-4.html
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/isl_bindings.rs"));
}
use bindings::*;

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
        let sset = semilinear_set_to_isl_set(semilinear_set, keys);
        dbg!(to_str(sset));
        let complement = isl_set_subtract(universe_set(keys), sset);
        dbg!(to_str(complement));
        complement
    }
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

    Constraints {
        num_vars,
        num_existential_vars: 2 * total_exists,
        constraints,
    }
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
