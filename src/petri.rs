use std::hash::Hash;
use crate::graphviz;
use crate::affine_constraints::*;
use std::collections::HashSet;

// Helper function to escape strings for use as node IDs in GraphViz DOT language
fn escape_for_graphviz_id(s: &str) -> String {
    // Replace any non-alphanumeric characters with underscore
    // This helps avoid syntax errors in the DOT language
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[derive(Clone)]
pub struct Petri<Place> {
    initial_marking: Vec<Place>,
    transitions: Vec<(Vec<Place>, Vec<Place>)>,
}

impl<Place> Petri<Place>
where
    Place: Clone + PartialEq + Eq + Hash,
{
    /// Create a new Petri net with initial marking
    pub fn new(initial_marking: Vec<Place>) -> Self {
        Petri {
            initial_marking,
            transitions: Vec::new(),
        }
    }

    /// Add a transition to the Petri net (input places, output places)
    pub fn add_transition(&mut self, input: Vec<Place>, output: Vec<Place>) {
        self.transitions.push((input, output));
    }

    /// Get all unique places in the Petri net
    pub fn get_places(&self) -> Vec<Place> {
        let mut places = HashSet::new();

        // Collect places from initial marking
        for place in &self.initial_marking {
            places.insert(place.clone());
        }

        // Collect places from transitions
        for (input, output) in &self.transitions {
            for place in input {
                places.insert(place.clone());
            }
            for place in output {
                places.insert(place.clone());
            }
        }

        places.into_iter().collect()
    }

    /// Get the initial marking of the Petri net
    pub fn get_initial_marking(&self) -> Vec<Place> {
        self.initial_marking.clone()
    }

    /// Get all transitions in the Petri net
    pub fn get_transitions(&self) -> Vec<(Vec<Place>, Vec<Place>)> {
        self.transitions.clone()
    }
}

impl<Place> Petri<Place>
where
    Place: Clone + PartialEq + Eq + Hash + std::fmt::Display,
{
    /// Generate Graphviz DOT format for visualizing the Petri net
    pub fn to_graphviz(&self) -> String {
        let mut dot = String::from("digraph PetriNet {\n");
        dot.push_str("  // Graph settings\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [fontsize=10, fontname=\"Arial\"];\n");
        dot.push_str("  edge [fontsize=10];\n");
        dot.push_str("  bgcolor=white;\n\n");

        // Configure graph attributes for a more traditional look
        dot.push_str("  // Graph properties\n");
        dot.push_str("  graph [nodesep=0.7, ranksep=0.7];\n\n");

        // Define place nodes (circles)
        dot.push_str("  // Place nodes\n");
        dot.push_str("  node [shape=circle, width=0.7, height=0.7, fixedsize=true, style=filled, fillcolor=\"#E5F5FF\", color=\"#2080B0\", penwidth=1.2];\n");

        // Define transition nodes (rectangles/bars)
        dot.push_str("\n  // Transition nodes\n");
        dot.push_str("  node [shape=rect, width=0.5, height=0.2, fixedsize=true, style=filled, fillcolor=\"#404040\", fontcolor=white];\n");

        for (i, _) in self.transitions.iter().enumerate() {
            dot.push_str(&format!("  T_{} [label=\"t{}\", fontcolor=white];\n", i, i));
        }

        let places = self.get_places();
        // Count tokens per place
        let mut initial_count = std::collections::HashMap::new();
        // Add zero to initial_count for each place
        for place in &places {
            initial_count.insert(place.clone(), 0);
        }
        for place in &self.initial_marking {
            *initial_count.entry(place.clone()).or_insert(0) += 1;
        }

        // Define places with tokens using direct Unicode in the label
        dot.push_str("\n  // Place marking with tokens\n");
        for (place, count) in &initial_count {
            // Using HTML labels for better control of token appearance
            let dots = if *count <= 5 {
                "● ".repeat(*count).trim().to_string()
            } else {
                format!("{}●", count)
            };
            // Escape special characters for GraphViz node ID
            let escaped_place_id = format!("P_{}", escape_for_graphviz_id(&format!("{}", place)));

            // Prepare HTML label with tokens
            let token_html = if *count > 0 {
                format!(
                    "<<TABLE BORDER=\"0\" CELLBORDER=\"0\" CELLSPACING=\"0\"><TR><TD>{}</TD></TR><TR><TD><FONT POINT-SIZE=\"14\">{}</FONT></TD></TR></TABLE>>",
                    place, dots
                )
            } else {
                format!("\"{}\"", place)
            };

            dot.push_str(&format!(
                "  {} [label={}, fillcolor=\"#D0F0FF\", fontcolor=\"#000000\", fixedsize=false, style=\"filled,rounded\"];\n",
                escaped_place_id, token_html
            ));
        }

        // Count inputs and outputs for each place and transition
        // to determine if we need to show weights
        let mut input_counts = std::collections::HashMap::new();
        let mut output_counts = std::collections::HashMap::new();

        for (i, (input, output)) in self.transitions.iter().enumerate() {
            // Count inputs from each place to this transition
            for place in input {
                let key = (place.clone(), i);
                *input_counts.entry(key).or_insert(0) += 1;
            }

            // Count outputs from this transition to each place
            for place in output {
                let key = (i, place.clone());
                *output_counts.entry(key).or_insert(0) += 1;
            }
        }

        // Define transition edges with weights
        dot.push_str("\n  // Transition edges\n");
        for (i, (input, output)) in self.transitions.iter().enumerate() {
            // Process unique input places
            let mut unique_inputs = std::collections::HashMap::new();
            for place in input {
                *unique_inputs.entry(place).or_insert(0) += 1;
            }

            // Connect input places to transition with weights if needed
            for (place, count) in unique_inputs {
                let escaped_place_id =
                    format!("P_{}", escape_for_graphviz_id(&format!("{}", place)));

                if count == 1 {
                    dot.push_str(&format!(
                        "  {} -> T_{} [arrowhead=normal, color=\"#404040\", penwidth=1.2];\n",
                        escaped_place_id, i
                    ));
                } else {
                    // Add weight label for multiple arcs
                    dot.push_str(&format!(
                        "  {} -> T_{} [label=\" {}\", fontsize=12, arrowhead=normal, color=\"#404040\", penwidth=1.2];\n",
                        escaped_place_id, i, count
                    ));
                }
            }

            // Process unique output places
            let mut unique_outputs = std::collections::HashMap::new();
            for place in output {
                *unique_outputs.entry(place).or_insert(0) += 1;
            }

            // Connect transition to output places with weights if needed
            for (place, count) in unique_outputs {
                let escaped_place_id =
                    format!("P_{}", escape_for_graphviz_id(&format!("{}", place)));

                if count == 1 {
                    dot.push_str(&format!(
                        "  T_{} -> {} [arrowhead=normal, color=\"#404040\", penwidth=1.2];\n",
                        i, escaped_place_id
                    ));
                } else {
                    // Add weight label for multiple arcs
                    dot.push_str(&format!(
                        "  T_{} -> {} [label=\" {}\", fontsize=12, arrowhead=normal, color=\"#404040\", penwidth=1.2];\n",
                        i, escaped_place_id, count
                    ));
                }
            }
        }

        // Close the graph
        dot.push_str("}\n");

        dot
    }

    /// Save GraphViz DOT representation of the Petri net
    pub fn save_graphviz(&self, name: &str, open_files: bool) -> Result<Vec<String>, String> {
        let dot_content = self.to_graphviz();
        graphviz::save_graphviz(&dot_content, name, "petri", open_files)
    }
}

impl<P> Petri<P> {
    /// Run an operation on each place
    pub fn for_each_place(&self, mut f: impl for<'a> FnMut(&'a P)) {
        self.initial_marking.iter().for_each(&mut f);
        for (from, to) in &self.transitions {
            from.iter().for_each(&mut f);
            to.iter().for_each(&mut f);
        }
    }

    /// Rename all the places
    pub fn rename<Q>(self, mut f: impl FnMut(P) -> Q) -> Petri<Q> {
        Petri {
            initial_marking: self.initial_marking.into_iter().map(&mut f).collect(),
            transitions: self
                .transitions
                .into_iter()
                .map(|(from, to)| {
                    (
                        from.into_iter().map(&mut f).collect(),
                        to.into_iter().map(&mut f).collect(),
                    )
                })
                .collect(),
        }
    }
}

