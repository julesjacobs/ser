/// Based on https://rust-lang.github.io/rust-bindgen/tutorial-4.html
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/isl_bindings.rs"));
}
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::{
    ffi::{CStr, c_uint},
    fmt::{self, Display},
    ptr,
};

#[derive(Debug)]
pub struct PresburgerSet<T> {
    isl_set: *mut bindings::isl_set, // raw pointer to the underlying ISL set
    mapping: Vec<T>,                 // mapping of dimensions to atoms of type T
}

// Ensure the ISL set is freed when PresburgerSet goes out of scope
impl<T> Drop for PresburgerSet<T> {
    fn drop(&mut self) {
        if !self.isl_set.is_null() {
            unsafe { bindings::isl_set_free(self.isl_set) }; // free the ISL set pointer
        }
    }
}

impl<T: Clone> Clone for PresburgerSet<T> {
    fn clone(&self) -> Self {
        let new_ptr = unsafe { bindings::isl_set_copy(self.isl_set) }; // increment refcount or duplicate&#8203;:contentReference[oaicite:1]{index=1}
        PresburgerSet {
            isl_set: new_ptr,
            mapping: self.mapping.clone(),
        }
    }
}

impl<T: Ord + Eq + Clone + Debug + ToString> PresburgerSet<T> {
    pub fn harmonize(&mut self, other: &mut PresburgerSet<T>) {
        // 1. Determine the combined, sorted mapping (same as before)
        let mut combined_atoms: BTreeSet<T> = BTreeSet::new();
        for atom in self.mapping.iter().chain(other.mapping.iter()) {
            combined_atoms.insert(atom.clone());
        }
        let combined_mapping: Vec<T> = combined_atoms.into_iter().collect();

        // 2. Early exit
        if self.mapping == combined_mapping && other.mapping == combined_mapping {
            // Optional space check
            let space1 = unsafe { bindings::isl_set_get_space(self.isl_set) };
            let space2 = unsafe { bindings::isl_set_get_space(other.isl_set) };
            let spaces_equal = unsafe { bindings::isl_space_is_equal(space1, space2) == 1 };
            unsafe {
                bindings::isl_space_free(space1);
                bindings::isl_space_free(space2);
            }
            if spaces_equal { return; }
        }


        let ctx = unsafe { bindings::isl_set_get_ctx(self.isl_set) };

        // 3. Create the target space (positional, no names needed here)
        let n_params = 0; // Adapt if needed
        let n_dims = combined_mapping.len() as c_uint;
        let target_space = unsafe { bindings::isl_space_set_alloc(ctx, n_params, n_dims) };
        // Optionally set tuple name if consistent:
        // target_space = unsafe { bindings::isl_space_set_tuple_name(...) };

        // 4. Call the C wrapper function
        // Temporarily take ownership of pointers to pass to C function which consumes them
        let set1_ptr = std::mem::replace(&mut self.isl_set, ptr::null_mut());
        let set2_ptr = std::mem::replace(&mut other.isl_set, ptr::null_mut());

        let result = unsafe { bindings::rust_harmonize_sets(set1_ptr, set2_ptr, target_space) };

        // 5. Update self and other from the result, handle errors
        if result.error != 0 || result.set1.is_null() || result.set2.is_null() {
            // Restore original pointers if C function failed, to allow safe drop
             self.isl_set = set1_ptr;
             other.isl_set = set2_ptr;
             unsafe { bindings::isl_space_free(target_space); } // Free space if error
            panic!("ISL harmonization failed in C wrapper");
        }

        self.isl_set = result.set1; // Take ownership of returned pointer
        self.mapping = combined_mapping.clone();
        other.isl_set = result.set2; // Take ownership of returned pointer
        other.mapping = combined_mapping;

        // 6. Cleanup
        unsafe {
            bindings::isl_space_free(target_space);
        }
    }
}

impl<T: Clone> PresburgerSet<T> {
    pub fn atom(atom: T) -> Self {
        // Create a 1-dimensional integer space (no parameters, 1 set dim)
        let space = unsafe { bindings::isl_space_set_alloc(bindings::isl_ctx_alloc(), 0, 1) };
        // Start with the universe of that 1D space (all integer points)
        let mut set_ptr = unsafe { bindings::isl_set_universe(space) };
        // Constrain the single dimension (dim 0) to be exactly 1
        set_ptr =
            unsafe { bindings::isl_set_fix_si(set_ptr, bindings::isl_dim_type_isl_dim_set, 0, 1) };
        // (Optionally ensure non-negativity: not needed since it's fixed at 1)
        PresburgerSet {
            isl_set: set_ptr,
            mapping: vec![atom], // one dimension corresponding to the single atom
        }
    }
}

