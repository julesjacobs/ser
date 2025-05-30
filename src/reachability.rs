use crate::affine_constraints::*;
use crate::isl::affine_constraints_for_complement;
use crate::petri::*;
use crate::semilinear::*;
use crate::spresburger::SPresburgerSet;
use either::{Either, Left, Right};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::fs;
use std::hash::Hash;
use std::io::Write;

/// Checks if the reachability set of the Petri net has the property that:
///   If places_that_must_be_zero are zero, then marking is in the semilinear set
/// To do this we check whether ~Q /\ P=0 is reachable.
pub fn is_petri_reachability_set_subset_of_semilinear<P, Q>(
    petri: Petri<Either<P, Q>>,
    places_that_must_be_zero: &[P],
    semilinear: SemilinearSet<Q>,
    out_dir: &str,
) -> bool
where
    P: Clone + Hash + Ord + Display,
    Q: Clone + Hash + Ord + Display,
{
    // Make new names for all the output places
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
    let mut renaming_readable = String::new();
    for q in &outputs {
        renaming_readable.push_str(&format!("{} <-> {}\n", renaming[q], q));
    }

    // Compute the constraints
    let mut constraints = affine_constraints_for_complement(outputs.len(), &semilinear);

    // Reify existential vars as places in the petri net
    for i in 0..constraints.num_existential_vars {
        let v = Var(constraints.num_vars + i);
        petri.add_existential_place(Right(v));
        renaming_readable.push_str(&format!("{v} <-> new existential variable\n"));
    }
    constraints.num_vars += constraints.num_existential_vars;
    constraints.num_existential_vars = 0;

    // Rename the non-output places; assert that they are zero at the end
    let renaming: HashMap<&P, Var> = non_outputs
        .iter()
        .enumerate()
        .map(|(i, p)| (p, Var(i + constraints.num_vars)))
        .collect();
    let mut petri: Petri<Var> = petri.rename(|place| match place {
        Left(p) => renaming[&p],
        Right(v) => v,
    });
    for p in &non_outputs {
        renaming_readable.push_str(&format!("{} <-> {}\n", renaming[p], p));
    }
    constraints.num_vars += places_that_must_be_zero.len();
    for p in places_that_must_be_zero {
        constraints.assert(Constraint {
            affine_formula: vec![(1, renaming[p])],
            offset: 0,
            constraint_type: EqualToZero,
        });
    }


    println!("*************************");

    // per each (AND) clause constraint
    for (i, and_clause) in constraints.constraints.iter_mut().enumerate() {
        println!("\n#####");
        println!("\nProcessing AND clause {}:", i);
        println!("Current constraints:");
        for constraint in &*and_clause {
            println!("  {:?}", constraint);
        }
        // deduce invariant on places that must be zero
        let deduced_new_zero_vars = petri.deduce_zero_places_from_constraints(&and_clause);

        // add to the current (iterated) AND clause a new constraint of the deduced places that is 0
        for var in deduced_new_zero_vars {
            and_clause.push(Constraint {
                affine_formula: vec![(1, var)],
                offset: 0,
                constraint_type: EqualToZero,
            });
        }

    }


    println!("*************************");

    // identify non-reachable places, and add a constraint that their marking is 0
    let unreachable = petri.find_unreachable_places();
    constraints.assert_places_zero(&unreachable);

    // IMPORTANT: to do this after finding upstream paths, as this changes the numbering of the transitions
    // remove transitions with input places = output places
    petri.remove_identity_transitions();

    // Save the Petri Net
    let string_representation_of_petri_net = petri.to_pnet(out_dir);
    let petri_net_file_output_path = format!("{}/temp_interleaving_petri_net.net", out_dir);
    fs::write(
        &petri_net_file_output_path,
        string_representation_of_petri_net,
    )
    .expect("Failed to write final Petri Net to output path");

    // Save the renaming
    fs::write(&format!("{out_dir}/temp_renaming.txt"), renaming_readable)
        .expect("Failed to write human-readable renaming");

    // Encode the constraints in XML for the SMPT tool
    let xml = constraints_to_xml(&constraints, "XML-file");
    let mut tmp = tempfile::Builder::new().suffix(".xml").tempfile().unwrap();
    tmp.write_all(xml.as_bytes()).unwrap();
    let tmp = tmp.into_temp_path();
    let _filename = tmp.to_str().unwrap();

    // also, save the XML in the main output directory
    let xml_file_output_path = format!("{}/temp_non_serializable_outputs.xml", out_dir);
    fs::write(&xml_file_output_path, xml).expect("Failed to write XML to output path");

    // 4. Run the SMPT tool
    return false; // TODO: Implement this
    // TODO: add optimization: if Constraints are empty (=FALSE) for the complement semilinear set, then
    // just return "FALSE". Currently the generated XML (e.g., simple_ser) is not parsed correctly
}