impl<P: Clone + PartialEq + Eq + Hash> Petri<P> {
    /// Add transitions to make arbitrarily many markings in the given place
    pub fn add_existential_place(&mut self, place: P) {
        self.add_transition(vec![], vec![place]);
    }
}

impl<Place> Petri<Place>
where
    Place: Clone + PartialEq + Eq + Hash,
{
    /// Remove transitions where input places are exactly the same as output places
    pub fn remove_identity_transitions(&mut self) {
        self.transitions.retain(|(input, output)| input != output);
    }

    /// Remove transitions that output to any of the specified zero places
    pub fn remove_transitions_outputting_to_zero_places(&mut self, zero_places: &[Place]) {
        self.transitions.retain(|(_, outputs)| {
            !outputs.iter().any(|place| zero_places.contains(place))
        });
    }

    /// Remove unreachable places from the Petri net
    /// This modifies the net by removing places that cannot be reached
    pub fn remove_unreachable_places(&mut self, unreachable_places: &[Place]) {
        // Remove unreachable places from initial marking
        self.initial_marking.retain(|place| !unreachable_places.contains(place));
        
        // Remove transitions that involve unreachable places
        self.transitions.retain(|(inputs, outputs)| {
            !inputs.iter().any(|place| unreachable_places.contains(place)) &&
            !outputs.iter().any(|place| unreachable_places.contains(place))
        });
    }

    /// Filter the Petri net to keep only reachable transitions using a worklist algorithm
    /// 
    /// Takes a list of initially reachable places and modifies the Petri net by:
    /// - Finding all transitions that can fire (all preconditions are reachable)
    /// - Removing all unreachable transitions from self.transitions
    /// 
    /// A transition is reachable (can fire) if all its precondition places are reachable.
    /// When a transition can fire, all its postcondition places become reachable.
    pub fn filter_reachable(&mut self, initial_places: &[Place]) {
        let mut reachable_places: HashSet<Place> = initial_places.iter().cloned().collect();
        let mut reachable_transitions: HashSet<usize> = HashSet::new();
        let mut worklist: Vec<Place> = initial_places.to_vec();
        
        while let Some(_current_place) = worklist.pop() {
            // Check all transitions to see if any can now fire
            for (transition_idx, (inputs, outputs)) in self.transitions.iter().enumerate() {
                // Skip if we've already processed this transition
                if reachable_transitions.contains(&transition_idx) {
                    continue;
                }
                
                // Check if all input places of this transition are reachable
                let can_fire = inputs.iter().all(|input_place| reachable_places.contains(input_place));
                
                if can_fire {
                    // Mark this transition as reachable (can fire)
                    reachable_transitions.insert(transition_idx);
                    
                    // Add all output places to reachable set and worklist
                    for output_place in outputs {
                        if !reachable_places.contains(output_place) {
                            reachable_places.insert(output_place.clone());
                            worklist.push(output_place.clone());
                        }
                    }
                }
            }
        }
        
        // Filter transitions to keep only reachable ones
        self.transitions = self.transitions
            .iter()
            .enumerate()
            .filter(|(idx, _)| reachable_transitions.contains(idx))
            .map(|(_, transition)| transition.clone())
            .collect();
    }

    /// Filter the Petri net to keep only transitions reachable from the initial marking
    pub fn filter_reachable_from_initial(&mut self) {
        let initial_marking = self.initial_marking.clone();
        self.filter_reachable(&initial_marking);
    }

    /// Flip the Petri net by reversing all transitions (input becomes output, output becomes input)
    /// This is useful for backwards reachability analysis
    pub fn flip(&mut self) {
        for (inputs, outputs) in &mut self.transitions {
            std::mem::swap(inputs, outputs);
        }
    }

    /// Filter the Petri net to keep only transitions that can reach the given target places
    /// This performs backwards reachability analysis by:
    /// 1. Flipping the net (reversing all transitions)
    /// 2. Running forward reachability from target places
    /// 3. Flipping back to original orientation
    pub fn filter_backwards_reachable(&mut self, target_places: &[Place]) {
        // Step 1: Flip the net
        self.flip();
        
        // Step 2: Run forward reachability from target places
        self.filter_reachable(target_places);
        
        // Step 3: Flip back to original orientation
        self.flip();
    }

    /// Iteratively filter the Petri net using alternating forward and backward reachability
    /// until a fixed point is reached.
    /// 
    /// This finds the minimal set of transitions that are:
    /// 1. Reachable from the initial marking (forward reachability)
    /// 2. Can reach the target places (backward reachability)
    /// 
    /// The algorithm alternates between these filters until no more transitions are removed.
    pub fn filter_bidirectional_reachable(&mut self, target_places: &[Place]) {
        let initial_places = self.initial_marking.clone();
        let mut previous_count = self.transitions.len();
        let mut iteration = 0;
        
        loop {
            iteration += 1;
            
            // Step 1: Filter forward from initial marking
            self.filter_reachable(&initial_places);
            
            // Step 2: Filter backward from target places
            self.filter_backwards_reachable(target_places);
            let after_backward = self.transitions.len();
            
            // Check if we've reached a fixed point (no changes)
            if after_backward == previous_count {
                // No transitions were removed in this iteration
                break;
            }
            
            previous_count = after_backward;
            
            // Safety check to prevent infinite loops (shouldn't be needed)
            if iteration > 100 {
                eprintln!("Warning: Bidirectional filtering exceeded 100 iterations");
                break;
            }
        }
    }

}

