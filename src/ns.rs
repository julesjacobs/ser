// Network System (NS) automata
//
// A Network System is defined by:
// - Requests (Req -> L): Client requests that transition to a local state
// - Responses (L -> Resp): Server responses from a local state
// - Transitions (L,G -> L',G'): State transitions between local and global states

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Network System representation with type parameters:
/// - G: Global state type
/// - L: Local state type
/// - Req: Request type
/// - Resp: Response type
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NS<G, L, Req, Resp> {
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
    /// Create a new empty Network System
    pub fn new() -> Self {
        NS {
            requests: Vec::new(),
            responses: Vec::new(),
            transitions: Vec::new(),
        }
    }

    /// Add a client request with its target local state
    pub fn add_request(&mut self, request: Req, local_state: L) {
        self.requests.push((request, local_state));
    }

    /// Add a response from a local state
    pub fn add_response(&mut self, local_state: L, response: Resp) {
        self.responses.push((local_state, response));
    }

    /// Add a state transition
    pub fn add_transition(&mut self, from_local: L, from_global: G, to_local: L, to_global: G) {
        self.transitions
            .push((from_local, from_global, to_local, to_global));
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
        // Define local state nodes style
        let local_state_nodes: Vec<_> = self.get_local_states().iter().map(|local| format!("L_{}", local)).collect();
        if !local_state_nodes.is_empty() {
            dot.push_str(&format!("  node [style=\"filled,rounded\", fillcolor=lightblue] {}; // Local states\n",
                local_state_nodes.join(" ")));
        }
        let request_nodes: Vec<_> = self.get_requests().iter().map(|req| format!("REQ_{}", req)).collect();
        if !request_nodes.is_empty() {
            dot.push_str(&format!("  node [shape=diamond, style=filled, fillcolor=lightgreen] {}; // Requests\n",
                request_nodes.join(" ")));
        }

        let response_nodes: Vec<_> = self.get_responses().iter().map(|resp| format!("RESP_{}", resp)).collect();
        if !response_nodes.is_empty() {
            dot.push_str(&format!("  node [shape=diamond, style=filled, fillcolor=salmon] {}; // Responses\n",
                response_nodes.join(" ")));
        }
        dot.push('\n');

        // Define all local states
        dot.push_str("  // Local state nodes\n");
        let local_states = self.get_local_states();
        for local in local_states {
            dot.push_str(&format!("  L_{} [label=\"{}\"];\n", local, local));
        }

        // Define request nodes and connections to local states
        dot.push_str("\n  // Request nodes and connections\n");
        let unique_requests = self.get_requests();
        for req in unique_requests {
            // Create request node
            dot.push_str(&format!("  REQ_{} [label=\"{}\"];\n", req, req));

            // Connect request to local states
            for (request, local) in &self.requests {
                if request == req {
                    dot.push_str(&format!("  REQ_{} -> L_{} [style=dashed];\n", req, local));
                }
            }
        }

        // Define response nodes and connections from local states
        dot.push_str("\n  // Response nodes and connections\n");
        let unique_responses = self.get_responses();
        for resp in unique_responses {
            // Create response node
            dot.push_str(&format!("  RESP_{} [label=\"{}\"];\n", resp, resp));

            // Connect local states to responses
            for (local, response) in &self.responses {
                if response == resp {
                    dot.push_str(&format!("  L_{} -> RESP_{} [style=dashed];\n", local, resp));
                }
            }
        }

        // Define transitions between local states with global states
        dot.push_str("\n  // Transitions between local states with global states\n");
        for (from_local, from_global, to_local, to_global) in &self.transitions {
            dot.push_str(&format!(
                "  L_{} -> L_{} [label=\"{} → {}\", color=blue, penwidth=1.5];\n",
                from_local, to_local, from_global, to_global
            ));
        }

        // Add serialized automaton visualization
        dot.push_str("\n  // Serialized automaton\n");
        dot.push_str("  subgraph cluster_serialized {\n");
        dot.push_str("    label=\"Serialized Automaton\";\n");
        dot.push_str("    style=dashed;\n");

        // Global state nodes in serialized view
        dot.push_str("    // Global state nodes\n");
        let global_nodes: Vec<_> = self.get_global_states().iter().map(|g| format!("G_{}", g)).collect();
        if !global_nodes.is_empty() {
            dot.push_str(&format!("    node [style=\"filled, rounded\", fillcolor=lightblue] {}; // Global states\n\n",
                global_nodes.join(" ")));
        }

        // Get all global states for the serialized automaton
        let globals = self.get_global_states();
        for global in globals {
            dot.push_str(&format!("    G_{} [label=\"{}\"];\n", global, global));
        }

        // Add transitions in the serialized automaton
        dot.push_str("\n    // Transitions in serialized automaton\n");
        let serialized = self.serialized_automaton();
        for (from_global, req, resp, to_global) in &serialized {
            dot.push_str(&format!(
                "    G_{} -> G_{} [label=\"{} / {}\"];\n",
                from_global, to_global, req, resp
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
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::*;

    #[test]
    fn test_ns_parse() {
        let input = r#"
            {
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
        let mut ns = NS::<String, String, String, String>::new();

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
        let mut ns = NS::<String, String, String, String>::new();

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
        let mut ns = NS::<String, String, String, String>::new();
        // Add a single request and response but no transitions
        ns.add_request("Req1".to_string(), "L0".to_string());
        ns.add_response("L0".to_string(), "RespA".to_string());

        // Without transitions, each (local, global) is just (L0, ?), but since
        // we don't have any global states defined, get_global_states() is empty.
        // That means we never enter the main loop since get_global_states() returns [].
        // So the serialized automaton should be empty.
        let automaton = ns.serialized_automaton();
        assert_eq!(automaton.len(), 0);
    }

    #[test]
    fn test_serialized_automaton_single_transition() {
        let mut ns = NS::<String, String, String, String>::new();

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

        // Now we expect to see if, for global state "G0", the request "Req1"
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
        let mut ns = NS::<String, String, String, String>::new();

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
        let mut ns = NS::<String, String, String, String>::new();

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

        // For request "ReqA" or "ReqB" starting from global state G0 and local L0:
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
        let mut ns = NS::<String, String, String, String>::new();

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
        let mut ns = NS::<String, String, String, String>::new();

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

    #[test]
    fn test_save_graphviz() {
        // This test is conditional on GraphViz being installed
        // We'll only verify the file creation, not the PNG generation

        let mut ns = NS::<String, String, String, String>::new();

        // Add a simple system
        ns.add_request("Req".to_string(), "L1".to_string());
        ns.add_response("L2".to_string(), "Resp".to_string());

        ns.add_transition(
            "L1".to_string(),
            "G1".to_string(),
            "L2".to_string(),
            "G2".to_string(),
        );

        // Save to out directory with test prefix, don't open files during testing
        let result = ns.save_graphviz("test", false);

        // Check if saving worked (may fail if GraphViz not installed)
        if result.is_ok() {
            let files = result.unwrap();

            // Check DOT files were created
            assert!(files.iter().any(|f| f.contains("test_network.dot")));

            // Check if files exist
            assert!(Path::new("out/test_network.dot").exists());

            // Clean up test files
            let _ = fs::remove_file("out/test_network.dot");

            // PNG files may or may not exist depending on GraphViz installation
            if Path::new("out/test_network.png").exists() {
                let _ = fs::remove_file("out/test_network.png");
            }
        }
        // Note: We don't assert on error case since GraphViz might not be installed
    }
}