//=============================================================================
// NEW ARCHITECTURE - REACHABILITY CHECKING WITH SPRESBURGER SETS
//=============================================================================


/// NEW IMPLEMENTATION: Checks if the reachability set of a Petri net is a subset of a semilinear set
/// using the new SPresburgerSet-based architecture.
///
/// GOAL: Check if Reachable(petri) ⊆ semilinear when places_that_must_be_zero = 0
/// APPROACH: Check if ¬semilinear ∩ {places_that_must_be_zero = 0} is reachable
///          If this intersection is reachable, then the subset property is violated
pub fn is_petri_reachability_set_subset_of_semilinear_new<P, Q>(
    petri: Petri<Either<P, Q>>,
    places_that_must_be_zero: &[P],
    semilinear: SemilinearSet<Q>,
    out_dir: &str,
) -> bool
where
    P: Clone + Hash + Ord + Display + Debug,
    Q: Clone + Hash + Ord + Display + Debug,
{
    // Step 1: Convert semilinear set to SPresburgerSet and embed it in Either<P,Q> domain
    let q_spresburger = SPresburgerSet::from_semilinear(semilinear);
    let embedded_semilinear = q_spresburger.rename(|q| Right(q));
    
    // Step 2: Create universe over places that can vary (filter out places_that_must_be_zero)
    // Since places_that_must_be_zero are constrained to 0, they don't participate in the analysis
    let all_places = petri.get_places();
    let places_that_can_vary: Vec<_> = all_places.into_iter()
        .filter(|place| {
            // Keep the place if it's not in places_that_must_be_zero
            match place {
                Left(p) => !places_that_must_be_zero.contains(p),
                Right(_) => true, // All Q-places can vary
            }
        })
        .collect();
    
    let universe = SPresburgerSet::universe(places_that_can_vary);
    
    // Step 3: Compute complement: universe - embedded_semilinear
    let complement = universe.difference(embedded_semilinear);
    
    // Step 4: Check if this constraint set is reachable
    // Note: we've effectively incorporated the zero constraints by filtering the universe
    !can_reach_presburger(petri, complement, out_dir)
}

/// Checks if a Petri net can reach any state satisfying the given SPresburgerSet constraints.
///
/// APPROACH: Convert SPresburgerSet to disjunctive normal form and check each disjunct.
/// A SPresburgerSet represents a union of constraint sets (disjuncts).
/// The Petri net can reach the SPresburgerSet if it can reach ANY of the disjuncts.
pub fn can_reach_presburger<P>(
    petri: Petri<P>, 
    mut presburger: SPresburgerSet<P>,
    out_dir: &str
) -> bool
where
    P: Clone + Hash + Ord + Display + Debug,
{
    // Convert SPresburgerSet to disjunctive normal form (list of quantified sets)
    let disjuncts = presburger.to_constraint_disjuncts();
    
    // Check if ANY disjunct is reachable
    for (i, quantified_set) in disjuncts.iter().enumerate() {
        println!("Checking disjunct {}: {:?}", i, quantified_set);
        
        if can_reach_quantified_set(petri.clone(), quantified_set.clone(), out_dir) {
            println!("Disjunct {} is reachable - constraint set is satisfiable", i);
            return true;
        }
    }
    
    println!("No disjuncts are reachable - constraint set is unsatisfiable");
    false
}

pub fn can_reach_quantified_set<P>(
    petri: Petri<P>,
    quantified_set: super::presburger::QuantifiedSet<P>, 
    out_dir: &str
) -> bool
where
    P: Clone + Hash + Ord + Display + Debug,
{
    let (variables, basic_constraint_set) = quantified_set.extract_and_reify_existential_variables();

    // Transform the Petri net from Petri<P> to Petri<Either<usize, P>>
    // by mapping all existing places to Right(p) and adding existential places as Left(i)
    let mut new_petri = petri.rename(|p| Either::Right(p));
    for place in variables {
        new_petri.add_existential_place(place);
    }

    can_reach_constraint_set(new_petri, basic_constraint_set, out_dir)
}


/// Simple reachability check with constraints using SMPT
pub fn can_reach_constraint_set<P>(
    petri: Petri<P>,
    constraints: Vec<super::presburger::Constraint<P>>,
    out_dir: &str
) -> bool
where
    P: Clone + Hash + Ord + Display + Debug,
{
    crate::smpt::can_reach_constraint_set(petri, constraints, out_dir)
}