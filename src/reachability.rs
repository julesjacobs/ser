use crate::affine_constraints::*;
use crate::isl::affine_constraints_for_complement;
use crate::petri::*;
use crate::semilinear::*;
use either::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::Hash;
use std::io::Write;

pub fn is_petri_reachability_set_subset_of_semilinear<P, Q>(
    petri: Petri<Either<P, Q>>,
    semilinear: SemilinearSet<Q>,
    out_dir: &str,
) -> bool
where
    P: Clone + Hash + Ord,
    Q: Clone + Hash + Ord,
{
    // 0. Make new names for all the vars in the semilinear set
    let mut outputs = HashSet::new();
    let mut non_outputs = HashSet::new();
    petri.for_each_place(|place| {
        match place {
            Left(p) => non_outputs.insert(p.clone()),
            Right(q) => outputs.insert(q.clone()),
        };
    });
    semilinear.for_each_key(|q| {
        outputs.insert(q.clone());
    });
    let mut outputs: Vec<_> = outputs.into_iter().collect();
    outputs.sort(); // so the renaming is predictable
    let num_vars = outputs.len();

    let renaming: HashMap<&Q, Var> = outputs
        .iter()
        .enumerate()
        .map(|(i, v)| (v, Var(i)))
        .collect();
    let petri: Petri<Either<P, Var>> = petri.rename(|p| p.map_right(|q| renaming[&q]));
    let petri: Petri<Var> =
        todo!("TODO: mark: add places for existentials, rename all places to vars");
    let semilinear = semilinear.rename(|p| renaming[&p]);

    // 1. Find the affine constraints for the bad states
    let constraints = affine_constraints_for_complement(num_vars, &semilinear);
    // TODO: mark: add constriants that the other Petri places are empty

    // 2. Encode the constraints in XML for the SMPT tool
    let xml = constraints_to_xml(&constraints, "XML-file");
    let mut tmp = tempfile::Builder::new().suffix(".xml").tempfile().unwrap();
    tmp.write_all(xml.as_bytes()).unwrap();
    let tmp = tmp.into_temp_path();
    let _filename = tmp.to_str().unwrap();

    // also, save the XML in the main output directory
    let output_path = format!("{}/non_serializable_outputs.xml", out_dir);

    /****** NEW: Write XML to both temporary and output paths ******/
    fs::write(&output_path, xml).expect("Failed to write XML to output path");

    // 3. Encode the Petri net for the SMPT tool
    // 4. Run the SMPT tool
    return false; // TODO: Implement this
}
