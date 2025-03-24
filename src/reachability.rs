use crate::petri::*;
use crate::semilinear::*;
use std::hash::Hash;

pub fn is_petri_reachability_set_subset_of_semilinear<Place>(
    petri: &Petri<Place>,
    semilinear: &SemilinearSet<Place>,
) -> bool
where
    Place: Clone + PartialEq + Eq + Hash + std::fmt::Display + Ord,
{
    // 1. Construct an ISL set for the semilinear set
    // 2. Complement it
    todo!();
}
