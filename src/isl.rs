use crate::semilinear::{Hash, LinearSet, SemilinearSet, SparseVector};
use isl_rs::{Aff, BasicSet, Context, DimType, Mat, Set, Space};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};

impl<K: Eq + Hash + Clone + Ord> SemilinearSet<K> {
    /// Returns a set of all unique keys (names) in all base and period SparseVectors.
    pub fn get_unique_keys(&self) -> HashSet<K> {
        let mut unique_keys = HashSet::new();

        // Iterate over all LinearSet components
        for linear_set in &self.components {
            // Add keys from the base SparseVector
            for key in linear_set.base.values.keys() {
                unique_keys.insert(key.clone());
            }

            // Add keys from all period SparseVectors
            for period in &linear_set.periods {
                for key in period.values.keys() {
                    unique_keys.insert(key.clone());
                }
            }
        }

        unique_keys
    }
}

/// Generates a string representation of the LinearSet and returns the set of all pi variables.
pub fn generate_linear_set_string<K: Eq + Hash + Clone + Ord + Debug + Display>(
    linear_set: &LinearSet<K>,
    unique_keys: &HashSet<K>,
    period_counter: usize,
) -> (String, usize) {
    let mut result = String::new();
    let mut pi_variables = HashSet::new();
    let mut current_counter = period_counter;

    // First extract only keys that exist in this linear set
    let mut relevant_keys: Vec<&K> = unique_keys.iter()
        .filter(|key| {
            // Check if key has non-zero value in base or any period
            linear_set.base.get(key).ne(&0) ||
                linear_set.periods.iter().any(|p| p.get(key).ne(&0))
        })
        .collect();

    // Convert to sorted Vec (now only for relevant keys)
    relevant_keys.sort();

    // Convert the HashSet of keys into a sorted Vec
    let mut sorted_keys: Vec<&K> = unique_keys.iter().collect();
    sorted_keys.sort(); // Sort the keys



    // Generate the main string
    for (i, key) in relevant_keys.iter().enumerate() {
        if i > 0 {
            result.push_str(" and ");
        }

        // Add the base value
        let base_value = linear_set.base.get(key);
        result.push_str(&format!("{} = {}", key, base_value));

        // Add period values
        for (period_index, period) in linear_set.periods.iter().enumerate() {
            let period_value = period.get(key);
            if period_value != 0 {
                let pi_var = format!("p{}", current_counter + period_index + 1);
                result.push_str(&format!(" + {} {}", period_value, pi_var));
                pi_variables.insert(pi_var);
            }
        }
    }

    current_counter += linear_set.periods.len();

    // Generate the prefix
    let mut sorted_pi_variables: Vec<String> = pi_variables.iter().cloned().collect();
    sorted_pi_variables.sort(); // Sort the pi variables
    let prefix_keys: Vec<String> = sorted_keys.iter().map(|key| format!("{}", key)).collect();
    let prefix = format!(
        "{{[{}] : exists ({} : ",
        prefix_keys.join(", "), // Join the sorted keys with commas
        sorted_pi_variables.join(", ")
    );

    // Generate the suffix
    let suffix = format!(
        " and {} and {} )}}",
        sorted_pi_variables
            .iter()
            .map(|var| format!("{} >= 0", var))
            .collect::<Vec<String>>()
            .join(" and "),
        sorted_keys
            .iter()
            .map(|key| format!("{} >= 0", key)) // Assuming keys are also non-negative
            .collect::<Vec<String>>()
            .join(" and ")
    );

    // Combine prefix, main string, and suffix
    let final_string = format!("{}{}{}", prefix, result, suffix);

    println!("{:}", "string generated is:");
    println!("{:}", final_string);
    (format!("{}{}{}", prefix, result, suffix), current_counter)
}

// This function receives a hash map and outputs a string representing the linear set of non-negatives
pub fn generate_string_of_linear_set_with_non_negatives<
    K: Eq + Hash + Clone + Ord + Debug + Display,
