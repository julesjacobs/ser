// Semi-linear sets

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::clone::Clone;

use crate::kleene::Kleene;

/// A sparse vector in d-dimensional nonnegative integer space.
/// Keys represent dimensions and values represent the value at that dimension.
/// Dimensions not present in the HashMap are assumed to be 0.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SparseVector<K: Eq + Hash + Clone + Ord> {
    values: HashMap<K, usize>,
}

// Manual implementation of Hash for SparseVector by converting the HashMap to a sorted Vec
impl<K: Eq + Hash + Clone + Ord> Hash for SparseVector<K> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Convert the HashMap to a sorted Vec of (key, value) pairs
        let mut entries: Vec<_> = self.values.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));

        // Hash the sorted entries
        entries.hash(state);
    }
}

impl<K: Eq + Hash + Clone + Ord> SparseVector<K> {
    /// Create a new empty sparse vector (all zeros)
    fn new() -> Self {
        SparseVector {
            values: HashMap::new(),
        }
    }

    /// Get the value at a specific dimension
    fn get(&self, key: &K) -> usize {
        *self.values.get(key).unwrap_or(&0)
    }

    /// Set the value at a specific dimension
    fn set(&mut self, key: K, value: usize) {
        if value == 0 {
            self.values.remove(&key);
        } else {
            self.values.insert(key, value);
        }
    }

    /// Create a unit vector with 1 at the specified dimension
    fn unit(key: K) -> Self {
        let mut values = HashMap::new();
        values.insert(key, 1);
        SparseVector { values }
    }

