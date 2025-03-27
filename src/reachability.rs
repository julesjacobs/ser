use crate::affine_constraints::*;
use crate::petri::*;
use crate::semilinear::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub fn is_petri_reachability_set_subset_of_semilinear<Place>(
    petri: Petri<Place>,
    semilinear: SemilinearSet<Place>,
) -> bool
where
    Place: Clone + PartialEq + Eq + Hash + Ord,
{
    // 0. Make new names for all the places
    let mut places = HashSet::new();
    petri.for_each_place(|p| {
        places.insert(p.clone());
    });
    semilinear.for_each_key(|p| {
        places.insert(p.clone());
    });
    let mut places: Vec<_> = places.into_iter().collect();
    places.sort(); // so the renaming is predictable
    let num_vars = places.len();

    let renaming: HashMap<&Place, Var> = places
        .iter()
        .enumerate()
        .map(|(i, v)| (v, Var(i)))
        .collect();
    let petri = petri.rename(|p| renaming[&p]);
    let semilinear = semilinear.rename(|p| renaming[&p]);

    // 1. Find the affine constraints for the bad states
    let constraints = affine_constraints_for_complement(num_vars, semilinear);

    // 2. Decide if petri reaches any bad states
    return false; // TODO: Implement this
}
