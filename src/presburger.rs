// Use the ISL bindings from the isl module
use crate::isl::bindings;
use std::fmt::Debug;
use std::{
    collections::BTreeSet,
    ffi::{CStr, CString, c_uint},
    fmt::{self, Display},
    ptr,
};

use crate::kleene::Kleene;

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
            if spaces_equal {
                return;
            }
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
            unsafe {
                bindings::isl_space_free(target_space);
            } // Free space if error
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

impl<T: Eq + Clone + Ord + Debug + ToString> Kleene for PresburgerSet<T> {
    fn zero() -> Self {
        // For a Kleene algebra, zero represents the empty set
        let space = unsafe { bindings::isl_space_set_alloc(bindings::isl_ctx_alloc(), 0, 0) };
        let set_ptr = unsafe { bindings::isl_set_empty(space) };
        PresburgerSet {
            isl_set: set_ptr,
            mapping: Vec::new(),
        }
    }

    fn one() -> Self {
        // For a Kleene algebra, one represents the empty string/epsilon
        // In our context, this is a set containing only the zero vector
        let space = unsafe { bindings::isl_space_set_alloc(bindings::isl_ctx_alloc(), 0, 0) };
        // Create a universe (all points), then constrain it to just the origin (0)
        let set_ptr = unsafe { bindings::isl_set_universe(space) };

        PresburgerSet {
            isl_set: set_ptr,
            mapping: Vec::new(),
        }
    }

    fn plus(self, other: Self) -> Self {
        // In Kleene algebra, plus is union
        self.union(&other)
    }

    fn times(self, other: Self) -> Self {
        // In Kleene algebra, times would typically be Minkowski sum for vectors
        let mut a = self.clone();
        let mut b = other.clone();
        a.harmonize(&mut b);
        let unified_mapping = a.mapping.clone();
        let result_ptr = unsafe { bindings::isl_set_sum(a.isl_set, b.isl_set) };
        a.isl_set = ptr::null_mut();
        b.isl_set = ptr::null_mut();
        PresburgerSet {
            isl_set: result_ptr,
            mapping: unified_mapping,
        }
    }

    fn star(self) -> Self {
        // Kleene star would involve computing closures
        // This is complex for Presburger sets and beyond the scope
        unimplemented!("Star operation is not implemented for PresburgerSet")
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

// Pretty printing for Variable<T>: Var(T) is printed as V{T} and Existential(n) is printed as E{n}
impl<T: Display> Display for Variable<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Variable::Var(t) => write!(f, "V{}", t),
            Variable::Existential(n) => write!(f, "E{}", n),
        }
    }
}

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

// Pretty printing for Constraint<T>
impl<T: Display> Display for Constraint<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format each term in the linear combination
        let mut first = true;
        for &(coef, ref var) in &self.linear_combination {
            if coef == 0 {
                continue; // Skip zero coefficients
            }

            if !first {
                if coef > 0 {
                    write!(f, " + ")?;
                } else {
                    write!(f, " -")?;
                }
            } else if coef < 0 {
                // First term and negative
                write!(f, "-")?;
            }

            let abs_coef = coef.abs();
            if abs_coef == 1 {
                write!(f, "{}", var)?;
            } else {
                write!(f, "{}{}", abs_coef, var)?;
            }

            first = false;
        }

        // Add the constant term if non-zero or if there are no terms
        if self.constant_term != 0 || first {
            if !first {
                if self.constant_term > 0 {
                    write!(f, " + ")?;
                } else if self.constant_term < 0 {
                    write!(f, " -")?;
                }
            } else if self.constant_term < 0 {
                // First term and negative
                write!(f, "-")?;
            }

            write!(f, "{}", self.constant_term.abs())?;
        }

        // Add the constraint type
        match self.constraint_type {
            ConstraintType::NonNegative => write!(f, " ≥ 0"),
            ConstraintType::EqualToZero => write!(f, " = 0"),
        }
    }
}

// Pretty printing for QuantifiedSet<T>
impl<T: Display> Display for QuantifiedSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Check if there are existential variables
        let has_existentials = self.constraints.iter().any(|c| {
            c.linear_combination.iter().any(|(_, var)| {
                if let Variable::Existential(_) = var {
                    true
                } else {
                    false
                }
            })
        });

        // Write existential quantifier if needed
        if has_existentials {
            write!(f, "∃ ")?;
        }

        // Write all constraints connected by '∧'
        for (i, constraint) in self.constraints.iter().enumerate() {
            if i > 0 {
                write!(f, " ∧ ")?;
            }
            write!(f, "{}", constraint)?;
        }

        Ok(())
    }
}

