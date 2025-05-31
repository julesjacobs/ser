use crate::affine_constraints::*;
use crate::debug_report::DebugLogger;
use crate::isl::affine_constraints_for_complement;
use crate::kleene::Kleene;
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
    debug_logger: &DebugLogger,
) -> bool
where
    P: Clone + Hash + Ord + Display + Debug,
    Q: Clone + Hash + Ord + Display + Debug,
{
    debug_logger.step("Reachability Analysis Start", "Starting new SPresburgerSet-based reachability analysis", &format!("Petri net places: [{}]\nPlaces that must be zero: [{}]\nSemilinear set: {}", petri.get_places().iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "), places_that_must_be_zero.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "), semilinear));

    // Step 1: Convert semilinear set to SPresburgerSet and embed it in Either<P,Q> domain
    let q_spresburger = SPresburgerSet::from_semilinear(semilinear);
    
    // Step 2: Create universe over places that can vary (filter out places_that_must_be_zero)
    // Since places_that_must_be_zero are constrained to 0, they don't participate in the analysis
    let all_places = petri.get_places();
    let places_that_can_vary: Vec<_> = all_places.into_iter()
        .filter(|place| {
            // Keep the place if it's not in places_that_must_be_zero
            match place {
                Left(p) => !places_that_must_be_zero.contains(p),
                Right(_) => false, // All Q-places can vary
            }
        })
        .collect();

    let varying_universe = SPresburgerSet::universe(places_that_can_vary);
    debug_logger.step("Varying Universe", "Varying universe", &format!("Varying universe: {}", varying_universe));

    let response_places = petri.get_places().iter().filter_map(|place| match place {
        Right(q) => Some(q.clone()),
        Left(_) => None,
    }).collect::<Vec<_>>();

    let response_universe = SPresburgerSet::universe(response_places);
    debug_logger.step("Response Universe", "Response universe", &format!("Response universe: {}", response_universe));

    // Step 3: Compute complement: universe - embedded_semilinear
    let complement = response_universe.difference(q_spresburger);
    debug_logger.step("Compute Complement", "Computing complement (universe - embedded_semilinear)", &format!("Complement: {}", complement));

    let complement_embedded = complement.rename(|q| Right(q));
    debug_logger.step("Complement Embedded", "Complement embedded in Either<P,Q> domain", &format!("Complement embedded: {}", complement_embedded));

    let end_result_set = varying_universe.times(complement_embedded);
    debug_logger.step("End Result Set", "End result set", &format!("End result set: {}", end_result_set));
    
    // Step 4: Check if this constraint set is reachable
    // Note: we've effectively incorporated the zero constraints by filtering the universe
    let result = !can_reach_presburger(petri, end_result_set, out_dir, debug_logger);
    
    debug_logger.step("Final Result", "Reachability analysis complete", &format!("Subset property holds: {}", result));
    
    result
}


