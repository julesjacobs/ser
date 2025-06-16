// Network System (NS) automata
//
// A Network System is defined by:
// - Requests (Req -> L): Client requests that transition to a local state
// - Responses (L -> Resp): Server responses from a local state
// - Transitions (L,G -> L',G'): State transitions between local and global states

use crate::ns_to_petri::ReqPetriState;
use crate::petri::Petri;
use colored::*;
use either::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::kleene::{Kleene, Regex, nfa_to_kleene};
use crate::semilinear::*;

// Use the shared utility function for GraphViz escaping
use crate::utils::string::escape_for_graphviz_id;

// Helper function to properly quote strings for GraphViz labels
fn quote_for_graphviz(s: &str) -> String {
    format!("\"{}\"", s.replace('\"', "\\\""))
}

/// Network System representation with type parameters:
/// - G: Global state type
/// - L: Local state type
/// - Req: Request type
/// - Resp: Response type
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NS<G, L, Req, Resp> {
    /// Initial global state
    pub initial_global: G,

    /// Requests from clients with their target local states
    pub requests: Vec<(Req, L)>,

    /// Responses from local states
    pub responses: Vec<(L, Resp)>,

    /// State transitions (from_local, from_global, to_local, to_global)
    pub transitions: Vec<(L, G, L, G)>,
}

