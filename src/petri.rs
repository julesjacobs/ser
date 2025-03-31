use crate::graphviz;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

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
    Place: ToString,
{
    /// Produce a textual representation of this Petri net,
    /// with `net_name` as the net's label.
    pub fn to_pnet(&self, net_name: &str) -> String {
        // A small helper to sanitize non-alphanumeric chars from strings.
        fn sanitize(s: &str) -> String {
            s.chars()
                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                .collect()
        }

        let mut out = String::new();

        // 1. net {...}
        out.push_str(&format!("net {{{}}}\n", sanitize(net_name)));

        // 2. Count how many times each place appears in the initial marking.
        let mut marking_count: HashMap<String, usize> = HashMap::new();
        for place in &self.initial_marking {
            let place_str = sanitize(&place.to_string());
            *marking_count.entry(place_str).or_insert(0) += 1;
        }

        // 3. Output the "pl" lines, e.g. "pl P1 (1)"
        //    for each place in initial marking.
        for (place, count) in marking_count {
            out.push_str(&format!("pl {} ({})\n", place, count));
        }

        // 4. Output each transition, named t0, t1, ...
        for (i, (input_places, output_places)) in self.transitions.iter().enumerate() {
            // "tr tX <inputs> -> <outputs>"
            out.push_str(&format!("tr t{} ", i));

            // Input places
            for p in input_places {
                out.push_str(&sanitize(&p.to_string()));
                out.push(' ');
            }

            // Arrow
            out.push_str("-> ");

            // Output places
            let mut first = true;
            for p in output_places {
                if !first {
                    out.push(' ');
                }
                out.push_str(&sanitize(&p.to_string()));
                first = false;
            }
            out.push('\n');
        }

        out
    }
}
