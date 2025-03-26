use crate::semilinear::*;
use isl_rs::{Context, Set};
use std::collections::HashSet;
use std::fmt::Display;

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
pub fn universe_set<K: Eq + Hash + Clone + Ord + Display>(ctx: &Context, keys: &[K]) -> Set {
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
    Set::read_from_str(ctx, &text)
}

/// Convert a semilinear set to an ISL set
pub fn semilinear_set_to_isl_set<K: Eq + Hash + Clone + Ord + Display>(
    ctx: &Context,
    semilinear_set: &SemilinearSet<K>,
    keys: &[K],
) -> Set {
    semilinear_set
        .components
        .iter()
        .map(|c| Set::read_from_str(ctx, &generate_linear_set_string(&c, keys)))
        .reduce(Set::union)
        .expect("empty semilinear set (TODO handle this case)")
}

/// The ISL set for the complement of the given semilinear set
pub fn complement_semilinear_set<K: Eq + Hash + Clone + Ord + Display>(
    ctx: &Context,
    semilinear_set: &SemilinearSet<K>,
    keys: &[K],
) -> Set {
    universe_set(ctx, keys).subtract(semilinear_set_to_isl_set(ctx, semilinear_set, keys))
}

// old tests
#[test]
pub fn test_1() {
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

    let ctx = Context::alloc();
    let result_set = complement_semilinear_set(&ctx, &semilinear_set, &keys);
    println!("Final Result Set: {}", result_set.to_str());
}