>(
    unique_keys: &HashSet<K>,
) -> String {
    let mut result = String::new();

    // Convert the HashSet of keys into a sorted Vec
    let mut sorted_keys: Vec<&K> = unique_keys.iter().collect();
    sorted_keys.sort(); // Sort the keys

    // Generate the prefix
    let prefix_keys: Vec<String> = sorted_keys.iter().map(|key| format!("{}", key)).collect();
    let prefix = format!(
        "{{[{}] : ", // Prefix with the sorted keys
        prefix_keys.join(", ")
    );

    // Generate the suffix with non-negativity constraints for all keys
    let suffix = format!(
        "{} }}",
        sorted_keys
            .iter()
            .map(|key| format!("{} >= 0", key)) // Add non-negativity constraints for each key
            .collect::<Vec<String>>()
            .join(" and ")
    );

    // Combine prefix and suffix
    let final_string = format!("{}{}", prefix, suffix);

    println!("{:}", "string generated for set of non-negatives:");
    println!("{:}", final_string);
    final_string
}

/// Function that iterates over all LinearSet components in a SemilinearSet,
/// and generates a hashSet with the equivalent ISL set components (before intersecting
/// and negating them)
pub fn translate_semilinear_set_to_ISL_sets<K: Eq + Hash + Clone + Ord + Debug + Display>(
    semilinear_set: &SemilinearSet<K>,
) -> Vec<Set> {
    let mut original_linear_sets_in_ISL_format = Vec::new();

    // extract unique keys in all SparsVectors in all LinearSet components of the SemilinearSet input
    let unique_keys = semilinear_set.get_unique_keys();

    let mut period_counter = 0;

    // Iterate over all LinearSet components
    for linear_set in &semilinear_set.components {
        // translate each single LinearSet to an equivalent ISL set format

        // Generate the string and pi variables
        let (string_encoding_of_set, new_counter) = generate_linear_set_string(&linear_set, &unique_keys, period_counter);
        period_counter = new_counter;


        // TODO - IMPORTANT!! Fix memory issues with the ctx object that Mark mentioned
        let ctx = Context::alloc();
        let isl_set_format = Set::read_from_str(&ctx, &string_encoding_of_set);
        original_linear_sets_in_ISL_format.push(isl_set_format);
    }

    original_linear_sets_in_ISL_format
}

// This function receves a SemilinearSet obejct and returns an ISL Set object representing
// the complement of the original SemiLinear Set (notice that the compliment is also semilinear,
// however, is  returned in the ISL Set format)
pub fn complement_semilinear_set<K: Eq + Hash + Clone + Ord + Debug + Display>(
    semilinear_set: &SemilinearSet<K>,
) -> Set {
    // Generate a collection (Rust Vector) of ISL sets, each corresponding to a LinearSet object of the SemilinearSet
    let vector_of_semilinear_set_translated_to_isl_sets =
        translate_semilinear_set_to_ISL_sets(&semilinear_set);

    // Create a vector of complemented sets directly in isl_main
    let vector_of_complemented_sets: Vec<Set> = vector_of_semilinear_set_translated_to_isl_sets
        .into_iter() // Consume the original vector
        .map(|isl_set| isl_set.complement()) // Negate each set
        .collect(); // Collect the results into a new vector

    // generate ISL set of non-negative values (to intersect later with all complemented ISL sets)
    let unique_keys = semilinear_set.get_unique_keys();
    let string_of_set_of_non_negatives =
        generate_string_of_linear_set_with_non_negatives(&unique_keys);
    let ctx_for_non_negatives = Context::alloc();
    let isl_set_of_non_negatives =
        Set::read_from_str(&ctx_for_non_negatives, &string_of_set_of_non_negatives);

    // Accumulate the result of intersections of the non-negative set with the complement sets
    let mut result_set = isl_set_of_non_negatives;

    // Intersect with all negated sets
    for complemented_isl_set in vector_of_complemented_sets {
        result_set = result_set.intersect(complemented_isl_set);
    }

    // Now `result_set` contains the final result of all intersections
    println!("Final Result Set: {}", result_set.to_str());

    result_set
}