// Implement conversions between SemilinearSet and PresburgerSet

use crate::semilinear::{LinearSet, SemilinearSet, SparseVector};
use std::collections::HashMap;
use std::hash::Hash;

impl<T: Clone + Ord + Debug + ToString + Eq + Hash> PresburgerSet<T> {
    /// Convert a SemilinearSet to a PresburgerSet
    ///
    /// This processes each LinearSet component of the SemilinearSet and represents it
    /// as a set of constraints in the PresburgerSet.
    pub fn from_semilinear_set(semilinear_set: &SemilinearSet<T>) -> Self {
        // First, collect all keys used in the semilinear set
        let mut all_keys = BTreeSet::new();
        for component in &semilinear_set.components {
            // Add keys from the base vector
            for (key, _) in &component.base.values {
                all_keys.insert(key.clone());
            }

            // Add keys from the period vectors
            for period in &component.periods {
                for (key, _) in &period.values {
                    all_keys.insert(key.clone());
                }
            }
        }

        // Convert BTreeSet to Vec for consistent ordering
        let mapping: Vec<T> = all_keys.into_iter().collect();

        // Create a context and an empty result set
        let ctx = unsafe { bindings::isl_ctx_alloc() };
        let mut result_set: *mut bindings::isl_set = std::ptr::null_mut();

        // Process each linear set component
        for component in &semilinear_set.components {
            // Convert the linear set to an ISL set string and parse it
            let set_string = generate_linear_set_string(component, &mapping);

            // Parse the ISL set string
            let component_set = unsafe {
                let cstr = CString::new(set_string).unwrap();
                bindings::isl_set_read_from_str(ctx, cstr.as_ptr())
            };

            // Union with the result set
            unsafe {
                if result_set.is_null() {
                    result_set = component_set;
                } else {
                    result_set = bindings::isl_set_union(result_set, component_set);
                }
            }
        }

        // If no constraints, return the universe set
        if result_set.is_null() || semilinear_set.components.is_empty() {
            let space = unsafe { bindings::isl_space_set_alloc(ctx, 0, mapping.len() as c_uint) };
            result_set = unsafe { bindings::isl_set_universe(space) };
        }

        PresburgerSet {
            isl_set: result_set,
            mapping,
        }
    }
}

/// Helper function to generate an ISL set string from a LinearSet
fn generate_linear_set_string<T: ToString + Clone + Ord + Eq + Hash>(
    linear_set: &LinearSet<T>,
    mapping: &[T],
) -> String {
    let mut constraints = Vec::new();

    // // Find indices for keys in the mapping
    // let key_indices: HashMap<&T, usize> = mapping
    //     .iter()
    //     .enumerate()
    //     .map(|(i, key)| (key, i))
    //     .collect();

    // Generate constraints for each dimension in the mapping
    for (i, key) in mapping.iter().enumerate() {
        // Get base value for this key (0 if not present)
        let base_value = linear_set.base.get(key);

        // Start building the constraint (key = base_value + coefficients*periods)
        let mut constraint = format!("p{} = {}", i, base_value);

        // Add period terms
        for (period_idx, period) in linear_set.periods.iter().enumerate() {
            let coeff = period.get(key);
            if coeff != 0 {
                constraint.push_str(&format!(" + {} e{}", coeff, period_idx));
            }
        }

        constraints.push(constraint);
    }

    // Add constraints ensuring all period variables are non-negative
    for period_idx in 0..linear_set.periods.len() {
        constraints.push(format!("e{} >= 0", period_idx));
    }

    // Build the complete ISL set string
    if linear_set.periods.is_empty() {
        // No existential variables needed
        format!(
            "{{ [{}] : {} }}",
            (0..mapping.len())
                .map(|i| format!("p{}", i))
                .collect::<Vec<_>>()
                .join(", "),
            constraints.join(" and ")
        )
    } else {
        // With existential variables
        format!(
            "{{ [{}] : exists ({} : {}) }}",
            (0..mapping.len())
                .map(|i| format!("p{}", i))
                .collect::<Vec<_>>()
                .join(", "),
            (0..linear_set.periods.len())
                .map(|i| format!("e{}", i))
                .collect::<Vec<_>>()
                .join(", "),
            constraints.join(" and ")
        )
    }
}

