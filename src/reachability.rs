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
    // 1. Make new names for all the output places
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
    let mut non_outputs: Vec<_> = non_outputs.into_iter().collect();
    outputs.sort(); // so the renaming is predictable
    non_outputs.sort();
    let renaming: HashMap<&Q, Var> = outputs
        .iter()
        .enumerate()
        .map(|(i, q)| (q, Var(i)))
        .collect();
    let mut petri: Petri<Either<P, Var>> = petri.rename(|p| p.map_right(|q| renaming[&q]));
    let semilinear = semilinear.rename(|p| renaming[&p]);

    // Compute the constraints
    let mut constraints = affine_constraints_for_complement(outputs.len(), &semilinear);

    // Reify existential vars as places in the petri net
    for i in 0..constraints.num_existential_vars {
        petri.add_existential_place(Right(Var(constraints.num_vars + i)));
    }
    constraints.num_vars += constraints.num_existential_vars;
    constraints.num_existential_vars = 0;

    // Rename the non-output places; assert that they are zero at the end
    let renaming: HashMap<&P, Var> = non_outputs
        .iter()
        .enumerate()
        .map(|(i, p)| (p, Var(i + constraints.num_vars)))
        .collect();
    let petri: Petri<Var> = petri.rename(|place| match place {
        Left(p) => renaming[&p],
        Right(v) => v,
    });
    constraints.num_vars += non_outputs.len();
    for (_, v) in renaming {
        constraints.assert(Constraint {
            affine_formula: vec![(1, v)],
            offset: 0,
            constraint_type: EqualToZero,
        });
    }

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
