// We convert the NS to a Petri net
// The places in the Petri net consist of the local states and the global states and the requests and responses.
// Each transition (l,g) -> (l',g') is converted to a corresponding transition in the Petri net.
// Additionally, for each request transition req -> l, we add a corresponding transition in the Petri net,
// and similarly for the response transitions l -> res.

use crate::petri::Petri;
use crate::ns::NS;
use std::hash::Hash;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PetriState<L, G, Req, Resp> {
    Local(L),
    Global(G),
    Request(Req),
    Response(Resp)
}

impl<L, G, Req, Resp> std::fmt::Display for PetriState<L, G, Req, Resp>
where
    L: std::fmt::Display,
    G: std::fmt::Display,
    Req: std::fmt::Display,
    Resp: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PetriState::Local(l) => write!(f, "L_{}", l),
            PetriState::Global(g) => write!(f, "G_{}", g),
            PetriState::Request(req) => write!(f, "REQ_{}", req),
            PetriState::Response(resp) => write!(f, "RESP_{}", resp),
        }
    }
}

pub fn ns_to_petri<L, G, Req, Resp>(ns: &NS<G, L, Req, Resp>) -> Petri<PetriState<L, G, Req, Resp>>
where
    L: Clone + PartialEq + Eq + Hash + std::fmt::Display,
    G: Clone + PartialEq + Eq + Hash + std::fmt::Display,
    Req: Clone + PartialEq + Eq + Hash + std::fmt::Display,
    Resp: Clone + PartialEq + Eq + Hash + std::fmt::Display,
{
    // Create a new Petri net with initial marking
    // Start with one token for each global state and one token for each request
    let mut initial_marking = Vec::new();
    
    // Add a token for the initial global state
    initial_marking.push(PetriState::Global(ns.initial_global.clone()));
    
    let mut petri = Petri::new(initial_marking);
    
    // Create transitions for each request transition
    for (req, local) in &ns.requests {
        petri.add_transition(
            vec![PetriState::Request(req.clone())],
            vec![PetriState::Local(local.clone())]
        );
    }
    
    // Create transitions for each response transition
    for (local, resp) in &ns.responses {
        petri.add_transition(
            vec![PetriState::Local(local.clone())],
            vec![PetriState::Response(resp.clone())]
        );
    }
    
    // Create transitions for each state transition (l, g) -> (l', g')
    for (from_local, from_global, to_local, to_global) in &ns.transitions {
        petri.add_transition(
            vec![
                PetriState::Local(from_local.clone()),
                PetriState::Global(from_global.clone())
            ],
            vec![
                PetriState::Local(to_local.clone()),
                PetriState::Global(to_global.clone())
            ]
        );
    }
    
    petri
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    
    #[test]
    fn test_ns_to_petri_simple() {
        // Create a simple network system
        // Note: NS<G, L, Req, Resp> - order of type parameters is important
        let mut ns = NS::<String, String, String, String>::new("NoSession".to_string());
        
        // Add a request and response
        ns.add_request("Login".to_string(), "Start".to_string());
        ns.add_response("LoggedIn".to_string(), "Success".to_string());
        
        // Add a transition
        ns.add_transition(
            "Start".to_string(),
            "NoSession".to_string(),
            "LoggedIn".to_string(),
            "ActiveSession".to_string(),
        );
        
        // Convert to Petri net
        let petri = ns_to_petri(&ns);
        
        // Verify the Petri net structure
        
        // Check that the places in the Petri net include all possible states
        let places: HashSet<_> = petri.get_places().into_iter().collect();
        
        // The Petri net should have places for:
        // - Request: Login
        // - Response: Success
        // - Local states: Start, LoggedIn
        // - Global states: NoSession, ActiveSession
        assert_eq!(places.len(), 6);
        
        // Check the initial marking (should contain only initial global state and request)
        let mut initial_marking_set = HashSet::new();
        for place in petri.get_initial_marking() {
            initial_marking_set.insert(place);
        }
        
        // Initial marking should contain 2 tokens: one for initial global state and one for the request
        assert_eq!(initial_marking_set.len(), 2);
        
        // Verify transitions count (one for request, one for response, one for state transition)
        assert_eq!(petri.get_transitions().len(), 3);
    }
}