impl<T: Clone> PresburgerSet<T> {
    pub fn universe(atoms: Vec<T>) -> Self {
        let n = atoms.len();
        // Allocate an n-dimensional space for the set (0 parameters, n set dims)
        let space =
            unsafe { bindings::isl_space_set_alloc(bindings::isl_ctx_alloc(), 0, n as c_uint) };
        // Start with the universe set of that space (all integer points in Z^n)
        let mut set_ptr = unsafe { bindings::isl_set_universe(space) };
        // Constrain each dimension to be >= 0 (non-negative)
        for dim_index in 0..n {
            set_ptr = unsafe {
                bindings::isl_set_lower_bound_si(
                    set_ptr,
                    bindings::isl_dim_type_isl_dim_set,
                    dim_index as c_uint,
                    0,
                )
            };
        }
        PresburgerSet {
            isl_set: set_ptr,
            mapping: atoms,
        }
    }
}

impl<T: Eq + Clone + Ord + Debug + ToString> PresburgerSet<T> {
    pub fn union(&self, other: &Self) -> Self {
        // Clone self and other so we can mutate/harmonize freely
        let mut a = self.clone();
        let mut b = other.clone();
        a.harmonize(&mut b);
        // Both a.mapping and b.mapping are now the same (harmonized)
        let unified_mapping = a.mapping.clone();
        // Perform the union operation on the underlying isl_set pointers.
        // We pass ownership of a.isl_set and b.isl_set to isl_set_union (so they will be used and freed inside).
        let result_ptr = unsafe { bindings::isl_set_union(a.isl_set, b.isl_set) };
        // Prevent a and b from freeing the now-consumed pointers in their Drop
        a.isl_set = ptr::null_mut();
        b.isl_set = ptr::null_mut();
        // Wrap the result pointer in a new PresburgerSet
        PresburgerSet {
            isl_set: result_ptr,
            mapping: unified_mapping,
        }
    }

    pub fn intersection(&self, other: &Self) -> Self {
        let mut a = self.clone();
        let mut b = other.clone();
        a.harmonize(&mut b);
        let unified_mapping = a.mapping.clone();
        let result_ptr = unsafe { bindings::isl_set_intersect(a.isl_set, b.isl_set) };
        a.isl_set = ptr::null_mut();
        b.isl_set = ptr::null_mut();
        PresburgerSet {
            isl_set: result_ptr,
            mapping: unified_mapping,
        }
    }

    pub fn difference(&self, other: &Self) -> Self {
        let mut a = self.clone();
        let mut b = other.clone();
        a.harmonize(&mut b);
        let unified_mapping = a.mapping.clone();
        let result_ptr = unsafe { bindings::isl_set_subtract(a.isl_set, b.isl_set) };
        a.isl_set = ptr::null_mut();
        b.isl_set = ptr::null_mut();
        PresburgerSet {
            isl_set: result_ptr,
            mapping: unified_mapping,
        }
    }
}

impl<T: Eq + Clone + Ord + Debug + ToString> PartialEq for PresburgerSet<T> {
    fn eq(&self, other: &Self) -> bool {
        let mut a = self.clone();
        let mut b = other.clone();
        a.harmonize(&mut b);
        // isl_set_is_equal returns isl_bool (1 = true, 0 = false, -1 = error)
        let result_bool = unsafe { bindings::isl_set_is_equal(a.isl_set, b.isl_set) };
        // No need to null out a.isl_set and b.isl_set here, because is_equal does not consume (it uses __isl_keep).
        // We can directly drop a and b, which will free their pointers.
        result_bool == 1 // return true if ISL indicated equality (isl_bool_true)
    }
}

impl<T: Eq + Clone + Ord + Debug + ToString> Eq for PresburgerSet<T> {}

// Implement .is_empty() for PresburgerSet<T>
impl<T: Eq + Clone + Ord + Debug + ToString> PresburgerSet<T> {
    pub fn is_empty(&self) -> bool {
        unsafe { bindings::isl_set_is_empty(self.isl_set) == 1 }
    }
}

