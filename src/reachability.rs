use crate::affine_constraints::*;
use crate::petri::*;
use crate::isl::affine_constraints_for_complement;
use crate::semilinear::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::fs;
use std::io::Write;

pub fn is_petri_reachability_set_subset_of_semilinear<Place>(
    petri: Petri<Place>,
    semilinear: SemilinearSet<Place>,
) -> bool
where
    Place: Clone + Hash + Ord,
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
    let _petri = petri.rename(|p| renaming[&p]);
    let semilinear = semilinear.rename(|p| renaming[&p]);

    // 1. Find the affine constraints for the bad states
    let constraints = affine_constraints_for_complement(num_vars, &semilinear);
    let xml = constraints_to_xml(&constraints, "XML-file");
    let mut tmp = tempfile::Builder::new().suffix(".xml").tempfile().unwrap();
    tmp.write_all(xml.as_bytes()).unwrap();
    let tmp = tmp.into_temp_path();
    let filename = tmp.to_str().unwrap();

    // 2. Decide if petri reaches any bad states
    return false; // TODO: Implement this
}
