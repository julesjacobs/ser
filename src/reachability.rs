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

    let renaming: HashMap<&Place, Var> = places
        .iter()
        .enumerate()
        .map(|(i, v)| (v, Var(i)))
        .collect();
    let _petri = petri.rename(|p| renaming[&p]);
    let _semilinear = semilinear.rename(|p| renaming[&p]);

    // 1. Construct an ISL set for the semilinear set
    // 2. Complement it
    todo!();
}