// Implementing display for PresburgerSet<T> using ISL's to_str function
impl<T: Display> Display for PresburgerSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str: *mut i8 = unsafe { bindings::isl_set_to_str(self.isl_set) };
        let mapping_str = self
            .mapping
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            f,
            "{} (mapping: {})",
            unsafe { CStr::from_ptr(str).to_string_lossy() },
            mapping_str
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presburger() {
        let a = PresburgerSet::atom('a');
        let b = PresburgerSet::atom('b');
        let ab = a.union(&b);
        println!("ab: {:}", ab);
        let universe = PresburgerSet::universe(vec!['a', 'b', 'c']);
        println!("universe: {:}", universe);
        let ab_union_universe = ab.union(&universe);
        println!("ab_union_universe: {:}", ab_union_universe);
        let universe_complement = universe.difference(&ab);
        println!("universe_complement: {:}", universe_complement);
        let universe2 = PresburgerSet::universe(vec!['c', 'b', 'a']);
        println!("universe2: {:}", universe2);
    }

    #[test]
    fn test_universe_reorder() {
        let mut u1 = PresburgerSet::universe(vec!['a', 'b']);
        let mut u2 = PresburgerSet::universe(vec!['b', 'a']);
        // print both
        println!("u1: {:}", u1);
        println!("u2: {:}", u2);
        u1.harmonize(&mut u2);
        // print both again
        println!("u1: {:}", u1);
        println!("u2: {:}", u2);

        u1 = PresburgerSet::universe(vec!['a', 'b', 'c']);
        u2 = PresburgerSet::universe(vec!['c', 'b']);
        u1.harmonize(&mut u2);
        println!("u1: {:}", u1);
        println!("u2: {:}", u2);
    }

    #[test]
    fn test_union_commutative() {
        let a = PresburgerSet::atom('a');
        let b = PresburgerSet::atom('b');
        let a_union_b = a.clone().union(&b);
        let b_union_a = b.clone().union(&a);
        assert_eq!(a_union_b, b_union_a);
    }

    #[test]
    fn test_union_associative() {
        let a = PresburgerSet::atom('a');
        let b = PresburgerSet::atom('b');
        let c = PresburgerSet::atom('c');
        let ab_union_c = a.clone().union(&b).union(&c);
        let a_union_bc = a.union(&b.union(&c));
        assert_eq!(ab_union_c, a_union_bc);
    }

    #[test]
    fn test_intersection_distributes_over_union() {
        let a = PresburgerSet::atom('a');
        let b = PresburgerSet::atom('b');
        let d = PresburgerSet::atom('d');
        let a_union_b = a.clone().union(&b);
        let a_intersect_d = a.clone().intersection(&d);
        let b_intersect_d = b.clone().intersection(&d);
        let distribute_left = a_union_b.intersection(&d);
        let distribute_right = a_intersect_d.union(&b_intersect_d);
        assert_eq!(distribute_left, distribute_right);
    }

    #[test]
    fn test_universe_difference_empty() {
        let universe = PresburgerSet::universe(vec!['a', 'b', 'c']);
        let empty = universe.clone().difference(&universe);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_union_with_universe() {
        let universe = PresburgerSet::universe(vec!['a', 'b', 'c']);
        let universe2 = PresburgerSet::universe(vec!['c', 'b', 'a']);
        let union_universe = universe.union(&universe2);
        assert_eq!(union_universe, universe);
    }

    #[test]
    fn test_demorgans_laws() {
        let universe = PresburgerSet::universe(vec!['a', 'b', 'c']);
        let a = PresburgerSet::atom('a');
        let b = PresburgerSet::atom('b');

        // Test De Morgan's Law: !(A ∪ B) = !A ∩ !B
        let a_union_b = a.clone().union(&b);
        let not_a_union_b = universe.clone().difference(&a_union_b);
        
        let not_a = universe.clone().difference(&a);
        let not_b = universe.clone().difference(&b); 
        let not_a_intersect_not_b = not_a.intersection(&not_b);

        assert_eq!(not_a_union_b, not_a_intersect_not_b);

        // Test De Morgan's Law: !(A ∩ B) = !A ∪ !B
        let a_intersect_b = a.clone().intersection(&b);
        let not_a_intersect_b = universe.clone().difference(&a_intersect_b);

        let not_a = universe.clone().difference(&a);
        let not_b = universe.clone().difference(&b);
        let not_a_union_not_b = not_a.union(&not_b);

        assert_eq!(not_a_intersect_b, not_a_union_not_b);
    }
}

// The types below are to convert an ISL to a Rust type that can be consumed by Rust code

/// Represents an existentially quantified conjunction of linear constraints
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantifiedSet<T> {
    // The variables T are from the original set, and the variables usize are the existential variables
    // The existential quantification is over *natural numbers* not integers
    // Note: the existential quantification is at the top-level: it's an existential of conjunctions,
    // not a conjunction of existentials.
    constraints: Vec<Constraint<Variable<T>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variable<T> {
    Var(T),
    Existential(usize),
}

// Implement pretty printing for Variable<T>: Var(T) is printed as V{T} and Existential(n) is printed as E{n} (without the curlies)



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

// Add pretty printing for Constraint and QuantifiedSet



// implement the conversion from PresburgerSet<T> to Vec<QuantifiedSet<T>>
// implement the inverse conversion as well