impl<G, L, Req, Resp> NS<G, L, Req, Resp>
where
    G: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Display,
    L: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Display,
    Req: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Display,
    Resp: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Display,
{
    /// Create a new Network System with the given initial global state
    pub fn new(initial_global: G) -> Self {
        NS {
            initial_global,
            requests: Vec::new(),
            responses: Vec::new(),
            transitions: Vec::new(),
        }
    }

    /// Set the initial global state
    pub fn set_initial_global(&mut self, initial_global: G) {
        self.initial_global = initial_global;
    }

    /// Add a client request with its target local state
    pub fn add_request(&mut self, request: Req, local_state: L) {
        if !self
            .requests
            .contains(&(request.clone(), local_state.clone()))
        {
            self.requests.push((request, local_state));
        }
    }

    /// Add a response from a local state
    pub fn add_response(&mut self, local_state: L, response: Resp) {
        if !self
            .responses
            .contains(&(local_state.clone(), response.clone()))
        {
            self.responses.push((local_state, response));
        }
    }

    /// Add a state transition
    pub fn add_transition(&mut self, from_local: L, from_global: G, to_local: L, to_global: G) {
        let transition = (
            from_local.clone(),
            from_global.clone(),
            to_local.clone(),
            to_global.clone(),
        );
        if !self.transitions.contains(&transition) {
            self.transitions.push(transition);
        }
    }

    /// Get all unique local states in the network system
    pub fn get_local_states(&self) -> Vec<&L> {
        let mut local_states = HashSet::new();

        // Collect local states from requests
        for (_, local) in &self.requests {
            local_states.insert(local);
        }

        // Collect local states from responses
        for (local, _) in &self.responses {
            local_states.insert(local);
        }

        // Collect local states from transitions
        for (from_local, _, to_local, _) in &self.transitions {
            local_states.insert(from_local);
            local_states.insert(to_local);
        }

        local_states.into_iter().collect()
    }

    /// Get all unique global states in the network system
    pub fn get_global_states(&self) -> Vec<&G> {
        let mut globals = HashSet::new();
        globals.insert(&self.initial_global);

        // Collect global states from transitions
        for (_, from_global, _, to_global) in &self.transitions {
            globals.insert(from_global);
            globals.insert(to_global);
        }

        globals.into_iter().collect()
    }

    /// Get all unique requests in the network system
    pub fn get_requests(&self) -> Vec<&Req> {
        let mut requests = HashSet::new();
        for (req, _) in &self.requests {
            requests.insert(req);
        }
        requests.into_iter().collect()
    }

    /// Get all unique responses in the network system
    pub fn get_responses(&self) -> Vec<&Resp> {
        let mut responses = HashSet::new();
        for (_, resp) in &self.responses {
            responses.insert(resp);
        }
        responses.into_iter().collect()
    }

    /// Make an automaton corresponding to the serialized executions of the network system
    /// An element (g, req, resp, g') is present if there is a
    /// - request req in the network system that goes to some local state l
    /// - a sequence of transitions from l to l' that transitions from g to g'
    /// - a response from l' to resp
    pub fn serialized_automaton(&self) -> Vec<(G, Req, Resp, G)> {
        let mut serialized_automaton: Vec<(G, Req, Resp, G)> = Vec::new();
        // iterate over all global states
        for g in self.get_global_states() {
            // iterate over all requests
            for (req, l) in &self.requests {
                // find all reachable states from (l, g)
                let mut todo = vec![(l, g)];
                let mut reached = HashSet::new();
                while let Some((l, g)) = todo.pop() {
                    reached.insert((l, g));
                    for (l1, g1, l2, g2) in &self.transitions {
                        if l == l1 && g == g1 && !reached.contains(&(l2, g2)) {
                            todo.push((l2, g2));
                        }
                    }
                }
                // find all reachable responses from (l, g)
                let mut reached_responses: HashSet<(&Resp, &G)> = HashSet::new();
                for (l, g) in reached {
                    for (l2, resp) in &self.responses {
                        if l == l2 {
                            reached_responses.insert((resp, g));
                        }
                    }
                }
                // add all reachable (g, req, resp, g') to the serialized automaton
                for (resp, g2) in reached_responses {
                    serialized_automaton.push((g.clone(), req.clone(), resp.clone(), g2.clone()));
                }
            }
        }
        serialized_automaton
    }

    pub fn serialized_automaton_kleene<K: Kleene + Clone>(
        &self,
        atom: impl Fn(Req, Resp) -> K,
    ) -> K {
        let nfa: Vec<(G, K, G)> = self
            .serialized_automaton()
            .into_iter()
            .map(|(g, req, resp, g2)| (g, atom(req, resp), g2))
            .collect();
        nfa_to_kleene(&nfa, self.initial_global.clone())
    }

    pub fn serialized_automaton_regex(&self) -> Regex<String> {
        self.serialized_automaton_kleene(|req, resp| Regex::Atom(format!("{req}/{resp}")))
    }

    pub fn serialized_automaton_semilinear(&self) -> SemilinearSet<String> {
        self.serialized_automaton_kleene(|req, resp| SemilinearSet::atom(format!("{req}/{resp}")))
    }

    /// Serialize the network system to a JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error>
    where
        G: Serialize,
        L: Serialize,
        Req: Serialize,
        Resp: Serialize,
    {
        serde_json::to_string_pretty(self)
    }

    /// Create a network system from a JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error>
    where
        for<'de> G: Deserialize<'de>,
        for<'de> L: Deserialize<'de>,
        for<'de> Req: Deserialize<'de>,
        for<'de> Resp: Deserialize<'de>,
    {
        serde_json::from_str(json)
    }

    /// Generate Graphviz DOT format for visualizing the network system
    pub fn to_graphviz(&self) -> String {
        let mut dot = String::from("digraph NetworkSystem {\n");
        dot.push_str("  // Graph settings\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [fontsize=10];\n");
        dot.push_str("  edge [fontsize=10];\n\n");

        // Define node styles for different types
        dot.push_str("  // Node styles\n");

        // Define separate styles without wildcards
        // Define local state nodes style with proper escaping
        let local_state_nodes: Vec<_> = self
            .get_local_states()
            .iter()
            .map(|local| format!("L_{}", escape_for_graphviz_id(&format!("{}", local))))
            .collect();
        if !local_state_nodes.is_empty() {
            dot.push_str(&format!(
                "  node [style=\"filled,rounded\", fillcolor=lightblue] {}; // Local states\n",
                local_state_nodes.join(" ")
            ));
        }

        let request_nodes: Vec<_> = self
            .get_requests()
            .iter()
            .map(|req| format!("REQ_{}", escape_for_graphviz_id(&format!("{}", req))))
            .collect();
        if !request_nodes.is_empty() {
            dot.push_str(&format!(
                "  node [shape=diamond, style=filled, fillcolor=lightgreen] {}; // Requests\n",
                request_nodes.join(" ")
            ));
        }

        let response_nodes: Vec<_> = self
            .get_responses()
            .iter()
            .map(|resp| format!("RESP_{}", escape_for_graphviz_id(&format!("{}", resp))))
            .collect();
        if !response_nodes.is_empty() {
            dot.push_str(&format!(
                "  node [shape=diamond, style=filled, fillcolor=salmon] {}; // Responses\n",
                response_nodes.join(" ")
            ));
        }
        dot.push('\n');

        // Define all local states
        dot.push_str("  // Local state nodes\n");
        let local_states = self.get_local_states();
        for local in local_states {
            let id = format!("L_{}", escape_for_graphviz_id(&format!("{}", local)));
            let label = quote_for_graphviz(&format!("{}", local));
            dot.push_str(&format!("  {} [label={}];\n", id, label));
        }

        // Define request nodes and connections to local states
        dot.push_str("\n  // Request nodes and connections\n");
        let unique_requests = self.get_requests();
        for req in unique_requests {
            // Create request node with proper escaping
            let req_id = format!("REQ_{}", escape_for_graphviz_id(&format!("{}", req)));
            let req_label = quote_for_graphviz(&format!("{}", req));
            dot.push_str(&format!("  {} [label={}];\n", req_id, req_label));

            // Connect request to local states
            for (request, local) in &self.requests {
                if request == req {
                    let local_id = format!("L_{}", escape_for_graphviz_id(&format!("{}", local)));
                    dot.push_str(&format!("  {} -> {} [style=dashed];\n", req_id, local_id));
                }
            }
        }

        // Define response nodes and connections from local states
        dot.push_str("\n  // Response nodes and connections\n");
        let unique_responses = self.get_responses();
        for resp in unique_responses {
            // Create response node with proper escaping
            let resp_id = format!("RESP_{}", escape_for_graphviz_id(&format!("{}", resp)));
            let resp_label = quote_for_graphviz(&format!("{}", resp));
            dot.push_str(&format!("  {} [label={}];\n", resp_id, resp_label));

            // Connect local states to responses
            for (local, response) in &self.responses {
                if response == resp {
                    let local_id = format!("L_{}", escape_for_graphviz_id(&format!("{}", local)));
                    dot.push_str(&format!("  {} -> {} [style=dashed];\n", local_id, resp_id));
                }
            }
        }

        // Define transitions between local states with global states
        dot.push_str("\n  // Transitions between local states with global states\n");
        for (from_local, from_global, to_local, to_global) in &self.transitions {
            let from_local_id = format!("L_{}", escape_for_graphviz_id(&format!("{}", from_local)));
            let to_local_id = format!("L_{}", escape_for_graphviz_id(&format!("{}", to_local)));
            let transition_label = quote_for_graphviz(&format!("{} → {}", from_global, to_global));

            dot.push_str(&format!(
                "  {} -> {} [label={}, color=blue, penwidth=1.5];\n",
                from_local_id, to_local_id, transition_label
            ));
        }

        // Add serialized automaton visualization
        dot.push_str("\n  // Serialized automaton\n");
        dot.push_str("  subgraph cluster_serialized {\n");
        dot.push_str("    label=\"Serialized Automaton\";\n");
        dot.push_str("    style=dashed;\n");

        // Global state nodes in serialized view
        dot.push_str("    // Global state nodes\n");
        let global_nodes: Vec<_> = self
            .get_global_states()
            .iter()
            .map(|g| format!("G_{}", escape_for_graphviz_id(&format!("{}", g))))
            .collect();
        if !global_nodes.is_empty() {
            dot.push_str(&format!("    node [style=\"filled, rounded\", fillcolor=lightblue] {}; // Global states\n\n",
                global_nodes.join(" ")));
        }

        // Get all global states for the serialized automaton
        let globals = self.get_global_states();
        for global in globals {
            // Check if this is the initial global state
            let is_initial = &self.initial_global == global;

            // Create properly escaped IDs and labels
            let global_id = format!("G_{}", escape_for_graphviz_id(&format!("{}", global)));
            let global_label = if is_initial {
                quote_for_graphviz(&format!("{} (initial)", global))
            } else {
                quote_for_graphviz(&format!("{}", global))
            };

            // Style initial global state differently
            if is_initial {
                dot.push_str(&format!(
                    "    {} [label={}, penwidth=3, color=darkgreen];\n",
                    global_id, global_label
                ));
            } else {
                dot.push_str(&format!("    {} [label={}];\n", global_id, global_label));
            }
        }

        // Add transitions in the serialized automaton
        dot.push_str("\n    // Transitions in serialized automaton\n");
        let serialized = self.serialized_automaton();
        for (from_global, req, resp, to_global) in &serialized {
            let from_global_id =
                format!("G_{}", escape_for_graphviz_id(&format!("{}", from_global)));
            let to_global_id = format!("G_{}", escape_for_graphviz_id(&format!("{}", to_global)));
            let transition_label = quote_for_graphviz(&format!("{} / {}", req, resp));

            dot.push_str(&format!(
                "    {} -> {} [label={}];\n",
                from_global_id, to_global_id, transition_label
            ));
        }

        dot.push_str("  }\n");

        // Close the graph
        dot.push_str("}\n");

        dot
    }

    /// Save GraphViz DOT files to disk and generate visualizations
    ///
    /// # Arguments
    /// * `name` - Base name for the generated files
    /// * `open_files` - Whether to open the generated PNG files for viewing
    ///
    /// Returns a Result with the paths to the generated files or an error message
    pub fn save_graphviz(&self, name: &str, open_files: bool) -> Result<Vec<String>, String> {
        let dot_content = self.to_graphviz();
        crate::graphviz::save_graphviz(&dot_content, name, "network", open_files)
    }

    /// Save GraphViz DOT files to disk and generate visualizations without opening files
    ///
    /// This is a convenience wrapper that calls save_graphviz(name, false)
    pub fn save_graphviz_no_open(&self, name: &str) -> Result<Vec<String>, String> {
        self.save_graphviz(name, false)
    }

    pub fn merge_requests(&mut self, other: &NS<G, L, Req, Resp>) {
        // Merge all requests
        for (req, l) in &other.requests {
            self.add_request(req.clone(), l.clone());
        }

        // Merge all the transitions
        for (l1, g1, l2, g2) in &other.transitions {
            self.add_transition(l1.clone(), g1.clone(), l2.clone(), g2.clone());
        }

        // Merge all responses
        for (l, resp) in &other.responses {
            self.add_response(l.clone(), resp.clone());
        }
    }
}