impl<Place> Petri<Place>
where
    Place: Clone + PartialEq + Eq + Hash + std::fmt::Display,
{
    /// Find and print all unreachable places (places with no incoming transitions,
    /// excluding places that are in the initial marking)
    pub fn find_unreachable_places(&self) -> Vec<Place> {
        let all_places = self.get_places();
        let initial_marking_set: HashSet<Place> = self.initial_marking.iter().cloned().collect();
        let mut reachable_places = HashSet::new();

        // Collect all places that appear as outputs of any transition
        for (_, output) in &self.transitions {
            for place in output {
                reachable_places.insert(place.clone());
            }
        }

        // Find places that are not in reachable_places and not in initial marking
        let unreachable: Vec<Place> = all_places
            .into_iter()
            .filter(|place| {
                !reachable_places.contains(place) && !initial_marking_set.contains(place)
            })
            .collect();

        // Print the results
        if !unreachable.is_empty() {
            println!("Unreachable places (no incoming transitions and not in initial marking):");
            for place in &unreachable {
                println!("- {}", place);
            }
        } else {
            println!("All places are either reachable (have incoming transitions) or in initial marking");
        }

        unreachable
    }
}

impl<Place> Petri<Place>
where
    Place: Clone + PartialEq + Eq + Hash,
{
    /// Returns all sink places (places with no outgoing edges to any transition)
    pub fn get_sink_places(&self) -> Vec<Place> {
        // First collect all places that appear as inputs to any transition
        let mut places_with_outputs = HashSet::new();
        for (input_places, _) in &self.transitions {
            for place in input_places {
                places_with_outputs.insert(place.clone());
            }
        }

        // Then find all places that don't appear in any transition's input
        self.get_places()
            .into_iter()
            .filter(|place| !places_with_outputs.contains(place))
            .collect()
    }
}

// Returns places that are (technically) not sinks, but effectively behave like ones.
// This occurs when these places have a constraint of being equal to zero, and have outgoing
// edges to (only) actual sinks AND all these output sinks have a constraint of being equal to zero.
impl<Place> Petri<Place>
where
    Place: Clone + PartialEq + Eq + Hash + std::fmt::Display + From<Var> + std::fmt::Debug,
{
    /// Identify non-sink places that effectively behave like sinks because:
    /// 1. They only point to sinks
    /// 2. All sinks they point to are constrained to zero
    /// 3. The place itself is constrained to zero
    pub fn get_effective_sinks(&self, clause: &[Constraint]) -> HashSet<Place> {
        // 1. Extract all zero-constrained places from the clause
        let zero_places: HashSet<Place> = Constraints::extract_zero_variables(clause)
            .into_iter()
            .map(|v| v.into())
            .collect();

        // 2. Get all sink places
        let sink_places = self.get_sink_places();
        println!("All sink places: {:?}", sink_places);

        // 3. Find non-sink places that effectively behave like sinks
        let effective_sinks: HashSet<Place> = self.get_places()
            .into_iter()
            .filter(|place| {
                // Skip actual sinks
                if sink_places.contains(place) {
                    return false;
                }

                // Must be zero-constrained
                if !zero_places.contains(place) {
                    return false;
                }

                // Check all transitions where this place is an input
                self.transitions
                    .iter()
                    .filter(|(inputs, _)| inputs.contains(place))
                    .flat_map(|(_, outputs)| outputs)
                    .all(|output| sink_places.contains(output) && zero_places.contains(output))
            })
            .collect();

        println!("Effective sinks found: {:?}", effective_sinks);
        effective_sinks
    }
}


impl<Place> Petri<Place>
where
    Place: Clone + PartialEq + Eq + Hash,
{
    /// Returns a new Petri net with:
    /// 1. The specified transitions removed
    /// 2. Any transitions that directly use the given place as input removed
    pub fn remove_transitions_and_dependents(
        &self,
        place_to_remove: &Place,
        transitions_to_remove: &HashSet<usize>,
    ) -> Petri<Place> {
        Petri {
            initial_marking: self.initial_marking.clone(),
            transitions: self.transitions
                .iter()
                .enumerate()
                .filter(|(i, (input, _))| {
                    !transitions_to_remove.contains(i) &&
                        !input.contains(place_to_remove)
                })
                .map(|(_, (input, output))| (input.clone(), output.clone()))
                .collect(),
        }
    }
}