    /// Add another vector to this one element-wise
    fn add(&self, other: &Self) -> Self {
        let mut result = self.clone();
        for (key, &value) in &other.values {
            let new_value = self.get(key) + value;
            result.set(key.clone(), new_value);
        }
        result
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LinearSet<K: Eq + Hash + Clone + Ord> {
    base: SparseVector<K>,            // u0: the base vector
    periods: Vec<SparseVector<K>>,    // [u1, u2, ..., um]: list of period generator vectors
}

#[derive(Debug, Clone)]
struct SemilinearSet<K: Eq + Hash + Clone + Ord> {
    components: Vec<LinearSet<K>>,  // finite list of linear sets whose union defines the set
}

impl<K: Eq + Hash + Clone + Ord> PartialEq for SemilinearSet<K> { // by Guy
    fn eq(&self, other: &Self) -> bool {
        let self_components: HashSet<_> = self.components.iter().cloned().collect();
        let other_components: HashSet<_> = other.components.iter().cloned().collect();
        self_components == other_components
    }
}

impl<K: Eq + Hash + Clone + Ord> SemilinearSet<K> {
    /// Create a new semilinear set from a list of LinearSet components.
    fn new(components: Vec<LinearSet<K>>) -> Self {
        // Filter out duplicate linear set components
        let mut new_components = HashSet::new();
        for lin in components {
            // Filter out duplicate period vectors
            let mut new_periods = HashSet::new();
            for p in lin.periods {
                new_periods.insert(p);
            }
            new_components.insert(LinearSet { base: lin.base, periods: new_periods.into_iter().collect() });
        }
        SemilinearSet { components: new_components.into_iter().collect() }
    }

    /// Check if the semilinear set is empty.
    fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Create a semilinear set containing a single vector (an atomic singleton).
    fn singleton(vector: SparseVector<K>) -> Self {
        SemilinearSet {
            components: vec![ LinearSet { base: vector, periods: vec![] } ],
        }
    }

    /// Singleton containing the zero vector.
    fn zero() -> Self {
        SemilinearSet::singleton(SparseVector::new())
    }

    /// The empty semilinear set (contains no vectors).
    fn empty() -> Self {
        SemilinearSet { components: Vec::new() }
    }

    /// The universe (all possible sparse vectors) as a semilinear set.
    /// This requires providing a set of possible dimensions.
    fn universe(keys: Vec<K>) -> Self {
        // Universe = linear set with base = empty (all zeros), periods = unit vectors for each key
        let base = SparseVector::new();
        let periods = keys.into_iter().map(SparseVector::unit).collect();
        SemilinearSet::new(vec![ LinearSet { base, periods } ])
    }
}

impl<K: Eq + Hash + Clone + Ord> Kleene for SemilinearSet<K> {
    fn zero() -> Self {
        SemilinearSet::empty()
    }

    fn one() -> Self {
        SemilinearSet::zero()
    }

    // Union of two semilinear sets.
    fn plus(self, other: Self) -> Self {
        // Clone components of both and combine
        let mut new_components = Vec::with_capacity(self.components.len() + other.components.len());
        new_components.extend(self.components.iter().cloned());
        new_components.extend(other.components.iter().cloned());
        // (TODO) we could attempt to simplify or merge components here.
        SemilinearSet::new(new_components)
    }

    // Sequential composition (a.k.a. Minkowski sum) of two semilinear sets.
    fn times(self, other: Self) -> Self {
        let mut result_components = Vec::new();
        for lin1 in &self.components {
            for lin2 in &other.components {
                // Compute the sum of lin1 and lin2 as a new LinearSet
                let new_base = lin1.base.add(&lin2.base);
                // periods: all periods from lin1 and lin2
                let mut new_periods = Vec::with_capacity(lin1.periods.len() + lin2.periods.len());
                new_periods.extend_from_slice(&lin1.periods);
                new_periods.extend_from_slice(&lin2.periods);
                // (TODO) remove duplicate period vectors in new_periods
                result_components.push( LinearSet { base: new_base, periods: new_periods } );
            }
        }
        SemilinearSet::new(result_components)
    }

    fn star(self) -> Self {
        let mut result_components = Vec::new();

        // We use bit masks to iterate over all non-empty subsets of components
        let n = self.components.len();
        // assert that the size is not too large
        debug_assert!(n <= 32, "Number of components in semilinear set is too large");
        for mask in 0..(1<<n) {
            // Determine subset X for this mask
            let mut subset_base = SparseVector::new();
            let mut subset_periods: Vec<SparseVector<K>> = Vec::new();
            // We'll also use a set to avoid duplicate period vectors
            let mut period_set = HashSet::new();

            for i in 0..n {
                if mask & (1<<i) != 0 {
                    let comp = &self.components[i];
                    // add this component's base to subset_base
                    subset_base = subset_base.add(&comp.base);
                    // include this component's base and periods in subset_periods
                    let base_vec = &comp.base;
                    if period_set.insert(base_vec) {
                        subset_periods.push(base_vec.clone());
                    }
                    for p in &comp.periods {
                        if period_set.insert(p) {
                            subset_periods.push(p.clone());
                        }
                    }
                }
            }
            // Create the linear set for this subset
            result_components.push( LinearSet { base: subset_base, periods: subset_periods } );
        }
        SemilinearSet::new(result_components)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_vector_operations() {
        let mut v1 = SparseVector::new();
        v1.set("x".to_string(), 1);
        v1.set("y".to_string(), 2);

        let mut v2 = SparseVector::new();
        v2.set("y".to_string(), 3);
        v2.set("z".to_string(), 4);

        let sum = v1.add(&v2);
        assert_eq!(sum.get(&"x".to_string()), 1);
        assert_eq!(sum.get(&"y".to_string()), 5);
        assert_eq!(sum.get(&"z".to_string()), 4);
        assert_eq!(sum.get(&"w".to_string()), 0);  // Non-existent key
    }

    #[test]
    fn test_semilinear_set_union() {
        let mut v1 = SparseVector::new();
        v1.set("x".to_string(), 1);
        v1.set("y".to_string(), 2);

        let mut v2 = SparseVector::new();
        v2.set("y".to_string(), 3);
        v2.set("z".to_string(), 4);

        let set1 = SemilinearSet::singleton(v1.clone());
        let set2: SemilinearSet<String> = SemilinearSet::singleton(v2.clone());
        let union = set1.plus(set2);

        assert_eq!(union.components.len(), 2);
        // Check that the components contain our original vectors
        assert!(union.components.iter().any(|c| c.base == v1));
        assert!(union.components.iter().any(|c| c.base == v2));
    }

    #[test]
    fn test_semilinear_set_add() {
        let mut v1 = SparseVector::new();
        v1.set("x".to_string(), 1);

        let mut v2 = SparseVector::new();
        v2.set("y".to_string(), 2);

        let set1 = SemilinearSet::singleton(v1);
        let set2 = SemilinearSet::singleton(v2);
        let sum = set1.times(set2);

        assert_eq!(sum.components.len(), 1);
        let result_vector = &sum.components[0].base;
        assert_eq!(result_vector.get(&"x".to_string()), 1);
        assert_eq!(result_vector.get(&"y".to_string()), 2);
    }

    //////////////////////////////////////////////
    ///Guy's Tests

    #[test]
    fn test_a_star() {

        let mut base = SparseVector::new();
        base.set("x".to_string(), 1);

        // a={(1,0,0);[]}
        let a = LinearSet {
            base:base.clone(),
            periods: vec![],
        };

        // computed a* result from code
        let result_a_star = SemilinearSet::new(vec![a]).star();


        // {(0,0,0);[]}
        let linear_set_1_base = SparseVector::new();
        let ground_truth_a_star_linear_set_1 = LinearSet {
            base:linear_set_1_base.clone(),
            periods: vec![],
        };

        // {(1,0,0);[(1,0,0)]}
        let mut linear_set_2_base = SparseVector::new();
        linear_set_2_base.set("x".to_string(), 1);

        let mut linear_set_2_period = SparseVector::new();
        linear_set_2_period.set("x".to_string(), 1);
        let ground_truth_a_star_linear_set_2 = LinearSet {
            base:linear_set_2_base.clone(),
            periods: vec![linear_set_2_period],
        };

        let ground_truth_a_star = SemilinearSet {
            components: vec![ground_truth_a_star_linear_set_1,ground_truth_a_star_linear_set_2]
        };

        assert_eq!(result_a_star, ground_truth_a_star);
    }

    #[test]
    fn test_a_star_proper() {
        // Use the Kleene operations to compute a*
        let a = SemilinearSet::singleton(SparseVector::unit("a"));
        let a_star = a.star();

        // Define the ground truth using the semilinear set constructors
        let ground_truth_a_star = SemilinearSet::new(vec![
            LinearSet {
                base: SparseVector::unit("a"),
                periods: vec![SparseVector::unit("a")],
            },
            LinearSet {
                base: SparseVector::new(),
                periods: vec![],
            },
        ]);

        assert_eq!(a_star, ground_truth_a_star);
    }


    #[test]
    fn test_b_times_c_proper() {
        // Use the Kleene operations to compute b;c
        let b = SemilinearSet::singleton(SparseVector::unit("b".to_string()));
        let c = SemilinearSet::singleton(SparseVector::unit("c".to_string()));
        let b_times_c = b.clone().times(c.clone());
        let c_times_b = c.times(b);
        // check symetry
        assert_eq!(b_times_c, c_times_b);


        let mut b_time_c_sparse_vector = SparseVector::new();
        b_time_c_sparse_vector.set("b".to_string(), 1);
        b_time_c_sparse_vector.set("c".to_string(), 1);


        // Define the ground truth using the semilinear set constructors
        let ground_truth_b_times_c = SemilinearSet::new(vec![
            LinearSet {
                base: b_time_c_sparse_vector,
                periods: vec![],
            }
        ]);

        assert_eq!(b_times_c, ground_truth_b_times_c);
        // println!("{:?}", b_times_c);
        // println!("done!!!");
    }


    #[test]
    fn test_a_star_times_b_proper() {
        // Use the Kleene operations to compute a*
        let a = SemilinearSet::singleton(SparseVector::unit("a".to_string()));
        let a_star = a.star();

        let b = SemilinearSet::singleton(SparseVector::unit("b".to_string()));
        // Use the Kleene operations to compute (a*);b
        let a_star_times_b = a_star.times(b);

        let mut a_b = SparseVector::new();
        a_b.set("a".to_string(), 1);
        a_b.set("b".to_string(), 1);

        // Define the ground truth using the semilinear set constructors
        let ground_truth_a_star_times_b = SemilinearSet::new(vec![
            LinearSet {
                base: SparseVector::unit("b".to_string()),  // Ensure consistency
                periods: vec![],
            },
            LinearSet {
                base: a_b,
                periods: vec![SparseVector::unit("a".to_string())],  // Ensure consistency
            },
        ]);

        assert_eq!(a_star_times_b, ground_truth_a_star_times_b);
    }


    #[test]
    fn test_a_star_times_b_plus_b_times_c_proper() {
        // Use the Kleene operations to compute a*
        let a = SemilinearSet::singleton(SparseVector::unit("a".to_string()));
        let a_star = a.star();

        let b = SemilinearSet::singleton(SparseVector::unit("b".to_string()));
        // Use the Kleene operations to compute (a*);b
        let a_star_times_b = a_star.times(b.clone());

        let mut a_b = SparseVector::new();
        a_b.set("a".to_string(), 1);
        a_b.set("b".to_string(), 1);

        let c = SemilinearSet::singleton(SparseVector::unit("c".to_string()));
        let b_times_c = b.times(c);
        let b_times_c_clone = b_times_c.clone(); // Clone before moving

        let a_star_times_b_plus_b_times_c = a_star_times_b.plus(b_times_c);

        // Define the ground truth using the semilinear set constructors
        let ground_truth_a_star_times_b_plus_b_times_c = SemilinearSet::new(vec![
            LinearSet {
                base: SparseVector::unit("b".to_string()),  // Ensure consistency
                periods: vec![],
            },
            LinearSet {
                base: a_b,
                periods: vec![SparseVector::unit("a".to_string())],  // Ensure consistency
            },
            LinearSet {
                base: b_times_c_clone.components[0].base.clone(),
                periods: vec![],  // Ensure consistency
            }
        ]);

        assert_eq!(a_star_times_b_plus_b_times_c, ground_truth_a_star_times_b_plus_b_times_c);
    }


//     #[test]
//     fn test_star_of_a_star_times_b_plus_b_times_c_proper() {
//
//     // Use the Kleene operations to compute a*
//     let a = SemilinearSet::singleton(SparseVector::unit("a".to_string()));
//     let a_star = a.star();
//
//     let b = SemilinearSet::singleton(SparseVector::unit("b".to_string()));
//     // Use the Kleene operations to compute (a*);b
//     let a_star_times_b = a_star.times(b.clone());
//
//     let mut a_b = SparseVector::new();
//     a_b.set("a".to_string(), 1);
//     a_b.set("b".to_string(), 1);
//
//     let mut b_c = SparseVector::new();
//     b_c.set("b".to_string(), 1);
//     b_c.set("c".to_string(), 1);
//
//     // (1,2,0)
//     let mut  a_1_b_2= SparseVector::new();
//     a_1_b_2.set("a".to_string(), 1);
//     a_1_b_2.set("b".to_string(), 2);
//
//     // (1,2,1)
//     let mut  a_1_b_2_c_1= SparseVector::new();
//     a_1_b_2_c_1.set("a".to_string(), 1);
//     a_1_b_2_c_1.set("b".to_string(), 2);
//     a_1_b_2_c_1.set("c".to_string(), 1);
//
//     // (1,3,1)
//     let mut  a_1_b_3_c_1= SparseVector::new();
//     a_1_b_3_c_1.set("a".to_string(), 1);
//     a_1_b_3_c_1.set("b".to_string(), 3);
//     a_1_b_3_c_1.set("c".to_string(), 1);
//
//     // (0,2,1)
//     let mut  b_2_c_1= SparseVector::new();
//     b_2_c_1.set("b".to_string(), 2);
//     b_2_c_1.set("c".to_string(), 1);
//
//     let c = SemilinearSet::singleton(SparseVector::unit("c".to_string()));
//     let b_times_c = b.times(c);
//
//     // (a*);b + (b;c)
//     let a_star_times_b_plus_b_times_c = a_star_times_b.plus(b_times_c);
//
//     // ( (a*);b + (b;c) )*
//     let star_of_a_star_times_b_plus_b_times_c = a_star_times_b_plus_b_times_c.star();
//
//     // Define the ground truth using the semilinear set constructors
//     let ground_truth_star_of_a_star_times_b_plus_b_times_c = SemilinearSet::new(vec![
//         LinearSet { // {(0,0,0);[]}
//             base: SparseVector::new(),
//             periods: vec![],
//         },
//         LinearSet { // {(0,1,0);[(0,1,0)]}
//             base: SparseVector::unit("b".to_string()),
//             periods: vec![SparseVector::unit("b".to_string())],
//         },
//         LinearSet {
//             base: a_b.clone(), // {(1,1,0);[(1,1,0),(1,0,0)]}
//             periods: vec![a_b.clone(), SparseVector::unit("a".to_string())],  // Ensure consistency
//         },
//         LinearSet { // {(0,1,1);[(0,1,1)]}
//             base: b_c.clone(),
//             periods: vec![b_c.clone()],
//         },
//         LinearSet { // {(1,2,0);[(0,1,0),(1,1,0),(1,0,0)]}
//             base: a_1_b_2,
//             periods: vec![SparseVector::unit("b".to_string()),a_b.clone(),SparseVector::unit("a".to_string())],
//         },
//         LinearSet { // {(0,2,1);[(0,1,0),(0,1,1)]}
//             base: b_2_c_1,
//             periods: vec![SparseVector::unit("b".to_string()),b_c.clone()],
//         },
//         LinearSet { // {(1,2,1);[(1,1,0),(1,0,0),(0,1,1)]}
//             base: a_1_b_2_c_1,
//             periods: vec![a_b.clone(),SparseVector::unit("a".to_string()),b_c.clone()],
//         },
//         LinearSet { // {(1,3,1);[(0,1,0),(1,1,0),(1,0,0),(0,1,1)]}
//             base: a_1_b_3_c_1,
//             periods: vec![SparseVector::unit("b".to_string()),a_b.clone(),SparseVector::unit("a".to_string()),b_c.clone()],
//             }
//         ]);
//
//         assert_eq!(star_of_a_star_times_b_plus_b_times_c, ground_truth_star_of_a_star_times_b_plus_b_times_c);
//     }
//
// }


// todo - add code to sort SemilinerSets and LinearSets
// after that --- unmask last test