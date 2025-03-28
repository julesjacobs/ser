use crate::affine_constraints::*;
use crate::isl::affine_constraints_for_complement;
use crate::petri::*;
use crate::semilinear::*;
use either::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::io::Write;
use std::fs;

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
    let mut vars = HashSet::new();
    petri.for_each_place(|place| {
        if let Right(q) = place {
            vars.insert(q.clone());
        }
    });
    semilinear.for_each_key(|q| {
        vars.insert(q.clone());
    });
    let mut vars: Vec<_> = vars.into_iter().collect();
    vars.sort(); // so the renaming is predictable
    let num_vars = vars.len();

    let renaming: HashMap<&Q, Var> = vars.iter().enumerate().map(|(i, v)| (v, Var(i))).collect();
    let _petri: Petri<Either<P, Var>> = petri.rename(|p| p.map_right(|q| renaming[&q]));
    let semilinear = semilinear.rename(|p| renaming[&p]);

    // 1. Find the affine constraints for the bad states
    let constraints = affine_constraints_for_complement(num_vars, &semilinear);

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