/// Checks if a Petri net can reach any state satisfying the given SPresburgerSet constraints.
///
/// APPROACH: Convert SPresburgerSet to disjunctive normal form and check each disjunct.
/// A SPresburgerSet represents a union of constraint sets (disjuncts).
/// The Petri net can reach the SPresburgerSet if it can reach ANY of the disjuncts.
pub fn can_reach_presburger<P>(
    petri: Petri<P>, 
    mut presburger: SPresburgerSet<P>,
    out_dir: &str,
    debug_logger: &DebugLogger,
) -> bool
where
    P: Clone + Hash + Ord + Display + Debug,
{
    debug_logger.step("Presburger Reachability Start", "Expanding domain and converting to disjunctive normal form", &format!("SPresburgerSet to be checked: {}", presburger));
    
    // First step: Expand the domain of the presburger set to include all places in the Petri net
    let all_petri_places = petri.get_places();
    debug_logger.step("Domain Expansion", "Expanding presburger set domain to match Petri net", &format!("Petri net places: [{}]", all_petri_places.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")));
    
    presburger = presburger.expand_domain(all_petri_places);
    debug_logger.step("Domain Expanded", "Presburger set domain expanded", &format!("Expanded presburger set: {}", presburger));
    
    // Convert SPresburgerSet to disjunctive normal form (list of quantified sets)
    let disjuncts = presburger.to_constraint_disjuncts();
    
    debug_logger.step("Disjunct Conversion", "SPresburgerSet converted to disjuncts", &format!("Number of disjuncts: {}\nDisjuncts: {}", disjuncts.len(), disjuncts.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", ")));
    
    // Check if ANY disjunct is reachable
    for (i, quantified_set) in disjuncts.iter().enumerate() {
        debug_logger.log_disjunct_start(i, quantified_set);
        println!("Checking disjunct {}: {}", i, quantified_set);
        
        if can_reach_quantified_set(petri.clone(), quantified_set.clone(), out_dir, i, debug_logger) {
            println!("Disjunct {} is reachable - constraint set is satisfiable", i);
            debug_logger.step(&format!("Disjunct {} Result", i), "Disjunct is REACHABLE - constraint set is satisfiable", &format!("Disjunct {}: REACHABLE", i));
            return true;
        }
        debug_logger.step(&format!("Disjunct {} Result", i), "Disjunct is UNREACHABLE", &format!("Disjunct {}: UNREACHABLE", i));
    }
    
    println!("No disjuncts are reachable - constraint set is unsatisfiable");
    debug_logger.step("All Disjuncts Checked", "No disjuncts are reachable - constraint set is unsatisfiable", &format!("Checked {} disjuncts, all UNREACHABLE", disjuncts.len()));
    false
}

pub fn can_reach_quantified_set<P>(
    petri: Petri<P>,
    quantified_set: super::presburger::QuantifiedSet<P>, 
    out_dir: &str,
    disjunct_id: usize,
    debug_logger: &DebugLogger,
) -> bool
where
    P: Clone + Hash + Ord + Display + Debug,
{
    debug_logger.step(&format!("Quantified Set {} Start", disjunct_id), "Extracting and reifying existential variables", &format!("Quantified set: {}", quantified_set));
    
    let (variables, basic_constraint_set) = quantified_set.extract_and_reify_existential_variables();

    debug_logger.step(&format!("Quantified Set {} Variables", disjunct_id), "Existential variables extracted", &format!("Variables: {:?}\nBasic constraint set: {}", variables, basic_constraint_set.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ")));

    // Transform the Petri net from Petri<P> to Petri<Either<usize, P>>
    // by mapping all existing places to Right(p) and adding existential places as Left(i)
    let mut new_petri = petri.rename(|p| Either::Right(p));
    for place in variables {
        new_petri.add_existential_place(place);
    }

    debug_logger.log_petri_net(&format!("Transformed Petri Net {}", disjunct_id), "Petri net with existential variables added", &new_petri);
    debug_logger.log_constraints(&format!("Final Constraints {}", disjunct_id), "Final constraints to be checked with SMPT", &basic_constraint_set);

    can_reach_constraint_set_with_debug(new_petri, basic_constraint_set, out_dir, disjunct_id, debug_logger)
}


/// Simple reachability check with constraints using SMPT with debug logging
pub fn can_reach_constraint_set_with_debug<P>(
    mut petri: Petri<P>,
    constraints: Vec<super::presburger::Constraint<P>>,
    out_dir: &str,
    disjunct_id: usize,
    debug_logger: &DebugLogger,
) -> bool
where
    P: Clone + Hash + Ord + Display + Debug,
{
    debug_logger.log_petri_net(&format!("Pre-Pruning Petri Net {}", disjunct_id), "Petri net before pruning and optimization", &petri);
    debug_logger.log_constraints(&format!("Input Constraints {}", disjunct_id), "Constraints for reachability check", &constraints);
    
    // Prune the petri net here by doing the iterative filtering where the target places 
    // are all the nonzero variables (i.e. all places in the petri net that are not part 
    // of the zero variables)
    
    // Extract zero variables from constraints
    let zero_variables = super::presburger::Constraint::extract_zero_variables(&constraints);
    let zero_variables_set: HashSet<P> = zero_variables.into_iter().collect();
    
    debug_logger.step(&format!("Zero Variables {}", disjunct_id), "Extracted zero variables from constraints", &format!("Zero variables: {:?}", zero_variables_set));
    
    // Get all places in the Petri net
    let all_places = petri.get_places();
    
    // Find nonzero variables (target places for filtering)
    let nonzero_places: Vec<P> = all_places
        .into_iter()
        .filter(|place| !zero_variables_set.contains(place))
        .collect();
    
    debug_logger.step(&format!("Nonzero Places {}", disjunct_id), "Determined nonzero places for bidirectional filtering", &format!("Nonzero places: [{}]", nonzero_places.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")));
    
    // Apply bidirectional iterative filtering to keep only transitions that:
    // 1. Are reachable from the initial marking
    // 2. Are backward reachable from the nonzero places
    if !nonzero_places.is_empty() {
        petri.filter_bidirectional_reachable(&nonzero_places);
        debug_logger.log_petri_net(&format!("Post-Pruning Petri Net {}", disjunct_id), "Petri net after bidirectional filtering", &petri);
    } else {
        debug_logger.step(&format!("Skip Pruning {}", disjunct_id), "No nonzero places found - skipping pruning", "No filtering needed");
    }
    
    crate::smpt::can_reach_constraint_set_with_logger(petri, constraints, out_dir, disjunct_id, Some(debug_logger))
}

/// Simple reachability check with constraints using SMPT
pub fn can_reach_constraint_set<P>(
    mut petri: Petri<P>,
    constraints: Vec<super::presburger::Constraint<P>>,
    out_dir: &str
) -> bool
where
    P: Clone + Hash + Ord + Display + Debug,
{
    // Prune the petri net here by doing the iterative filtering where the target places 
    // are all the nonzero variables (i.e. all places in the petri net that are not part 
    // of the zero variables)
    
    // Extract zero variables from constraints
    let zero_variables = super::presburger::Constraint::extract_zero_variables(&constraints);
    let zero_variables_set: HashSet<P> = zero_variables.into_iter().collect();
    
    // Get all places in the Petri net
    let all_places = petri.get_places();
    
    // Find nonzero variables (target places for filtering)
    let nonzero_places: Vec<P> = all_places
        .into_iter()
        .filter(|place| !zero_variables_set.contains(place))
        .collect();
    
    // Apply bidirectional iterative filtering to keep only transitions that:
    // 1. Are reachable from the initial marking
    // 2. Are backward reachable from the nonzero places
    if !nonzero_places.is_empty() {
        petri.filter_bidirectional_reachable(&nonzero_places);
    }
    
    crate::smpt::can_reach_constraint_set(petri, constraints, out_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presburger::{Constraint, ConstraintType};

    #[test]
    fn test_petri_net_pruning_with_zero_constraints() {
        // Create a Petri net: Start -> A -> B -> C, with unreachable D -> E
        let mut petri = Petri::new(vec!["Start"]);
        petri.add_transition(vec!["Start"], vec!["A"]);       // t0: Start -> A (reachable)
        petri.add_transition(vec!["A"], vec!["B"]);           // t1: A -> B (reachable)  
        petri.add_transition(vec!["B"], vec!["C"]);           // t2: B -> C (reachable)
        petri.add_transition(vec!["D"], vec!["E"]);           // t3: D -> E (unreachable from Start)
        petri.add_transition(vec!["C"], vec!["F"]);           // t4: C -> F (reachable)
        
        // Before pruning: 5 transitions
        assert_eq!(petri.get_transitions().len(), 5);
        
        // Create constraints: A = 0, C = 0 (so B and F are nonzero)
        let constraints = vec![
            Constraint::new(vec![(1, "A")], 0, ConstraintType::EqualToZero),  // A = 0
            Constraint::new(vec![(1, "C")], 0, ConstraintType::EqualToZero),  // C = 0
        ];
        
        // Extract zero variables  
        let zero_vars = Constraint::extract_zero_variables(&constraints);
        assert_eq!(zero_vars.len(), 2);
        assert!(zero_vars.contains(&"A"));
        assert!(zero_vars.contains(&"C"));
        
        // Get all places and find nonzero places
        let all_places = petri.get_places();
        let zero_vars_set: HashSet<&str> = zero_vars.into_iter().collect();
        let nonzero_places: Vec<&str> = all_places
            .into_iter()
            .filter(|place| !zero_vars_set.contains(place))
            .collect();
        
        // Nonzero places should be: Start, B, D, E, F
        assert_eq!(nonzero_places.len(), 5);
        assert!(nonzero_places.contains(&"Start"));
        assert!(nonzero_places.contains(&"B"));
        assert!(nonzero_places.contains(&"D"));
        assert!(nonzero_places.contains(&"E"));
        assert!(nonzero_places.contains(&"F"));
        
        // Apply bidirectional filtering
        petri.filter_bidirectional_reachable(&nonzero_places);
        
        // After filtering, should keep only transitions that can reach nonzero places
        // from the initial marking: Start -> A -> B and B -> C -> F
        // t3 (D -> E) should be removed as it's not reachable from Start
        let remaining_transitions = petri.get_transitions();
        assert!(remaining_transitions.len() <= 4); // Should remove at least t3
        
        // Verify the remaining transitions form a path from Start to nonzero places
        let has_start_to_a = remaining_transitions.contains(&(vec!["Start"], vec!["A"]));
        let has_a_to_b = remaining_transitions.contains(&(vec!["A"], vec!["B"]));
        let has_b_to_c = remaining_transitions.contains(&(vec!["B"], vec!["C"]));
        let has_c_to_f = remaining_transitions.contains(&(vec!["C"], vec!["F"]));
        let has_d_to_e = remaining_transitions.contains(&(vec!["D"], vec!["E"]));
        
        // Should keep transitions that lead to nonzero places (B, F)
        assert!(has_start_to_a); // Needed to reach B
        assert!(has_a_to_b);     // Creates nonzero place B
        
        // Should not keep isolated transition
        assert!(!has_d_to_e);    // Not reachable from Start
    }
}