impl<Place> Petri<Place>
where
    Place: Clone + PartialEq + Eq + Hash,
{
    pub fn can_reach_with_available_transitions(&self, destination: &Place) -> bool {
        let mut reachable_places: HashSet<Place> = self.initial_marking.iter().cloned().collect();

        // Early exit if destination is already in initial marking
        if reachable_places.contains(destination) {
            return true;
        }

        let mut changed = true;
        while changed {
            changed = false;

            // Check all transitions where ALL inputs are available
            for (inputs, outputs) in &self.transitions {
                // Verify we have ALL inputs for this transition
                if inputs.iter().all(|input| reachable_places.contains(input)) {
                    // Add all outputs if transition can fire
                    for place in outputs {
                        if !reachable_places.contains(place) {
                            reachable_places.insert(place.clone());
                            changed = true;

                            // Early exit if we've found our destination
                            if place == destination {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        reachable_places.contains(destination)
    }
}



impl<Place> Petri<Place>
where
    Place: Clone + PartialEq + Eq + Hash + std::fmt::Display + From<Var> + std::fmt::Debug,  // Added Debug
{
    pub fn deduce_transitions_that_are_locked(
        &self,
        clause: &[Constraint],
    ) -> (HashSet<usize>, HashSet<usize>) {
        // Step 1: Extract all variables with EqualToZero constraint
        let zero_vars: HashSet<Place> = Constraints::extract_zero_variables(clause)
            .into_iter()
            .map(|v| v.into())
            .collect();

        println!("Zero variables: {:?}", zero_vars);

        // Step 2: Identify all critical places (sinks + effective sinks)
        let sink_places = self.get_sink_places();
        let effective_sinks = self.get_effective_sinks(clause);

        // Combine into a single set of critical places
        let mut critical_places = sink_places;
        critical_places.extend(effective_sinks);
        println!("Critical places (sinks + effective sinks): {:?}", critical_places);

        // Initialize transition sets
        let mut locked = HashSet::new();
        let mut potentially_firing: HashSet<usize> = (0..self.transitions.len()).collect();

        // Step 4: Check sink nodes with EqualToZero constraint
        for place in critical_places {
            if zero_vars.contains(&place) {
                println!("Processing zero critical place: {:?}", place);

                // Find transitions that output to this sink
                for (i, (_, outputs)) in self.transitions.iter().enumerate() {
                    if outputs.contains(&place) && potentially_firing.contains(&i) {
                        println!("  Locking transition {} (outputs to zero critical place {:?})", i, place);
                        locked.insert(i);
                        potentially_firing.remove(&i);
                    }
                }
            }
        }

        // Step 5: Process places from locked transitions
        let mut changed = true;
        while changed {
            changed = false;

            // Collect all places output by locked transitions that aren't zero vars
            let mut places_to_check = HashSet::new();
            for &t_idx in &locked {
                for place in &self.transitions[t_idx].1 {
                    if !zero_vars.contains(place) {
                        places_to_check.insert(place.clone());
                    }
                }
            }

            for place in places_to_check {
                println!("Processing place: {:?}", place);

                // Find all transitions that output to this place
                let mut output_transitions = Vec::new();
                for (i, (_, outputs)) in self.transitions.iter().enumerate() {
                    if outputs.contains(&place) {
                        output_transitions.push(i);
                    }
                }

                // Count how many are potentially firing
                let pf_count = output_transitions.iter()
                    .filter(|&i| potentially_firing.contains(i))
                    .count();

                if pf_count <= 1 {
                    // Create new Petri net without locked transitions
                    let new_petri = self.remove_transitions_and_dependents(&place, &locked);

                    // Check if we can reach the place with available transitions
                    if !new_petri.can_reach_with_available_transitions(&place) {
                        // Find the potentially firing transition (if any)
                        if let Some(&t_idx) = output_transitions.iter()
                            .find(|&i| potentially_firing.contains(i))
                        {
                            println!("  Locking transition {} (only path to {:?} is blocked)", t_idx, place);
                            locked.insert(t_idx);
                            potentially_firing.remove(&t_idx);
                            changed = true;
                        }
                    }
                }
            }
        }

        // Final printout
        println!("Final locked transitions:");
        for &t_idx in &locked {
            println!("  t{}: {:?}", t_idx, self.transitions[t_idx]);
        }

        println!("\nFinal potentially-firing transitions:");
        for &t_idx in &potentially_firing {
            println!("  t{}: {:?}", t_idx, self.transitions[t_idx]);
        }

        (locked, potentially_firing)
    }
}


impl<Place> Petri<Place>
where
    Place: Clone + PartialEq + Eq + Hash + std::fmt::Display + From<Var> + std::fmt::Debug,
{
    pub fn deduce_zero_places_from_constraints(
        &self,
        clause: &[Constraint],
    ) -> Vec<Var> {
        println!("Starting deduction of zero places...");

        // 1. Get locked transitions
        println!("\nStep 1: Identifying locked transitions...");
        let (locked_transitions, _) = self.deduce_transitions_that_are_locked(clause);
        println!("Found {} locked transitions: {:?}", locked_transitions.len(), locked_transitions);

        // 2. Create new net without locked transitions
        println!("\nStep 2: Creating filtered Petri net...");
        let filtered_net = Petri {
            initial_marking: self.initial_marking.clone(),
            transitions: self.transitions
                .iter()
                .enumerate()
                .filter(|(i, _)| !locked_transitions.contains(i))
                .map(|(_, (i, o))| (i.clone(), o.clone()))
                .collect(),
        };
        println!("New net has {} transitions (original had {})",
                 filtered_net.transitions.len(), self.transitions.len());

        // 3. Get places that are already constrained to zero
        println!("\nStep 3: Checking existing zero constraints...");
        let existing_zero_vars = Constraints::extract_zero_variables(clause);
        let existing_zero_places: HashSet<Place> = existing_zero_vars.iter()
            .map(|v| (*v).into())  // Fixed: Dereference v before conversion
            .collect();
        println!("Already constrained to zero: {:?}", existing_zero_vars);

        // 4. Check each place in original net
        println!("\nStep 4: Analyzing place reachability...");
        let mut new_zero_vars = Vec::new();

        for place in self.get_places() {
            // Skip places already constrained to zero
            if existing_zero_places.contains(&place) {
                println!("- Place {}: Already constrained to zero", place);
                continue;
            }

            // Check reachability in filtered net
            if !filtered_net.can_reach_with_available_transitions(&place) {
                if let Some(stripped) = place.to_string().strip_prefix("P") {
                    if let Ok(idx) = stripped.parse::<usize>() {
                        let var = Var(idx);
                        println!("- Place {}: UNREACHABLE, adding Var({}) to zero list", place, idx);
                        new_zero_vars.push(var);
                    }
                }
            } else {
                println!("- Place {}: reachable", place);
            }
        }

        // Final output
        println!("\nStep 5: Final results");
        println!("New variables to constrain to zero: {:?}", new_zero_vars);

        new_zero_vars
    }
}


impl<Place> Petri<Place>
where
    Place: ToString + Clone + PartialEq + Eq + Hash,
{
    /// Produce a textual representation of this Petri net in SMPT format,
    /// with `net_name` as the net's label.
    pub fn to_pnet(&self, net_name: &str) -> String {
        crate::smpt::petri_to_pnet(self, net_name)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_reachable() {
        // Create a simple Petri net: P0 -> P1 -> P2, with P3 isolated
        let mut petri = Petri::new(vec!["P0"]);
        petri.add_transition(vec!["P0"], vec!["P1"]);      // t0: P0 -> P1
        petri.add_transition(vec!["P1"], vec!["P2"]);      // t1: P1 -> P2
        petri.add_transition(vec!["P3"], vec!["P4"]);      // t2: P3 -> P4 (unreachable)
        
        // Before filtering: should have 3 transitions
        assert_eq!(petri.transitions.len(), 3);
        
        petri.filter_reachable_from_initial();
        
        // After filtering: should have only 2 reachable transitions (t0 and t1)
        assert_eq!(petri.transitions.len(), 2);
        assert_eq!(petri.transitions[0], (vec!["P0"], vec!["P1"]));  // t0
        assert_eq!(petri.transitions[1], (vec!["P1"], vec!["P2"]));  // t1
        // t2 (P3 -> P4) should be removed
    }

    #[test]
    fn test_filter_reachable_complex() {
        // More complex net: requires multiple places to fire transition
        let mut petri = Petri::new(vec!["A", "B"]);
        petri.add_transition(vec!["A"], vec!["C"]);           // t0: A -> C
        petri.add_transition(vec!["B"], vec!["D"]);           // t1: B -> D  
        petri.add_transition(vec!["C", "D"], vec!["E"]);      // t2: C+D -> E (needs both C and D)
        petri.add_transition(vec!["F"], vec!["G"]);           // t3: F -> G (unreachable, F not reachable)
        
        // Before filtering: should have 4 transitions
        assert_eq!(petri.transitions.len(), 4);
        
        petri.filter_reachable_from_initial();
        
        // After filtering: should have only 3 reachable transitions (t0, t1, t2)
        assert_eq!(petri.transitions.len(), 3);
        assert_eq!(petri.transitions[0], (vec!["A"], vec!["C"]));           // t0
        assert_eq!(petri.transitions[1], (vec!["B"], vec!["D"]));           // t1
        assert_eq!(petri.transitions[2], (vec!["C", "D"], vec!["E"]));      // t2
        // t3 (F -> G) should be removed
    }

    #[test]
    fn test_filter_reachable_with_custom_initial() {
        // Test with custom initial places instead of initial marking
        let mut petri = Petri::new(vec!["P0"]);  // P0 in initial marking
        petri.add_transition(vec!["P0"], vec!["P1"]);      // t0: P0 -> P1
        petri.add_transition(vec!["P2"], vec!["P3"]);      // t1: P2 -> P3
        
        // Before filtering: should have 2 transitions
        assert_eq!(petri.transitions.len(), 2);
        
        // Filter reachable from custom set including P2 (not in initial marking)
        petri.filter_reachable(&["P2"]);
        
        // After filtering: should have only 1 reachable transition (t1)
        assert_eq!(petri.transitions.len(), 1);
        assert_eq!(petri.transitions[0], (vec!["P2"], vec!["P3"]));  // t1
        // t0 (P0 -> P1) should be removed since P0 is not in custom initial set
    }

    #[test]
    fn test_filter_reachable_usage_example() {
        // Example showing how to use reachability filtering to remove unreachable parts
        let mut petri = Petri::new(vec!["Start", "Resource"]);
        petri.add_transition(vec!["Start"], vec!["Process1"]);                    // t0
        petri.add_transition(vec!["Process1", "Resource"], vec!["Process2"]);    // t1  
        petri.add_transition(vec!["Process2"], vec!["End", "Resource"]);         // t2
        petri.add_transition(vec!["Unreachable"], vec!["AlsoUnreachable"]);      // t3
        
        // Before filtering: 4 transitions
        assert_eq!(petri.transitions.len(), 4);
        
        petri.filter_reachable_from_initial();
        
        // After filtering: only 3 reachable transitions remain (t0, t1, t2)
        assert_eq!(petri.transitions.len(), 3);
        assert_eq!(petri.transitions[0], (vec!["Start"], vec!["Process1"]));
        assert_eq!(petri.transitions[1], (vec!["Process1", "Resource"], vec!["Process2"]));
        assert_eq!(petri.transitions[2], (vec!["Process2"], vec!["End", "Resource"]));
        // t3 (Unreachable -> AlsoUnreachable) should be removed
    }

    #[test]
    fn test_flip() {
        let mut petri = Petri::new(vec!["P0"]);
        petri.add_transition(vec!["P0"], vec!["P1"]);                    // t0: P0 -> P1
        petri.add_transition(vec!["P1", "P2"], vec!["P3"]);              // t1: P1+P2 -> P3
        
        // Before flip
        assert_eq!(petri.transitions[0], (vec!["P0"], vec!["P1"]));
        assert_eq!(petri.transitions[1], (vec!["P1", "P2"], vec!["P3"]));
        
        petri.flip();
        
        // After flip: inputs and outputs should be swapped
        assert_eq!(petri.transitions[0], (vec!["P1"], vec!["P0"]));      // was P0 -> P1, now P1 -> P0
        assert_eq!(petri.transitions[1], (vec!["P3"], vec!["P1", "P2"])); // was P1+P2 -> P3, now P3 -> P1+P2
        
        // Flip back
        petri.flip();
        
        // Should be back to original
        assert_eq!(petri.transitions[0], (vec!["P0"], vec!["P1"]));
        assert_eq!(petri.transitions[1], (vec!["P1", "P2"], vec!["P3"]));
    }

    #[test]
    fn test_filter_backwards_reachable() {
        // Create a linear chain: P0 -> P1 -> P2 -> P3, with unreachable branch P4 -> P5
        let mut petri = Petri::new(vec!["P0"]);
        petri.add_transition(vec!["P0"], vec!["P1"]);        // t0: P0 -> P1
        petri.add_transition(vec!["P1"], vec!["P2"]);        // t1: P1 -> P2
        petri.add_transition(vec!["P2"], vec!["P3"]);        // t2: P2 -> P3
        petri.add_transition(vec!["P4"], vec!["P5"]);        // t3: P4 -> P5 (unreachable from target)
        
        // Before filtering: 4 transitions
        assert_eq!(petri.transitions.len(), 4);
        
        // Filter backwards reachable to P2 (only transitions that can lead to P2)
        petri.filter_backwards_reachable(&["P2"]);
        
        // After filtering: should keep t0 and t1 (can reach P2), remove t2 and t3
        assert_eq!(petri.transitions.len(), 2);
        assert_eq!(petri.transitions[0], (vec!["P0"], vec!["P1"]));     // t0: P0 -> P1 (can reach P2)
        assert_eq!(petri.transitions[1], (vec!["P1"], vec!["P2"]));     // t1: P1 -> P2 (can reach P2)
        // t2 (P2 -> P3) removed because it doesn't lead TO P2
        // t3 (P4 -> P5) removed because it can't reach P2
    }

    #[test]
    fn test_filter_backwards_reachable_complex() {
        // More complex backwards reachability: multiple paths to target
        let mut petri = Petri::new(vec!["A", "B"]);
        petri.add_transition(vec!["A"], vec!["C"]);          // t0: A -> C (leads to target E)
        petri.add_transition(vec!["B"], vec!["D"]);          // t1: B -> D (leads to target E)
        petri.add_transition(vec!["C", "D"], vec!["E"]);     // t2: C+D -> E (creates target E)
        petri.add_transition(vec!["E"], vec!["F"]);          // t3: E -> F (doesn't lead TO E)
        petri.add_transition(vec!["X"], vec!["Y"]);          // t4: X -> Y (unconnected, can't reach E)
        
        // Before filtering: 5 transitions
        assert_eq!(petri.transitions.len(), 5);
        
        // Filter backwards reachable to E (transitions that can lead to E)
        petri.filter_backwards_reachable(&["E"]);
        
        // Should keep t0, t1, t2 (all can lead to E), remove t3, t4
        assert_eq!(petri.transitions.len(), 3);
        assert_eq!(petri.transitions[0], (vec!["A"], vec!["C"]));        // t0: can contribute to E
        assert_eq!(petri.transitions[1], (vec!["B"], vec!["D"]));        // t1: can contribute to E  
        assert_eq!(petri.transitions[2], (vec!["C", "D"], vec!["E"]));   // t2: directly creates E
        // t3 (E -> F) removed: doesn't lead TO E
        // t4 (X -> Y) removed: unconnected
    }

    #[test]
    fn test_backwards_vs_forwards_filtering_example() {
        // Demonstrate the difference between forward and backward reachability
        let mut petri_forward = Petri::new(vec!["Start"]);
        petri_forward.add_transition(vec!["Start"], vec!["Middle"]);     // t0: Start -> Middle
        petri_forward.add_transition(vec!["Middle"], vec!["End"]);       // t1: Middle -> End  
        petri_forward.add_transition(vec!["End"], vec!["Cleanup"]);      // t2: End -> Cleanup
        petri_forward.add_transition(vec!["Isolated"], vec!["Nowhere"]); // t3: Isolated -> Nowhere
        
        let mut petri_backward = petri_forward.clone();
        
        println!("Original net: Start->Middle->End->Cleanup, Isolated->Nowhere");
        assert_eq!(petri_forward.transitions.len(), 4);
        
        // Forward reachability from "Start"
        petri_forward.filter_reachable(&["Start"]);
        println!("Forward from 'Start': {} transitions remain", petri_forward.transitions.len());
        assert_eq!(petri_forward.transitions.len(), 3); // Keep t0, t1, t2; remove t3
        
        // Backward reachability to "End" 
        petri_backward.filter_backwards_reachable(&["End"]);
        println!("Backward to 'End': {} transitions remain", petri_backward.transitions.len());
        assert_eq!(petri_backward.transitions.len(), 2); // Keep t0, t1; remove t2, t3
        
        // Forward: transitions reachable FROM start
        assert_eq!(petri_forward.transitions[0], (vec!["Start"], vec!["Middle"]));
        assert_eq!(petri_forward.transitions[1], (vec!["Middle"], vec!["End"]));
        assert_eq!(petri_forward.transitions[2], (vec!["End"], vec!["Cleanup"]));
        
        // Backward: transitions that can reach TO end
        assert_eq!(petri_backward.transitions[0], (vec!["Start"], vec!["Middle"]));
        assert_eq!(petri_backward.transitions[1], (vec!["Middle"], vec!["End"]));
        // End->Cleanup removed because it doesn't lead TO End
    }

    #[test]
    fn test_filter_bidirectional_reachable_simple() {
        // Simple case: linear chain with extra branches
        let mut petri = Petri::new(vec!["Start"]);
        petri.add_transition(vec!["Start"], vec!["A"]);          // t0: Start -> A (needed)
        petri.add_transition(vec!["A"], vec!["Target"]);         // t1: A -> Target (needed)
        petri.add_transition(vec!["Target"], vec!["After"]);     // t2: Target -> After (not needed for Target)
        petri.add_transition(vec!["Isolated"], vec!["B"]);       // t3: Isolated -> B (unreachable from Start)
        
        // Before filtering: 4 transitions
        assert_eq!(petri.transitions.len(), 4);
        
        // Bidirectional filter to Target
        petri.filter_bidirectional_reachable(&["Target"]);
        
        // Should keep only t0 and t1 (path from Start to Target)
        assert_eq!(petri.transitions.len(), 2);
        assert_eq!(petri.transitions[0], (vec!["Start"], vec!["A"]));     // t0: needed for path
        assert_eq!(petri.transitions[1], (vec!["A"], vec!["Target"]));    // t1: needed for path
        // t2 removed: Target->After doesn't help reach Target
        // t3 removed: Isolated->B unreachable from Start
    }

    #[test]
    fn test_filter_bidirectional_reachable_complex() {
        // More complex case requiring multiple iterations
        let mut petri = Petri::new(vec!["Start"]);
        petri.add_transition(vec!["Start"], vec!["A"]);          // t0: Start -> A (needed)
        petri.add_transition(vec!["A"], vec!["B"]);              // t1: A -> B (needed)
        petri.add_transition(vec!["B"], vec!["Target"]);         // t2: B -> Target (needed)
        petri.add_transition(vec!["A"], vec!["C"]);              // t3: A -> C (dead end, not needed)
        petri.add_transition(vec!["C"], vec!["D"]);              // t4: C -> D (dead end, not needed)
        petri.add_transition(vec!["Target"], vec!["E"]);         // t5: Target -> E (not needed for Target)
        petri.add_transition(vec!["Unreachable"], vec!["F"]);    // t6: Unreachable -> F (isolated)
        
        // Before filtering: 7 transitions
        assert_eq!(petri.transitions.len(), 7);
        
        // Bidirectional filter to Target
        petri.filter_bidirectional_reachable(&["Target"]);
        
        // Should keep only the direct path: Start -> A -> B -> Target
        assert_eq!(petri.transitions.len(), 3);
        assert_eq!(petri.transitions[0], (vec!["Start"], vec!["A"]));     // t0: needed
        assert_eq!(petri.transitions[1], (vec!["A"], vec!["B"]));         // t1: needed
        assert_eq!(petri.transitions[2], (vec!["B"], vec!["Target"]));    // t2: needed
        // t3, t4 removed: A->C->D is a dead end that doesn't reach Target
        // t5 removed: Target->E doesn't help reach Target
        // t6 removed: Unreachable->F is isolated from Start
    }

    #[test]
    fn test_filter_bidirectional_reachable_multiple_targets() {
        // Test with multiple target places
        let mut petri = Petri::new(vec!["Start"]);
        petri.add_transition(vec!["Start"], vec!["A"]);          // t0: Start -> A (needed for both targets)
        petri.add_transition(vec!["A"], vec!["Target1"]);        // t1: A -> Target1 (needed)
        petri.add_transition(vec!["A"], vec!["Target2"]);        // t2: A -> Target2 (needed)
        petri.add_transition(vec!["Target1"], vec!["B"]);        // t3: Target1 -> B (not needed)
        petri.add_transition(vec!["Target2"], vec!["C"]);        // t4: Target2 -> C (not needed)
        petri.add_transition(vec!["Isolated"], vec!["D"]);       // t5: Isolated -> D (unreachable)
        
        // Before filtering: 6 transitions
        assert_eq!(petri.transitions.len(), 6);
        
        // Bidirectional filter to both targets
        petri.filter_bidirectional_reachable(&["Target1", "Target2"]);
        
        // Should keep path to both targets: Start -> A -> {Target1, Target2}
        assert_eq!(petri.transitions.len(), 3);
        assert_eq!(petri.transitions[0], (vec!["Start"], vec!["A"]));     // t0: needed for both
        assert_eq!(petri.transitions[1], (vec!["A"], vec!["Target1"]));   // t1: needed for Target1
        assert_eq!(petri.transitions[2], (vec!["A"], vec!["Target2"]));   // t2: needed for Target2
        // t3, t4 removed: don't help reach the targets
        // t5 removed: isolated from Start
    }

    #[test]
    fn test_filter_bidirectional_reachable_convergence() {
        // Test case that requires multiple iterations to converge
        let mut petri = Petri::new(vec!["Start"]);
        petri.add_transition(vec!["Start"], vec!["A"]);          // t0: Start -> A (needed)
        petri.add_transition(vec!["A"], vec!["B"]);              // t1: A -> B (needed)
        petri.add_transition(vec!["B"], vec!["C"]);              // t2: B -> C (needed)
        petri.add_transition(vec!["C"], vec!["Target"]);         // t3: C -> Target (needed)
        petri.add_transition(vec!["A"], vec!["X"]);              // t4: A -> X (initially seems reachable)
        petri.add_transition(vec!["X"], vec!["Y"]);              // t5: X -> Y (depends on t4)
        petri.add_transition(vec!["Y"], vec!["Z"]);              // t6: Y -> Z (depends on t5, doesn't reach Target)
        
        // Before filtering: 7 transitions
        assert_eq!(petri.transitions.len(), 7);
        
        // Bidirectional filter to Target
        petri.filter_bidirectional_reachable(&["Target"]);
        
        // Should eliminate the X->Y->Z branch since it doesn't reach Target
        assert_eq!(petri.transitions.len(), 4);
        assert_eq!(petri.transitions[0], (vec!["Start"], vec!["A"]));     // t0: needed
        assert_eq!(petri.transitions[1], (vec!["A"], vec!["B"]));         // t1: needed  
        assert_eq!(petri.transitions[2], (vec!["B"], vec!["C"]));         // t2: needed
        assert_eq!(petri.transitions[3], (vec!["C"], vec!["Target"]));    // t3: needed
        // t4, t5, t6 removed: A->X->Y->Z doesn't contribute to reaching Target
    }

    #[test]
    fn test_bidirectional_vs_individual_filtering_demo() {
        // Demonstrate the power of bidirectional filtering vs individual approaches
        println!("\n=== Bidirectional vs Individual Filtering Demo ===");
        
        // Create a complex network with multiple branches
        let mut petri_original = Petri::new(vec!["Start"]);
        petri_original.add_transition(vec!["Start"], vec!["A"]);         // t0: Start -> A (essential)
        petri_original.add_transition(vec!["A"], vec!["B"]);             // t1: A -> B (essential)  
        petri_original.add_transition(vec!["B"], vec!["Target"]);        // t2: B -> Target (essential)
        petri_original.add_transition(vec!["A"], vec!["DeadEnd"]);       // t3: A -> DeadEnd (forward reachable, but useless)
        petri_original.add_transition(vec!["Target"], vec!["After"]);    // t4: Target -> After (backward unreachable)
        petri_original.add_transition(vec!["Isolated"], vec!["Nowhere"]); // t5: Isolated -> Nowhere (completely unreachable)
        
        println!("Original net has {} transitions", petri_original.transitions.len());
        assert_eq!(petri_original.transitions.len(), 6);
        
        // Test 1: Forward-only filtering
        let mut petri_forward = petri_original.clone();
        petri_forward.filter_reachable_from_initial();
        println!("Forward-only filtering: {} transitions remain", petri_forward.transitions.len());
        assert_eq!(petri_forward.transitions.len(), 5); // Removes only t5 (isolated), keeps everything else
        
        // Test 2: Backward-only filtering  
        let mut petri_backward = petri_original.clone();
        petri_backward.filter_backwards_reachable(&["Target"]);
        println!("Backward-only filtering: {} transitions remain", petri_backward.transitions.len());
        assert_eq!(petri_backward.transitions.len(), 3); // Removes t4, t5; keeps t3
        
        // Test 3: Bidirectional filtering (the optimal result)
        let mut petri_bidirectional = petri_original.clone();
        petri_bidirectional.filter_bidirectional_reachable(&["Target"]);
        println!("Bidirectional filtering: {} transitions remain", petri_bidirectional.transitions.len());
        assert_eq!(petri_bidirectional.transitions.len(), 3); // Keeps only essential path: t0, t1, t2
        
        // Verify the bidirectional result is optimal
        assert_eq!(petri_bidirectional.transitions[0], (vec!["Start"], vec!["A"]));
        assert_eq!(petri_bidirectional.transitions[1], (vec!["A"], vec!["B"]));
        assert_eq!(petri_bidirectional.transitions[2], (vec!["B"], vec!["Target"]));
        
        println!("✓ Bidirectional filtering found the minimal essential path!");
        println!("  Forward-only: kept dead-end branches reachable from Start");
        println!("  Backward-only: kept irrelevant transitions after Target"); 
        println!("  Bidirectional: kept only Start → A → B → Target");
    }


#[test]
fn test_sink_places() {
    let mut petri = Petri::new(vec!["P0", "P1", "P2"]);
    petri.add_transition(vec!["P0"], vec!["P1"]);  // P0 has outgoing edge
    petri.add_transition(vec!["P1"], vec!["P2"]);  // P1 has outgoing edge
    // P2 has no outgoing edges

    let sinks = petri.get_sink_places();
    assert_eq!(sinks, vec!["P2"]);
    }



    #[test]
    fn test_fred_arith_2_petri_net_filtering() {
        // (1) Encode the given Petri net
        let mut petri = Petri::new(vec!["P16"]);

        // Add transitions (input places, output places)
        petri.add_transition(vec![], vec!["P12"]);          // t0
        petri.add_transition(vec![], vec!["P6"]);           // t1
        petri.add_transition(vec!["P9"], vec!["P1"]);       // t2
        petri.add_transition(vec!["P8"], vec!["P0"]);       // t3 (originally t4)
        petri.add_transition(vec!["P12", "P16"], vec!["P14", "P17"]);  // t4 (originally t9)
        petri.add_transition(vec!["P6", "P17"], vec!["P8", "P16"]);    // t5 (originally t19)
        petri.add_transition(vec!["P15"], vec!["P5"]);      // t6
        petri.add_transition(vec!["P12", "P17"], vec!["P15", "P18"]);  // t7 (originally t10)
        petri.add_transition(vec!["P6", "P18"], vec!["P9", "P17"]);    // t8 (originally t18)


        let mut to_remove = HashSet::new();
        to_remove.insert(4);  // Remove t4

        let mut petri_without_P17_and_t4 = petri.remove_transitions_and_dependents(
            &"P17",  // Also remove any transitions that take P1 as input
            &to_remove
        );

        let b1T = petri.can_reach_with_available_transitions(&"P17");
        assert!(b1T);

        let b2F = petri_without_P17_and_t4.can_reach_with_available_transitions(&"P17");
        assert!(!b2F);

        let b3T = petri_without_P17_and_t4.can_reach_with_available_transitions(&"P16");
        assert!(b3T);

        let b4F = petri_without_P17_and_t4.can_reach_with_available_transitions(&"P8");
        assert!(!b4F);

        petri_without_P17_and_t4.add_transition(vec!["P12"], vec!["P8"]);    // NEW-transition
        let b5T = petri_without_P17_and_t4.can_reach_with_available_transitions(&"P8");
        assert!(b5T);

        let b6F = petri_without_P17_and_t4.can_reach_with_available_transitions(&"P17");
        assert!(!b6F);

        let b7T = petri_without_P17_and_t4.can_reach_with_available_transitions(&"P0");
        assert!(b7T);

        let b8F = petri_without_P17_and_t4.can_reach_with_available_transitions(&"P1");
        assert!(!b8F);

    }



    #[test]
    fn test_deduce_locked_transitions_with_petri_net_fred_arith_2() {
        // Create the Petri net with Var places
        let mut petri = Petri::new(vec![Var(16)]);  // P16

        // Add transitions using Var directly
        petri.add_transition(vec![], vec![Var(12)]);          // t0 (P12)
        petri.add_transition(vec![], vec![Var(6)]);           // t1 (P6)
        petri.add_transition(vec![Var(9)], vec![Var(1)]);     // t2 (P9->P1)
        petri.add_transition(vec![Var(8)], vec![Var(0)]);     // t3 (P8->P0)
        petri.add_transition(vec![Var(12), Var(16)], vec![Var(14), Var(17)]);  // t4
        petri.add_transition(vec![Var(6), Var(17)], vec![Var(8), Var(16)]);    // t5
        petri.add_transition(vec![Var(15)], vec![Var(5)]);    // t6 (P15->P5)
        petri.add_transition(vec![Var(12), Var(17)], vec![Var(15), Var(18)]);  // t7
        petri.add_transition(vec![Var(6), Var(18)], vec![Var(9), Var(17)]);    // t8


        // Create constraint1: P14 = 0 (Var(14) = 0)
        let constraint1 = Constraint {
            affine_formula: vec![(1, Var(14))],
            offset: 0,
            constraint_type: EqualToZero,
        };


        // Create a clause containing our constraint
        let clause = vec![constraint1];

        // Analyze the Petri net with our constraint
        let (locked, potentially_firing) = petri.deduce_transitions_that_are_locked(&clause);

        // Verify expected locked transitions
        assert_eq!(locked.len(), 2);
        assert!(locked.contains(&4), "t4 should be locked (outputs to sink P14=0)");
        assert!(locked.contains(&8), "t8 should be locked due to deduction");

    }




    #[test]
    fn test_deduce_locked_places_with_petri_net_fred_arith_2() {
        // Create the Petri net with Var places
        let mut petri = Petri::new(vec![Var(16)]);  // P16

        // Add transitions using Var directly
        petri.add_transition(vec![], vec![Var(12)]);          // t0 (P12)
        petri.add_transition(vec![], vec![Var(6)]);           // t1 (P6)
        petri.add_transition(vec![Var(9)], vec![Var(1)]);     // t2 (P9->P1)
        petri.add_transition(vec![Var(8)], vec![Var(0)]);     // t3 (P8->P0)
        petri.add_transition(vec![Var(12), Var(16)], vec![Var(14), Var(17)]);  // t4
        petri.add_transition(vec![Var(6), Var(17)], vec![Var(8), Var(16)]);    // t5
        petri.add_transition(vec![Var(15)], vec![Var(5)]);    // t6 (P15->P5)
        petri.add_transition(vec![Var(12), Var(17)], vec![Var(15), Var(18)]);  // t7
        petri.add_transition(vec![Var(6), Var(18)], vec![Var(9), Var(17)]);    // t8


        // Create constraint1: P14 = 0 (Var(14) = 0)
        let constraint1 = Constraint {
            affine_formula: vec![(1, Var(14))],
            offset: 0,
            constraint_type: EqualToZero,
        };


        // Create a clause containing our constraint
        let clause = vec![constraint1];

        let new_zero_vars = petri.deduce_zero_places_from_constraints(&clause);


        // Places P0, P1, P5, P8, P9, P15, P17, P18 --- are new places with constraints P'=0 following our deduction procedure
        assert!(new_zero_vars.contains(&Var(0)), "P0 cannot be reached");
        assert!(new_zero_vars.contains(&Var(1)), "P1 cannot be reached");
        assert!(new_zero_vars.contains(&Var(5)), "P5 cannot be reached");
        assert!(new_zero_vars.contains(&Var(8)), "P8 cannot be reached");
        assert!(new_zero_vars.contains(&Var(9)), "P9 cannot be reached");
        assert!(new_zero_vars.contains(&Var(15)), "P15 cannot be reached");
        assert!(new_zero_vars.contains(&Var(17)), "P17 cannot be reached");
        assert!(new_zero_vars.contains(&Var(18)), "P18 cannot be reached");

        // Places P6, P12, P16 --- cannot be reached
        assert!(!new_zero_vars.contains(&Var(6)), "P6 is a spawning place and can be reached");
        assert!(!new_zero_vars.contains(&Var(12)), "P12 is a spawning place and can be reached");
        assert!(!new_zero_vars.contains(&Var(16)), "P16 has an initial marking and should be reached even though P17, leading to t5, cannot");
    }

}


#[test]
fn test_effective_sinks_found_for_fred_arith_with_constraints() {
    // Create the Petri net
    let mut petri = Petri::new(vec!["P16"]);

    // Add transitions (input places, output places)
    petri.add_transition(vec![], vec!["P12"]);          // t0
    petri.add_transition(vec![], vec!["P6"]);           // t1
    petri.add_transition(vec!["P14"], vec!["P4"]);      // t2
    petri.add_transition(vec!["P13"], vec!["P3"]);      // t3
    petri.add_transition(vec!["P15"], vec!["P5"]);      // t4
    petri.add_transition(vec!["P9"], vec!["P1"]);       // t5
    petri.add_transition(vec!["P8"], vec!["P0"]);       // t6
    petri.add_transition(vec!["P10"], vec!["P2"]);      // t7
    petri.add_transition(vec!["P7", "P16"], vec!["P9", "P17"]);  // t8
    petri.add_transition(vec!["P6", "P17"], vec!["P8", "P16"]);  // t9
    petri.add_transition(vec!["P7", "P17"], vec!["P10", "P18"]); // t10
    petri.add_transition(vec!["P6", "P18"], vec!["P9", "P17"]);  // t11
    petri.add_transition(vec!["P12", "P16"], vec!["P14", "P17"]); // t12
    petri.add_transition(vec!["P11", "P17"], vec!["P13", "P16"]); // t13
    petri.add_transition(vec!["P12", "P17"], vec!["P15", "P18"]); // t14
    petri.add_transition(vec!["P11", "P18"], vec!["P14", "P17"]); // t15

    // Create the constraints clause
    let clause = vec![
        Constraint { affine_formula: vec![(-1, Var(1)), (1, Var(5))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(4))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(3))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(2))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(0))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(1))], offset: -1, constraint_type: NonNegative },
        Constraint { affine_formula: vec![(1, Var(11))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(14))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(7))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(6))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(9))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(12))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(8))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(13))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(15))], offset: 0, constraint_type: EqualToZero },
        Constraint { affine_formula: vec![(1, Var(10))], offset: 0, constraint_type: EqualToZero },
    ];


    // Convert the Petri net to use Var instead of strings for places
    let var_petri = petri.rename(|s| {
        let num = s.strip_prefix("P").unwrap().parse::<usize>().unwrap();
        Var(num)
    });

    // Get the effective sinks
    let effective_sinks = var_petri.get_effective_sinks(&clause);


    // Verify we found all expected effective sinks
    assert_eq!(effective_sinks.len(), 4);

    // All the candidates are: P8, P9, P10, P13, P14, P15
    assert!(effective_sinks.contains(&Var(8)));
    assert!(effective_sinks.contains(&Var(10)));
    assert!(effective_sinks.contains(&Var(13)));
    assert!(effective_sinks.contains(&Var(14)));

    // However,P9 and P15 won't be returned because the sinks they point to are not constrained to zero
    assert!(!effective_sinks.contains(&Var(9)));
    assert!(!effective_sinks.contains(&Var(15)));

}