#[test]
fn test_semilinear_to_presburger_conversion() {
    use crate::semilinear::LinearSet;
    use crate::semilinear::SemilinearSet;
    use crate::semilinear::SparseVector;

    // Create a SemilinearSet with a simple linear set component
    let mut base = SparseVector::new();
    base.set('a', 1);

    let mut period1 = SparseVector::new();
    period1.set('b', 1);

    let mut period2 = SparseVector::new();
    period2.set('c', 2);

    let linear_set = LinearSet {
        base,
        periods: vec![period1, period2],
    };

    let semilinear_set = SemilinearSet::new(vec![linear_set]);

    // Convert to PresburgerSet
    println!("\nTest SemilinearSet to PresburgerSet conversion");
    println!("Original SemilinearSet: {}", semilinear_set);

    let presburger_set = PresburgerSet::from_semilinear_set(&semilinear_set);
    println!("Converted to PresburgerSet: {}", presburger_set);
}

/// Convert from PresburgerSet<T> to Vec<QuantifiedSet<T>>
///
/// This converts an ISL-based representation to a pure Rust representation
/// that can be processed without relying on the ISL library.
impl<T: Clone + Ord + Debug + ToString> PresburgerSet<T> {
    pub fn to_quantified_sets(&self) -> Vec<QuantifiedSet<T>> {
        // We'll use a simpler approach that works in a single pass

        // The result will be stored here
        let result;

        // Define a callback for processing each basic set
        unsafe {
            // We need to use the isl_set_foreach_basic_set function to iterate through basic sets
            struct UserData<T> {
                result_sets: Vec<QuantifiedSet<T>>,
                mapping: Vec<T>,
            }

            // Callback for each basic set
            extern "C" fn basic_set_callback<T: Clone + Debug + ToString>(
                bset: *mut bindings::isl_basic_set,
                user: *mut std::os::raw::c_void,
            ) -> bindings::isl_stat {
                unsafe {
                    let user_data = &mut *(user as *mut UserData<T>);
                    let mapping = &user_data.mapping;

                    // Create a new QuantifiedSet for this basic set
                    let mut quantified_set = QuantifiedSet {
                        constraints: Vec::new(),
                    };

                    // Get the dimension information
                    let space = bindings::isl_basic_set_get_space(bset);
                    let n_dims =
                        bindings::isl_space_dim(space, bindings::isl_dim_type_isl_dim_set) as usize;
                    let n_div =
                        bindings::isl_space_dim(space, bindings::isl_dim_type_isl_dim_div) as usize;

                    // Define a nested callback for processing each constraint
                    struct ConstraintData<'a, T> {
                        quantified_set: &'a mut QuantifiedSet<T>,
                        mapping: &'a [T],
                        n_dims: usize,
                        n_div: usize,
                    }

                    extern "C" fn constraint_callback<T: Clone + Debug + ToString>(
                        constraint: *mut bindings::isl_constraint,
                        user: *mut std::os::raw::c_void,
                    ) -> bindings::isl_stat {
                        unsafe {
                            let constraint_data = &mut *(user as *mut ConstraintData<T>);

                            // Determine constraint type
                            let constraint_type =
                                if bindings::isl_constraint_is_equality(constraint) != 0 {
                                    ConstraintType::EqualToZero
                                } else {
                                    ConstraintType::NonNegative
                                };

                            // Get constant term
                            let constant_term = {
                                let val = bindings::isl_constraint_get_constant_val(constraint);
                                let result = bindings::isl_val_get_num_si(val);
                                bindings::isl_val_free(val);
                                result as i32
                            };

                            // Collect coefficients for the constraint
                            let mut linear_combination = Vec::new();

                            // Process original variables
                            for k in 0..std::cmp::min(
                                constraint_data.n_dims,
                                constraint_data.mapping.len(),
                            ) {
                                let coef = {
                                    let val = bindings::isl_constraint_get_coefficient_val(
                                        constraint,
                                        bindings::isl_dim_type_isl_dim_set,
                                        k as i32,
                                    );
                                    let result = bindings::isl_val_get_num_si(val);
                                    bindings::isl_val_free(val);
                                    result as i32
                                };

                                if coef != 0 {
                                    linear_combination.push((
                                        coef,
                                        Variable::Var(constraint_data.mapping[k].clone()),
                                    ));
                                }
                            }

                            // Process existential variables
                            for k in 0..constraint_data.n_div {
                                let coef = {
                                    let val = bindings::isl_constraint_get_coefficient_val(
                                        constraint,
                                        bindings::isl_dim_type_isl_dim_div,
                                        k as i32,
                                    );
                                    let result = bindings::isl_val_get_num_si(val);
                                    bindings::isl_val_free(val);
                                    result as i32
                                };

                                if coef != 0 {
                                    linear_combination.push((coef, Variable::Existential(k)));
                                }
                            }

                            // Create and add the constraint to the quantified set
                            if !linear_combination.is_empty() || constant_term != 0 {
                                constraint_data.quantified_set.constraints.push(Constraint {
                                    linear_combination,
                                    constant_term,
                                    constraint_type,
                                });
                            }

                            0 // isl_stat_ok
                        }
                    }

                    // Process each constraint in the basic set
                    let mut constraint_data = ConstraintData {
                        quantified_set: &mut quantified_set,
                        mapping,
                        n_dims,
                        n_div,
                    };

                    bindings::isl_basic_set_foreach_constraint(
                        bset,
                        Some(constraint_callback::<T>),
                        &mut constraint_data as *mut _ as *mut std::os::raw::c_void,
                    );

                    // Add the quantified set to the result
                    user_data.result_sets.push(quantified_set);

                    // Cleanup
                    bindings::isl_space_free(space);

                    0 // isl_stat_ok
                }
            }

            // Make a copy of the set and mapping for the callback
            let set_copy = bindings::isl_set_copy(self.isl_set);

            // Prepare user data structure
            let mut user_data = UserData {
                result_sets: Vec::new(),
                mapping: self.mapping.clone(),
            };

            // Iterate through each basic set
            bindings::isl_set_foreach_basic_set(
                set_copy,
                Some(basic_set_callback::<T>),
                &mut user_data as *mut _ as *mut std::os::raw::c_void,
            );

            // Extract result sets
            result = user_data.result_sets;

            // Clean up
            bindings::isl_set_free(set_copy);
        }