// old tests
#[test]
pub fn test_1() {
    // todo - semilinear set #1
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

    let isl_set_representing_complement_of_ths_semilinear_set =
        complement_semilinear_set(&semilinear_set);
}

#[test]
pub fn test_2() {
    let ctx_1 = Context::alloc();
    // Create a new ISL context
    let set_1 = Set::read_from_str(&ctx_1, "{[x] : exists (k : x = 2 k and x >= 0 and k >= 0)}");
    println!("{:}", "DEFINE set_1");
    println!("{:}", set_1.to_str());

    let ctx_2 = Context::alloc();
    // Create a new ISL context
    let set_2 = Set::read_from_str(&ctx_2, "{[x] : exists (k : x = 5 k and x >= 0 and k >= 0)}");
    println!("{:}", "DEFINE set_2");
    println!("{:}", set_2.to_str());
    let set_3 = set_1.intersect(set_2);
    let set_3_copy = set_3.copy();

    println!("{:}", "set_3 = INTERSECT set_1 and set_2");
    println!("{:}", set_3.to_str());

    println!("{:}", "set_4 = COMPLEMENT set_3");
    let set_4 = set_3.complement();
    println!("{:}", set_4.to_str());

    let ctx_5 = Context::alloc();
    let set_5 = Set::read_from_str(&ctx_5, "{[x] : (x >= 0)}");
    println!("{:}", "set_5 = DEFINE {X | X>=0 }}");
    println!("{:}", set_5.to_str());

    let set_6 = set_4.intersect(set_5);
    println!("{:}", "set_6 = INTERSECT set_4 and set_5");
    println!("{:}", set_6.to_str());
    // let set_5 = set_;
    // println!("{:}", set_4.to_str());

    // todo - NEW (start)
    let ctx_7 = Context::alloc();
    // Create a new ISL context
    let set_7_string = "{[x,y,z] : \
    exists (p1_x,p1_y,p1_z,p2_x,p2_y,p2_z : \
    x = 1 + 7 p1_x + 6 p2_x and \
    y = 2 + 8 p1_y + 5 p2_y and \
    z = 3 + 9 p1_z + 2 p2_z and \
    p1_x >= 0 and \
    p1_y >= 0 and \
    p1_z >= 0 and \
    p2_x >= 0 and \
    p2_y >= 0 and \
    p2_z >= 0 \
    x >= 0 \
    y >= 0 \
    z >= 0 \
    )}";
    let set_7 = Set::read_from_str(&ctx_7, set_7_string);
    println!("{:}", "DEFINE set_7");
    println!("{:}", set_7.to_str());

    let ctx_8 = Context::alloc();
    let set_8 = Set::read_from_str(&ctx_8, "{[x,y,z] | (x = 20) and (y = 20) and (z = 16)}");
    // {[0,0,0],[1,2,3],[20,20,16]}
    println!("{:}", "DEFINE set_8");
    println!("{:}", set_8.to_str());

    let set_9 = set_7.intersect(set_8);
    println!("{:}", "set_9 = INTERSECT set_7 and set_8");
    println!("{:}", set_9.to_str());

    // todo delete - start

    let ctx_a = Context::alloc();
    let set_a = Set::read_from_str(&ctx_8, "{[x] :  exists (k : x = 2 k)}");
    println!("{:}", set_a.to_str());
    println!("{:}", "complement is: ");
    let set_b = set_a.complement();
    println!("{:}", set_b.to_str());
    //todo delete - end

    // //
    //semilinear
    // // todo - NEW (end)
    //
    //
    // println!("Hello, World!");
    // println!("boo");
}