impl<G, L, Req, Resp> NS<G, L, Req, Resp>
where
    G: Clone + Ord + Hash + Display + Debug,
    L: Clone + Ord + Hash + Display + Debug,
    Req: Clone + Ord + Hash + Display + Debug,
    Resp: Clone + Ord + Hash + Display + Debug,
{
    /// Check if the network system is serializable using both methods and report results
    #[must_use]
    pub fn is_serializable(&self, out_dir: &str) -> bool {
        use crate::ns_to_petri::*;
        use ReqPetriState::*;

        // Initialize debug logger
        let program_name = std::path::Path::new(out_dir)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        crate::reachability::init_debug_logger(
            program_name.clone(),
            format!("Network System: {:?}", self),
        );
        let start_time = std::time::Instant::now();

        // Initialize and get reference to debug logger for ns-level logging
        let debug_logger = crate::reachability::get_debug_logger();
        debug_logger.step(
            "Initialization",
            "Starting serializability analysis",
            &format!("Program: {}\nOutput directory: {}", program_name, out_dir),
        );

        let mut places_that_must_be_zero = HashSet::new();
        let petri = ns_to_petri_with_requests(self).rename(|st| match st {
            Response(_, _) => Right(st),
            Global(_) => Left(st),
            Local(_, _) | Request(_) => {
                places_that_must_be_zero.insert(st.clone());
                Left(st)
            }
        });
        let places_that_must_be_zero: Vec<_> = places_that_must_be_zero.into_iter().collect();

        debug_logger.log_petri_net(
            "Original Petri Net",
            "Petri net converted from Network System",
            &petri,
        );
        debug_logger.step(
            "Places to Zero",
            "Places that must be zero for serializability",
            &format!(
                "Places: [{}]",
                places_that_must_be_zero
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );

        let ser: SemilinearSet<_> = self.serialized_automaton_kleene(|req, resp| {
            SemilinearSet::singleton(SparseVector::unit(Response(req, resp)))
        });

        debug_logger.log_semilinear_set(
            "Serialized Automaton",
            "Expected serializable behavior as semilinear set",
            &ser,
        );

        // clone here so we can still use it below
        let petri_for_trace = petri.clone();

        // Call original version
        let result_original =
            crate::reachability::is_petri_reachability_set_subset_of_semilinear_new(
                petri.clone(),
                &places_that_must_be_zero,
                ser.clone(),
                out_dir,
            );

        // Call new proof-based version
        let result_with_proofs =
            crate::reachability_with_proofs::is_petri_reachability_set_subset_of_semilinear_new(
                petri.clone(),
                &places_that_must_be_zero,
                ser.clone(),
                out_dir,
            );

        let total_time = start_time.elapsed().as_millis() as u64;
        let result_original_str = if result_original {
            "Serializable"
        } else {
            "Not serializable"
        };
        // Determine the proof-based result based on Decision variant
        let (result_proofs_bool, result_proofs_str) = match &result_with_proofs {
            crate::reachability_with_proofs::Decision::Proof { .. } => (true, "Serializable"),
            crate::reachability_with_proofs::Decision::CounterExample { .. } => {
                (false, "Not serializable")
            }
        };

        // Report results
        println!("Serializability check results:");
        println!(
            "  Original method: {}",
            if result_original {
                "Serializable"
            } else {
                "Not serializable"
            }
        );
        println!(
            "  Proof-based method: {} ({})",
            match &result_with_proofs {
                crate::reachability_with_proofs::Decision::Proof { .. } => "Proof",
                crate::reachability_with_proofs::Decision::CounterExample { .. } =>
                    "CounterExample",
            },
            result_proofs_str
        );

        // Print proof or counterexample details with color
        println!();

        // ANSI color codes
        const CYAN: &str = "\x1b[36m";
        const GREEN: &str = "\x1b[32m";
        const RED: &str = "\x1b[31m";
        const YELLOW: &str = "\x1b[33m";
        const BOLD: &str = "\x1b[1m";
        const RESET: &str = "\x1b[0m";

        println!("{}{}{}{}", BOLD, CYAN, "=".repeat(80), RESET);
        println!("{}{}PROOF/COUNTEREXAMPLE DETAILS:{}", BOLD, CYAN, RESET);
        println!("{}{}{}{}", BOLD, CYAN, "=".repeat(80), RESET);

        match &result_with_proofs {
            crate::reachability_with_proofs::Decision::Proof { proof } => {
                if let Some(p) = proof {
                    println!("{}{}✅ PROOF CERTIFICATE FOUND{}", BOLD, GREEN, RESET);
                    println!("{}   Variables:{}", YELLOW, RESET);
                    // Pretty print variables
                    for (i, var) in p.variables.iter().enumerate() {
                        println!("      {}{}: {}{}", YELLOW, i, format!("{}", var), RESET);
                    }
                    println!("{}   Formula:{}", YELLOW, RESET);
                    println!("      {}", p.formula);
                } else {
                    println!(
                        "{}{}✅ PROOF: Program is serializable{} (no explicit certificate available)",
                        BOLD, GREEN, RESET
                    );
                }
            }
            crate::reachability_with_proofs::Decision::CounterExample { trace } => {
                println!("{}{}❌ COUNTEREXAMPLE TRACE FOUND{}", BOLD, RED, RESET);
                if trace.is_empty() {
                    println!(
                        "{}   (Empty trace - violation found at initial state){}",
                        YELLOW, RESET
                    );
                } else {
                    println!("{}   Transition sequence:{}", YELLOW, RESET);
                    println!("      {:?}", trace);
                    println!(
                        "{}   This trace demonstrates a non-serializable execution{}",
                        YELLOW, RESET
                    );
                    print_counterexample_trace(&petri_for_trace, trace);
                }
            }
        }
        println!("{}{}{}{}", BOLD, CYAN, "=".repeat(80), RESET);
        println!();

        // Verify consistency
        if result_original != result_proofs_bool {
            eprintln!("WARNING: Results differ between original and proof-based methods!");
            eprintln!("  Original: {}", result_original);
            eprintln!("  Proof-based: {}", result_proofs_bool);
        } else {
            println!("✓ Both methods agree on the result");
        }

        // Finalize debug report
        if let Err(e) = debug_logger.finalize(result_original_str.to_string(), total_time, out_dir)
        {
            eprintln!("Warning: Failed to generate debug report: {}", e);
        }

        // Return the original result as the primary result
        result_original
    }
}

fn display_vec<T: Display>(v: &[T]) -> String {
    v.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Prints a counterexample trace step-by-step on the given Petri net.
fn print_counterexample_trace<L, G, Req, Resp>(petri: &Petri<Either<ReqPetriState<L, G, Req, Resp>, ReqPetriState<L, G, Req, Resp>>>, trace: &[usize])
where
    L: Clone + Eq + PartialEq + Hash + std::fmt::Display,
    G: Clone + Eq + PartialEq + Hash + std::fmt::Display,
    Req: Clone + Eq + PartialEq + Hash + std::fmt::Display,
    Resp: Clone + Eq + PartialEq + Hash + std::fmt::Display,
{
    // Header
    println!("{}", "❌ COUNTEREXAMPLE TRACE FOUND".bold().red());

    if trace.is_empty() {
        // Empty trace
        println!("{}", "(Empty trace – violation at initial state)".yellow());
    } else {
        // Raw sequence
        println!(
            "{}",
            format!("Raw transition sequence: {:?}", trace).yellow()
        );

        // Replay
        let transitions = petri.get_transitions();
        let mut marking = petri.get_initial_marking();
        println!(
            "{}",
            format!("Step 0 – initial marking: {}", display_vec(&marking)).yellow()
        );

        for (i, &t_idx) in trace.iter().enumerate() {
            let (inputs, outputs) = &transitions[t_idx];

            // consume inputs
            for p in inputs {
                if let Some(pos) = marking.iter().position(|x| x == p) {
                    marking.remove(pos);
                } else {
                    println!("{}", format!("Step {} – fired t{}: input {} not in marking", i + 1, t_idx, p).bold().red());
                    assert!(false, "Input not in marking");
                }
            }
            // produce outputs
            marking.extend(outputs.clone());

            println!(
                "{}",
                format!(
                    "Step {} – fired t{}: inputs={}, outputs={}, marking={}",
                    i + 1,
                    t_idx,
                    display_vec(inputs),
                    display_vec(outputs),
                    display_vec(&marking)
                )
                .yellow()
            );
        }

        // Final marking summary
        let total_tokens = marking.len();
        let mut counts = HashMap::new();
        for p in &marking {
            *counts.entry(p).or_insert(0) += 1;
        }
        let unique_places = counts.len();
        println!(
            "{}",
            format!(
                "Final marking has {} token(s) across {} place(s)",
                total_tokens, unique_places
            )
            .yellow()
        );
        println!("{}", "Places with tokens:".yellow());
        for (place, count) in &counts {
            println!("{}", format!("{}: {} token(s)", place, count).yellow());
        }

        // Conclusion
        println!(
            "{}",
            "This trace demonstrates a non-serializable execution, with the following outputs"
                .yellow()
        );
        // cyan separator
        println!(
            "{}",
            "================================================================================
        "
            .cyan()
        );
        print!("{}", "❌ COUNTEREXAMPLE request/responses: ".bold().red());
        // for each place, look for the Debug pattern "Right(Response(...), resp)" and extract
        for (place, &cnt) in &counts {
            use crate::ns_to_petri::ReqPetriState::Response;
            match place {
                Right(Response(req, resp)) => {
                    if cnt == 1 {
                        print!("{req}/{resp} ");
                    } else {
                        print!("({req}/{resp})^{cnt} ");
                    }
                }
                _ => (),
            }
        }
        println!();
    }
}

/// Given something like `"ExprRequest { name: \"foo\" }, 0)"`,
/// returns Some(("foo", 0)) or None.
fn extract_name_and_value(s: &str) -> Option<(String, usize)> {
    // 1) Trim any surrounding quotes:
    let inner = s.trim().trim_matches('"').trim();

    // 2) Turn `\"` into plain `"` so our split-on-quote works:
    let unescaped = inner.replace("\\\"", "\"");

    // 3) Grab the request name between the first pair of real quotes:
    let name = unescaped.split('"').nth(1)?.to_string();

    // 4) Grab the number after the comma and before the `)`:
    let num_part = unescaped.splitn(2, ',').nth(1)?.trim();
    let num = num_part.trim_end_matches(')').parse::<usize>().ok()?;

    Some((name, num))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ns_parse() {
        let input = r#"
            {
                "initial_global": "G0",
                "requests": [["Req1", "L0"], ["Req2", "L1"], ["Req3", "L2"]],
                "responses": [["L0", "RespA"], ["L1", "RespB"], ["L2", "RespC"]],
                "transitions": [
                    ["L0", "G0", "L1", "G1"],
                    ["L1", "G1", "L2", "G2"],
                    ["L2", "G2", "L0", "G3"]
                ]
            }"#;

        let ns: NS<String, String, String, String> = serde_json::from_str(input).unwrap();

        assert_eq!(ns.requests.len(), 3);
        assert_eq!(ns.responses.len(), 3);
        assert_eq!(ns.transitions.len(), 3);

        assert_eq!(ns.requests[0], ("Req1".to_string(), "L0".to_string()));
        assert_eq!(ns.responses[1], ("L1".to_string(), "RespB".to_string()));
        assert_eq!(
            ns.transitions[2],
            (
                "L2".to_string(),
                "G2".to_string(),
                "L0".to_string(),
                "G3".to_string()
            )
        );
    }

    #[test]
    fn test_ns_from_json() {
        let input = r#"
            {
                "initial_global": "G0",
                "requests": [["Req1", "L0"], ["Req2", "L1"]],
                "responses": [["L0", "RespA"], ["L1", "RespB"]],
                "transitions": [
                    ["L0", "G0", "L1", "G1"],
                    ["L1", "G1", "L0", "G0"]
                ]
            }"#;

        let ns = NS::<String, String, String, String>::from_json(input).unwrap();

        assert_eq!(ns.requests.len(), 2);
        assert_eq!(ns.responses.len(), 2);
        assert_eq!(ns.transitions.len(), 2);
    }

    #[test]
    fn test_ns_build_and_serialize() {
        let mut ns = NS::<String, String, String, String>::new("EmptySession".to_string());

        // Add requests
        ns.add_request("Login".to_string(), "Start".to_string());
        ns.add_request("Query".to_string(), "LoggedIn".to_string());

        // Add responses
        ns.add_response("Start".to_string(), "LoginResult".to_string());
        ns.add_response("LoggedIn".to_string(), "QueryResult".to_string());

        // Add transitions
        ns.add_transition(
            "Start".to_string(),
            "EmptySession".to_string(),
            "LoggedIn".to_string(),
            "ActiveSession".to_string(),
        );

        ns.add_transition(
            "LoggedIn".to_string(),
            "ActiveSession".to_string(),
            "Start".to_string(),
            "EmptySession".to_string(),
        );

        // Test serialization
        let json = ns.to_json().unwrap();
        assert!(json.contains("\"requests\""));
        assert!(json.contains("\"responses\""));
        assert!(json.contains("\"transitions\""));

        // Test deserialization roundtrip
        let ns2 = NS::<String, String, String, String>::from_json(&json).unwrap();
        assert_eq!(ns.requests.len(), ns2.requests.len());
        assert_eq!(ns.transitions.len(), ns2.transitions.len());
    }

    #[test]
    fn test_get_local_and_global_states() {
        let mut ns = NS::<String, String, String, String>::new("G1".to_string());

        // Add transitions
        ns.add_transition(
            "L1".to_string(),
            "G1".to_string(),
            "L2".to_string(),
            "G2".to_string(),
        );

        ns.add_transition(
            "L2".to_string(),
            "G2".to_string(),
            "L3".to_string(),
            "G3".to_string(),
        );

        // Check local states
        let local_states = ns.get_local_states();
        assert_eq!(local_states.len(), 3);
        assert!(local_states.iter().any(|&l| l == "L1"));
        assert!(local_states.iter().any(|&l| l == "L2"));
        assert!(local_states.iter().any(|&l| l == "L3"));

        // Check global states
        let globals = ns.get_global_states();
        assert_eq!(globals.len(), 3);
        assert!(globals.iter().any(|&g| g == "G1"));
        assert!(globals.iter().any(|&g| g == "G2"));
        assert!(globals.iter().any(|&g| g == "G3"));
    }
    #[test]
    fn test_serialized_automaton_no_transitions() {
        let mut ns = NS::<String, String, String, String>::new("Initial".to_string());
        // Add a single request and response but no transitions
        ns.add_request("Req1".to_string(), "L0".to_string());
        ns.add_response("L0".to_string(), "RespA".to_string());

        // Without transitions, a request to L0 directly creates a response since L0 has
        // a response RespA. This will produce a tuple (Initial, Req1, RespA, Initial).
        let automaton = ns.serialized_automaton();
        assert_eq!(automaton.len(), 1);
        assert_eq!(
            automaton[0],
            (
                "Initial".to_string(),
                "Req1".to_string(),
                "RespA".to_string(),
                "Initial".to_string()
            )
        );
    }

    #[test]
    fn test_serialized_automaton_single_transition() {
        let mut ns = NS::<String, String, String, String>::new("G0".to_string());

        // Request/Response
        ns.add_request("Req1".to_string(), "L0".to_string());
        ns.add_response("L1".to_string(), "RespA".to_string());

        // One transition from (L0, G0) -> (L1, G1)
        ns.add_transition(
            "L0".to_string(),
            "G0".to_string(),
            "L1".to_string(),
            "G1".to_string(),
        );

        // Now we expect to see if, for initial global state "G0", the request "Req1"
        // can eventually yield a response "RespA" in global state "G1".
        // That should produce exactly one tuple: (G0, Req1, RespA, G1).
        let automaton = ns.serialized_automaton();
        assert_eq!(automaton.len(), 1);
        assert_eq!(
            automaton[0],
            (
                "G0".to_string(),
                "Req1".to_string(),
                "RespA".to_string(),
                "G1".to_string()
            )
        );
    }

    #[test]
    fn test_serialized_automaton_chain_of_transitions() {
        let mut ns = NS::<String, String, String, String>::new("G0".to_string());

        // Requests / Responses
        ns.add_request("Req1".to_string(), "L0".to_string());
        ns.add_response("L2".to_string(), "RespA".to_string());

        // Chain: (L0, G0) -> (L1, G1) -> (L2, G2)
        ns.add_transition(
            "L0".to_string(),
            "G0".to_string(),
            "L1".to_string(),
            "G1".to_string(),
        );
        ns.add_transition(
            "L1".to_string(),
            "G1".to_string(),
            "L2".to_string(),
            "G2".to_string(),
        );

        // We should get (G0, Req1, RespA, G2) because from (L0, G0),
        // we can walk transitions to (L2, G2) which has response "RespA".
        let automaton = ns.serialized_automaton();
        assert_eq!(automaton.len(), 1);
        assert_eq!(
            automaton[0],
            (
                "G0".to_string(),
                "Req1".to_string(),
                "RespA".to_string(),
                "G2".to_string()
            )
        );
    }

    #[test]
    fn test_serialized_automaton_branching_paths() {
        let mut ns = NS::<String, String, String, String>::new("G0".to_string());

        // Requests
        ns.add_request("ReqA".to_string(), "L0".to_string());
        ns.add_request("ReqB".to_string(), "L0".to_string());
        // Responses
        ns.add_response("L1".to_string(), "RespA".to_string());
        ns.add_response("L2".to_string(), "RespB".to_string());

        // Branch: (L0, G0) -> (L1, G1) or (L0, G0) -> (L2, G2)
        ns.add_transition(
            "L0".to_string(),
            "G0".to_string(),
            "L1".to_string(),
            "G1".to_string(),
        );
        ns.add_transition(
            "L0".to_string(),
            "G0".to_string(),
            "L2".to_string(),
            "G2".to_string(),
        );

        // For request "ReqA" or "ReqB" starting from initial global state G0 and local L0:
        //   - We can reach L1, G1 => yields "RespA"
        //   - We can reach L2, G2 => yields "RespB"
        //
        // So we expect:
        //   (G0, ReqA, RespA, G1)
        //   (G0, ReqA, RespB, G2)
        //   (G0, ReqB, RespA, G1)
        //   (G0, ReqB, RespB, G2)
        let mut results = ns.serialized_automaton();
        results.sort(); // sort for consistent assertion

        assert_eq!(results.len(), 4);
        assert_eq!(
            results,
            vec![
                (
                    "G0".to_string(),
                    "ReqA".to_string(),
                    "RespA".to_string(),
                    "G1".to_string()
                ),
                (
                    "G0".to_string(),
                    "ReqA".to_string(),
                    "RespB".to_string(),
                    "G2".to_string()
                ),
                (
                    "G0".to_string(),
                    "ReqB".to_string(),
                    "RespA".to_string(),
                    "G1".to_string()
                ),
                (
                    "G0".to_string(),
                    "ReqB".to_string(),
                    "RespB".to_string(),
                    "G2".to_string()
                ),
            ]
        );
    }

    #[test]
    fn test_serialized_automaton_cycle() {
        let mut ns = NS::<String, String, String, String>::new("G0".to_string());

        // Request -> local state L0
        ns.add_request("Req1".to_string(), "L0".to_string());
        // Response from local state L0
        ns.add_response("L0".to_string(), "RespX".to_string());

        // Cycle: (L0, G0) -> (L0, G0)
        ns.add_transition(
            "L0".to_string(),
            "G0".to_string(),
            "L0".to_string(),
            "G0".to_string(),
        );

        // Because there's a cycle on (L0, G0), we remain in the same local/global pair,
        // which has response "RespX". That means:
        //   from G0, with request Req1 that goes to L0, we can stay in L0, G0 indefinitely.
        // The result is (G0, Req1, RespX, G0).
        let automaton = ns.serialized_automaton();
        assert_eq!(automaton.len(), 1);
        assert_eq!(
            automaton[0],
            (
                "G0".to_string(),
                "Req1".to_string(),
                "RespX".to_string(),
                "G0".to_string()
            )
        );
    }

    #[test]
    fn test_graphviz_output() {
        let mut ns = NS::<String, String, String, String>::new("NoSession".to_string());

        // Add requests and responses
        ns.add_request("Login".to_string(), "Init".to_string());
        ns.add_response("LoggedIn".to_string(), "Success".to_string());

        // Add transition
        ns.add_transition(
            "Init".to_string(),
            "NoSession".to_string(),
            "LoggedIn".to_string(),
            "ActiveSession".to_string(),
        );

        // Generate GraphViz DOT
        let dot = ns.to_graphviz();

        // Basic checks on the output format
        assert!(dot.starts_with("digraph NetworkSystem {"));
        assert!(dot.ends_with("}\n"));

        // Check for local state nodes
        assert!(dot.contains("L_Init [label=\"Init\"]"));
        assert!(dot.contains("L_LoggedIn [label=\"LoggedIn\"]"));

        // Check for request and response nodes
        assert!(dot.contains("REQ_Login [label=\"Login\"]"));
        assert!(dot.contains("RESP_Success [label=\"Success\"]"));

        // Check for connections
        assert!(dot.contains("REQ_Login -> L_Init"));
        assert!(dot.contains("L_LoggedIn -> RESP_Success"));

        // Check for transition
        assert!(dot.contains("L_Init -> L_LoggedIn"));
        assert!(dot.contains("NoSession → ActiveSession"));

        // Check serialized automaton section
        assert!(dot.contains("subgraph cluster_serialized"));
        assert!(dot.contains("G_NoSession"));
        assert!(dot.contains("G_ActiveSession"));
        assert!(dot.contains("G_NoSession -> G_ActiveSession"));
        assert!(dot.contains("Login / Success"));
    }

    // #[test]
    // fn test_save_graphviz() {
    //     // This test is conditional on GraphViz being installed
    //     // We'll only verify the file creation, not the PNG generation

    //     let mut ns = NS::<String, String, String, String>::new("G1".to_string());

    //     // Add a simple system
    //     ns.add_request("Req".to_string(), "L1".to_string());
    //     ns.add_response("L2".to_string(), "Resp".to_string());

    //     ns.add_transition(
    //         "L1".to_string(),
    //         "G1".to_string(),
    //         "L2".to_string(),
    //         "G2".to_string(),
    //     );

    //     // Save to out directory with test prefix, don't open files during testing
    //     let result = ns.save_graphviz("test_graphviz", false);

    //     // Check if saving worked (may fail if GraphViz not installed)
    //     if result.is_ok() {
    //         let files = result.unwrap();

    //         // Check DOT files were created
    //         assert!(files.iter().any(|f| f.contains("network.dot")));

    //         // Check if files exist
    //         assert!(Path::new("out/test_graphviz/network.dot").exists());

    //         // Clean up test files
    //         let _ = fs::remove_dir_all("out/test_graphviz");
    //     }
    //     // Note: We don't assert on error case since GraphViz might not be installed
    // }
}