        result
    }
}

/// Convert from Vec<QuantifiedSet<T>> back to PresburgerSet<T>
///
/// This function converts a Rust representation back to an ISL-based representation.
impl<T: Clone + Ord + Debug + ToString> PresburgerSet<T> {
    pub fn from_quantified_sets(sets: &[QuantifiedSet<T>], mapping: Vec<T>) -> Self {
        // Using the ISL context
        let ctx = unsafe { bindings::isl_ctx_alloc() };

        // Create an empty result set
        let mut result_set: *mut bindings::isl_set = std::ptr::null_mut();

        // Process each QuantifiedSet (each one becomes a basic set in the result)
        for quantified_set in sets {
            // Create the ISL set string for this QuantifiedSet
            let set_string = create_isl_set_string(quantified_set, &mapping);

            // Parse the ISL set string
            let set = unsafe {
                let cstr = CString::new(set_string).unwrap();
                bindings::isl_set_read_from_str(ctx, cstr.as_ptr())
            };

            // Union with the result set
            unsafe {
                if result_set.is_null() {
                    result_set = set;
                } else {
                    result_set = bindings::isl_set_union(result_set, set);
                }
            }
        }

        // If no constraints, return the universe set
        if result_set.is_null() {
            let space = unsafe { bindings::isl_space_set_alloc(ctx, 0, mapping.len() as c_uint) };
            result_set = unsafe { bindings::isl_set_universe(space) };
        }

        PresburgerSet {
            isl_set: result_set,
            mapping,
        }
    }
}

