// Semi-linear sets

use std::collections::HashSet;

/// A vector in d-dimensional nonnegative integer space.
type Vector = Vec<usize>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LinearSet {
    base: Vector,            // u0: the base vector
    periods: Vec<Vector>,    // [u1, u2, ..., um]: list of period generator vectors
}

#[derive(Debug, Clone)]
struct SemilinearSet {
    dimension: usize,
    components: Vec<LinearSet>,  // finite list of linear sets whose union defines the set
}

impl SemilinearSet {
    /// Create a new semilinear set from a list of LinearSet components.
    fn new(dimension: usize, components: Vec<LinearSet>) -> Self {
        // Assert that the dimension is the same for all components
        debug_assert!(components.iter().all(|lin| lin.base.len() == dimension && lin.periods.iter().all(|p| p.len() == dimension)));

        let mut new_components = HashSet::new();
        for lin in components {
            // filter out duplicate period vectors from lin
            let mut new_periods = HashSet::new();
            for p in lin.periods {
                new_periods.insert(p);
            }
            new_components.insert(LinearSet { base: lin.base, periods: new_periods.into_iter().collect() });
        }
        SemilinearSet { dimension, components: new_components.into_iter().collect() }
    }

    /// Check if the semilinear set is empty.
    fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Create a semilinear set containing a single vector (an atomic singleton).
    fn singleton(vector: Vector) -> Self {
        SemilinearSet {
            dimension: vector.len(),
            components: vec![ LinearSet { base: vector, periods: vec![] } ],
        }
    }

    /// Singleton containing the zero vector.
    fn zero(dimension: usize) -> Self {
        SemilinearSet::singleton(vec![0; dimension])
    }

    /// The empty semilinear set in dimension d (contains no vectors).
    fn empty(dimension: usize) -> Self {
        SemilinearSet { dimension, components: Vec::new() }
    }

    /// The universe (all of N^d) as a semilinear set.
    fn universe(dimension: usize) -> Self {
        // Universe = linear set with base = [0,0,...,0], periods = unit vectors e1..ed
        let base = vec![0; dimension];
        let mut periods = Vec::with_capacity(dimension);
        for i in 0..dimension {
            let mut unit = vec![0; dimension];
            unit[i] = 1;
            periods.push(unit);
        }
        SemilinearSet::new(dimension, vec![ LinearSet { base, periods } ])
    }

    /// Return the union of this set with another semilinear set.
    fn union(&self, other: &SemilinearSet) -> SemilinearSet {
        assert_eq!(self.dimension, other.dimension, "Dimension mismatch in union");
        // Clone components of both and combine
        let mut new_components = Vec::with_capacity(self.components.len() + other.components.len());
        new_components.extend(self.components.iter().cloned());
        new_components.extend(other.components.iter().cloned());
        // (TODO) we could attempt to simplify or merge components here.
        SemilinearSet::new(self.dimension, new_components)
    }

    /// Sequential composition (a.k.a. Minkowski sum) of two semilinear sets.
    fn add(&self, other: &SemilinearSet) -> SemilinearSet {
        assert_eq!(self.dimension, other.dimension, "Dimension mismatch in add");
        let mut result_components = Vec::new();
        for lin1 in &self.components {
            for lin2 in &other.components {
                // Compute the sum of lin1 and lin2 as a new LinearSet
                let mut new_base = vec![0; self.dimension];
                // elementwise add the base vectors
                for i in 0..self.dimension {
                    new_base[i] = lin1.base[i] + lin2.base[i];
                }
                // periods: all periods from lin1 and lin2
                let mut new_periods = Vec::with_capacity(lin1.periods.len() + lin2.periods.len());
                new_periods.extend_from_slice(&lin1.periods);
                new_periods.extend_from_slice(&lin2.periods);
                // (TODO) remove duplicate period vectors in new_periods
                result_components.push( LinearSet { base: new_base, periods: new_periods } );
            }
        }
        SemilinearSet::new(self.dimension, result_components)
    }

    /// Kleene star (closure under addition) of the semilinear set.
    fn star(&self) -> SemilinearSet {
        let mut result_components = Vec::new();

        // We use bit masks to iterate over all non-empty subsets of components
        let n = self.components.len();
        // assert that the size is not too large
        debug_assert!(n <= 32, "Number of components in semilinear set is too large");
        for mask in 0..(1<<n) {
            // Determine subset X for this mask
            let mut subset_base = vec![0; self.dimension];
            let mut subset_periods: Vec<Vector> = Vec::new();
            // We'll also use a set to avoid duplicate period vectors
            use std::collections::HashSet;
            let mut period_set = HashSet::new();

            for i in 0..n {
                if mask & (1<<i) != 0 {
                    let comp = &self.components[i];
                    // add this component's base to subset_base
                    for d in 0..self.dimension {
                        subset_base[d] += comp.base[d];
                    }
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
        SemilinearSet::new(self.dimension, result_components)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semilinear_set_union() {
        let set1 = SemilinearSet::singleton(vec![1, 2]);
        let set2 = SemilinearSet::singleton(vec![3, 4]);
        let union = set1.union(&set2);
        assert_eq!(union.components.len(), 2);
        assert_eq!(union.components[0].base, vec![1, 2]);
        assert_eq!(union.components[1].base, vec![3, 4]);
    }
}