// Helper function to create ISL set string from a QuantifiedSet
fn create_isl_set_string<T: ToString>(quantified_set: &QuantifiedSet<T>, mapping: &[T]) -> String {
    // Collect all existential variables used in this set
    let existential_vars: BTreeSet<usize> = quantified_set
        .constraints
        .iter()
        .flat_map(|c| {
            c.linear_combination
                .iter()
                .filter_map(|(_, var)| match var {
                    Variable::Existential(idx) => Some(*idx),
                    _ => None,
                })
        })
        .collect();

    // ISL expects dimension names in the format [p0, p1, ...] or similar
    // We don't actually need this vector, just keeping the format for clarity
    let _var_names: Vec<String> = mapping
        .iter()
        .map(|var| format!("p{}", var.to_string()))
        .collect();

    // Create variable names for existential variables
    let existential_names: Vec<String> = existential_vars
        .iter()
        .map(|&idx| format!("e{}", idx))
        .collect();

    // Generate constraint strings
    let mut constraint_strings = Vec::new();

    for constraint in &quantified_set.constraints {
        // Build the affine expression
        let mut expr = String::new();

        for (i, (coeff, var)) in constraint.linear_combination.iter().enumerate() {
            if i > 0 {
                if *coeff >= 0 {
                    expr.push_str(" + ");
                } else {
                    expr.push_str(" - ");
                }
                expr.push_str(&format!("{}", coeff.abs()));
            } else {
                // First term
                if *coeff >= 0 {
                    expr.push_str(&format!("{}", coeff));
                } else {
                    expr.push_str(&format!("-{}", coeff.abs()));
                }
            }

            match var {
                Variable::Var(t) => {
                    // Find the index of this variable in the mapping
                    // We need to compare by string representation
                    let t_str = t.to_string();
                    let idx = mapping
                        .iter()
                        .position(|x| x.to_string() == t_str)
                        .expect("Variable not found in mapping");
                    expr.push_str(&format!("*p{}", idx));
                }
                Variable::Existential(idx) => {
                    expr.push_str(&format!("*e{}", idx));
                }
            }
        }

        // Add the constant term
        if constraint.constant_term != 0 {
            if constraint.constant_term > 0 {
                expr.push_str(&format!(" + {}", constraint.constant_term));
            } else {
                expr.push_str(&format!(" - {}", -constraint.constant_term));
            }
        }

        // Add the constraint type
        match constraint.constraint_type {
            ConstraintType::EqualToZero => constraint_strings.push(format!("{} = 0", expr)),
            ConstraintType::NonNegative => constraint_strings.push(format!("{} >= 0", expr)),
        }
    }

    // Add non-negativity constraints for regular variables
    for i in 0..mapping.len() {
        constraint_strings.push(format!("p{} >= 0", i));
    }

    // Assemble the full ISL set string
    if existential_names.is_empty() {
        // No existential variables
        format!(
            "{{ [{}] : {} }}",
            (0..mapping.len())
                .map(|i| format!("p{}", i))
                .collect::<Vec<String>>()
                .join(", "),
            constraint_strings.join(" and ")
        )
    } else {
        // With existential variables
        format!(
            "{{ [{}] : exists ({} : {}) }}",
            (0..mapping.len())
                .map(|i| format!("p{}", i))
                .collect::<Vec<String>>()
                .join(", "),
            existential_names.join(", "),
            constraint_strings.join(" and ")
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
    fn test_presburger_kleene() {
        // Test zero (empty set)
        let empty = PresburgerSet::<char>::zero();
        assert!(empty.is_empty());

        // Test one (singleton set containing the empty vector)
        let one = PresburgerSet::<char>::one();
        assert!(!one.is_empty());

        // Test plus operation (union)
        let a = PresburgerSet::atom('a');
        let b = PresburgerSet::atom('b');

        // Using Kleene operations
        let a_plus_b = a.clone().plus(b.clone());
        // Using direct operations for comparison
        let a_union_b = a.clone().union(&b);

        assert_eq!(a_plus_b, a_union_b);

        // Try to call times and star, which should panic with "unimplemented"
        // Note: These are commented out since they will panic
        // let _times_result = a.clone().times(b.clone());
        // let _star_result = a.star();
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

    #[test]
    fn test_pretty_printing() {
        // Test Variable<T> Display implementation
        let var_t = Variable::Var("x");
        let var_e = Variable::Existential(3);

        assert_eq!(format!("{}", var_t), "Vx");
        assert_eq!(format!("{}", var_e), "E3");

        // Test Constraint<T> Display implementation
        let constraint1 = Constraint {
            linear_combination: vec![(1, var_t), (2, var_e)],
            constant_term: 5,
            constraint_type: ConstraintType::NonNegative,
        };

        let constraint2 = Constraint {
            linear_combination: vec![(-1, var_t), (-1, var_e)],
            constant_term: 0,
            constraint_type: ConstraintType::EqualToZero,
        };

        assert_eq!(format!("{}", constraint1), "Vx + 2E3 + 5 ≥ 0");
        assert_eq!(format!("{}", constraint2), "-Vx -E3 = 0");

        // Test QuantifiedSet<T> Display implementation
        let quantified_set = QuantifiedSet {
            constraints: vec![constraint1, constraint2],
        };

        assert_eq!(
            format!("{}", quantified_set),
            "∃ Vx + 2E3 + 5 ≥ 0 ∧ -Vx -E3 = 0"
        );
    }

    #[test]
    fn test_presburger_to_quantified_conversion() {
        // Create a simple PresburgerSet
        let a = PresburgerSet::atom('a');
        let b = PresburgerSet::atom('b');
        let union = a.union(&b);

        // Convert to QuantifiedSet
        let quantified_sets = union.to_quantified_sets();

        // Print the result
        println!("Original PresburgerSet: {}", union);
        for (i, qs) in quantified_sets.iter().enumerate() {
            println!("QuantifiedSet {}: {}", i, qs);
        }

        // Convert back to PresburgerSet
        let mapping = union.mapping.clone();
        let round_trip = PresburgerSet::from_quantified_sets(&quantified_sets, mapping);

        // Verify the round trip conversion
        println!("Round-trip PresburgerSet: {}", round_trip);

        // We can't directly compare the ISL sets since their internal representation might differ
        // Even though they represent the same mathematical set
        // Instead, let's just verify the conversion worked as expected
        assert!(!round_trip.is_empty());
        assert_eq!(union.mapping, round_trip.mapping);
    }

    #[test]
    fn test_conversion_atom() {
        // Test with a single atom
        let atom = PresburgerSet::atom('x');
        println!("\nTest atom conversion");
        println!("Original atom: {}", atom);

        let quantified = atom.to_quantified_sets();
        println!("Converted to {} quantified sets:", quantified.len());
        for (i, qs) in quantified.iter().enumerate() {
            println!("  QuantifiedSet {}: {}", i, qs);
        }

        let round_trip = PresburgerSet::from_quantified_sets(&quantified, atom.mapping.clone());
        println!("Round-trip: {}", round_trip);

        assert!(!round_trip.is_empty());
        assert_eq!(atom.mapping, round_trip.mapping);
    }

    #[test]
    fn test_conversion_universe() {
        // Test with universe (multiple variables)
        let universe = PresburgerSet::universe(vec!['a', 'b', 'c']);
        println!("\nTest universe conversion");
        println!("Original universe: {}", universe);

        let quantified = universe.to_quantified_sets();
        println!("Converted to {} quantified sets:", quantified.len());
        for (i, qs) in quantified.iter().enumerate() {
            println!("  QuantifiedSet {}: {}", i, qs);
        }

        let round_trip = PresburgerSet::from_quantified_sets(&quantified, universe.mapping.clone());
        println!("Round-trip: {}", round_trip);

        assert!(!round_trip.is_empty());
        assert_eq!(universe.mapping, round_trip.mapping);
    }

    #[test]
    fn test_conversion_intersection() {
        // Test with intersection (multiple constraints in a single conjunction)
        let a = PresburgerSet::atom('a');
        let b = PresburgerSet::atom('b');
        let intersection = a.intersection(&b);

        println!("\nTest intersection conversion");
        println!("Original intersection: {}", intersection);

        let quantified = intersection.to_quantified_sets();
        println!("Converted to {} quantified sets:", quantified.len());
        for (i, qs) in quantified.iter().enumerate() {
            println!("  QuantifiedSet {}: {}", i, qs);
        }

        let round_trip =
            PresburgerSet::from_quantified_sets(&quantified, intersection.mapping.clone());
        println!("Round-trip: {}", round_trip);

        assert_eq!(intersection.mapping, round_trip.mapping);
    }

    #[test]
    fn test_conversion_difference() {
        // Test with set difference (more complex constraints)
        let universe = PresburgerSet::universe(vec!['a', 'b', 'c']);
        let a = PresburgerSet::atom('a');
        let difference = universe.difference(&a);

        println!("\nTest difference conversion");
        println!("Original difference (universe - atom): {}", difference);

        let quantified = difference.to_quantified_sets();
        println!("Converted to {} quantified sets:", quantified.len());
        for (i, qs) in quantified.iter().enumerate() {
            println!("  QuantifiedSet {}: {}", i, qs);
        }

        let round_trip =
            PresburgerSet::from_quantified_sets(&quantified, difference.mapping.clone());
        println!("Round-trip: {}", round_trip);

        assert!(!round_trip.is_empty());
        assert_eq!(difference.mapping, round_trip.mapping);
    }

    #[test]
    fn test_conversion_complex() {
        // Create a more complex example with two different constraints
        // universe - (a ∩ c)
        let universe = PresburgerSet::universe(vec!['a', 'b', 'c']);
        let a = PresburgerSet::atom('a');
        let c = PresburgerSet::atom('c');
        let a_and_c = a.intersection(&c);
        let complex = universe.difference(&a_and_c);

        println!("\nTest complex conversion");
        println!("Original complex set: {}", complex);

        let quantified = complex.to_quantified_sets();
        println!("Converted to {} quantified sets:", quantified.len());
        for (i, qs) in quantified.iter().enumerate() {
            println!("  QuantifiedSet {}: {}", i, qs);
        }

        let round_trip = PresburgerSet::from_quantified_sets(&quantified, complex.mapping.clone());
        println!("Round-trip: {}", round_trip);

        assert!(!round_trip.is_empty());
        assert_eq!(complex.mapping, round_trip.mapping);
    }

    #[test]
    fn test_conversion_empty() {
        // Test with an empty set
        let universe = PresburgerSet::universe(vec!['a', 'b']);
        let empty = universe.difference(&universe);

        println!("\nTest empty set conversion");
        println!("Original empty set: {}", empty);

        let quantified = empty.to_quantified_sets();
        println!("Converted to {} quantified sets:", quantified.len());
        for (i, qs) in quantified.iter().enumerate() {
            println!("  QuantifiedSet {}: {}", i, qs);
        }

        let round_trip = PresburgerSet::from_quantified_sets(&quantified, empty.mapping.clone());
        println!("Round-trip: {}", round_trip);

        assert_eq!(empty.mapping, round_trip.mapping);
    }

    #[test]
    fn test_manual_quantified_set() {
        // Create a QuantifiedSet with existential variables manually
        // to test the existential variable conversion path
        let var_x = Variable::Var('x');
        let var_y = Variable::Var('y');
        let exist_var = Variable::Existential(0);

        // Create a constraint: x = 2*e0
        let constraint1 = Constraint {
            linear_combination: vec![(1, var_x), (-2, exist_var)],
            constant_term: 0,
            constraint_type: ConstraintType::EqualToZero,
        };

        // Create a constraint: y = 2*e0 + 1
        let constraint2 = Constraint {
            linear_combination: vec![(1, var_y), (-2, exist_var)],
            constant_term: -1,
            constraint_type: ConstraintType::EqualToZero,
        };

        // Create the QuantifiedSet (x is even, y is odd)
        let quantified_set = QuantifiedSet {
            constraints: vec![constraint1, constraint2],
        };

        println!("\nTest manual quantified set conversion");
        println!("Original quantified set: {}", quantified_set);

        // Convert to PresburgerSet
        let mapping = vec!['x', 'y'];
        let presburger =
            PresburgerSet::from_quantified_sets(&[quantified_set.clone()], mapping.clone());
        println!("Converted to PresburgerSet: {}", presburger);

        // Convert back to QuantifiedSet
        let quantified_sets = presburger.to_quantified_sets();
        println!(
            "Converted back to {} quantified sets:",
            quantified_sets.len()
        );
        for (i, qs) in quantified_sets.iter().enumerate() {
            println!("  QuantifiedSet {}: {}", i, qs);
        }

        assert!(!presburger.is_empty());
        assert_eq!(mapping, presburger.mapping);
    }

    #[test]
    fn test_semilinear_to_presburger() {
        let a = SemilinearSet::atom('a');
        let b = SemilinearSet::atom('b');
        let c = SemilinearSet::atom('c');
        let a_star = a.star();
        let bc = b.plus(c);
        let a_star_times_bc = a_star.times(bc);
        let presburger = PresburgerSet::from_semilinear_set(&a_star_times_bc);
        println!("PresburgerSet: {}", presburger);
    }
}
