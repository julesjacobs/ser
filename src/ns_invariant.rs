use crate::ns::NS;
use crate::ns_to_petri::ReqPetriState;
use crate::proof_parser::ProofInvariant;
use either::Either;
use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use std::hash::Hash;

/// Domain-specific type representing the state of a request in the NS
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RequestState<L, Resp> {
    /// Request is in-flight at local state L
    InFlight(L),
    /// Request completed with response Resp
    Completed(Resp),
}

impl<L: Display, Resp: Display> Display for RequestState<L, Resp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestState::InFlight(l) => write!(f, "InFlight({})", l),
            RequestState::Completed(resp) => write!(f, "Completed({})", resp),
        }
    }
}

/// Wrapper struct for (Req, RequestState<L, Resp>) to implement Display
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RequestStatePair<Req, L, Resp>(pub Req, pub RequestState<L, Resp>);

impl<Req: Display, L: Display, Resp: Display> Display for RequestStatePair<Req, L, Resp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.1 {
            RequestState::InFlight(l) => write!(f, "{}{}", self.0, l),
            RequestState::Completed(resp) => write!(f, "{}/{}", self.0, resp),
        }
    }
}

/// NS-level invariant structure that captures per-global-state invariants
#[derive(Clone, Debug)]
pub struct NSInvariant<G, L, Req, Resp>
where
    G: Eq + Hash,
    L: Eq + Hash,
    Req: Eq + Hash,
    Resp: Eq + Hash,
{
    /// For each global state, invariant over RequestStatePair<Req, L, Resp>
    /// RequestState::InFlight(L) means request is in-flight at local state L
    /// RequestState::Completed(Resp) means request completed with response Resp
    pub global_invariants: HashMap<G, ProofInvariant<RequestStatePair<Req, L, Resp>>>,
}

impl<G, L, Req, Resp> NSInvariant<G, L, Req, Resp>
where
    G: Display + Eq + Hash + Display,
    L: Display + Eq + Hash + Display,
    Req: Display + Eq + Hash + Display,
    Resp: Display + Eq + Hash + Display,
{
    /// Pretty print the NS invariant
    pub fn pretty_print(&self) {
        println!("NS-Level Invariants per Global State:");
        println!("=====================================");
        
        for (global_state, invariant) in &self.global_invariants {
            println!("\nGlobal State: {}", global_state);
            println!("-------------");
            
            // Print variables
            println!("Variables:");
            for (i, pair) in invariant.variables.iter().enumerate() {
                println!("  [{}] {}", i, pair);
            }
            
            // Print formula
            println!("Formula: {}", invariant.formula);
        }
    }
}

/// Translate a Petri net proof to NS-level invariants
pub fn translate_petri_proof_to_ns<G, L, Req, Resp>(
    petri_proof: ProofInvariant<Either<ReqPetriState<L, G, Req, Resp>, ReqPetriState<L, G, Req, Resp>>>,
    ns: &NS<G, L, Req, Resp>,
) -> NSInvariant<G, L, Req, Resp>
where
    G: Clone + Eq + Hash + Debug + Display,
    L: Clone + Eq + Hash + Debug + Display,
    Req: Clone + Eq + Hash + Debug + Display,
    Resp: Clone + Eq + Hash + Debug + Display,
{
    let mut global_invariants = HashMap::new();
    
    // Get all global states from the NS
    let global_states = ns.get_global_states();
    
    for global_state in global_states {
        // Create substitution mapping for this global state
        let specialized_proof = petri_proof.substitute(|place| {
            match place {
                // LEFT side - Global, Local, Request places
                Either::Left(req_petri_state) => match req_petri_state {
                    ReqPetriState::Global(g) => {
                        if g == global_state {
                            Either::Right(1) // This global state is active
                        } else {
                            Either::Right(0) // Other global states are inactive
                        }
                    }
                    ReqPetriState::Local(req, l) => {
                        // Map to RequestStatePair with InFlight state
                        Either::Left(RequestStatePair(req.clone(), RequestState::InFlight(l.clone())))
                    }
                    ReqPetriState::Request(_) => {
                        // Requests are always 0 in serializable execution
                        Either::Right(0)
                    }
                    ReqPetriState::Response(_, _) => {
                        panic!("Response found in Left - this should be unreachable!");
                    }
                }
                
                // RIGHT side - Response places
                Either::Right(req_petri_state) => match req_petri_state {
                    ReqPetriState::Response(req, resp) => {
                        // Map to RequestStatePair with Completed state
                        Either::Left(RequestStatePair(req.clone(), RequestState::Completed(resp.clone())))
                    }
                    _ => {
                        panic!("Non-Response found in Right - this should be unreachable!");
                    }
                }
            }
        });
        
        global_invariants.insert(global_state.clone(), specialized_proof);
    }
    
    NSInvariant { global_invariants }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_parser::{AffineExpr, CompOp, Constraint, Formula};
    
    #[test]
    fn test_simple_substitution() {
        // Create a simple proof invariant with mixed Left/Right variables
        let expr1 = AffineExpr::from_var(Either::Left(ReqPetriState::Global("G1".to_string())));
        let constraint1 = Constraint::new(expr1, CompOp::Eq);
        
        let expr2 = AffineExpr::from_var(Either::Left(ReqPetriState::Local(
            "req1".to_string(),
            "L1".to_string(),
        )));
        let constraint2 = Constraint::new(expr2, CompOp::Geq);
        
        let formula = Formula::And(vec![
            Formula::Constraint(constraint1),
            Formula::Constraint(constraint2),
        ]);
        
        let proof = ProofInvariant {
            variables: vec![
                Either::Left(ReqPetriState::Global("G1".to_string())),
                Either::Left(ReqPetriState::Local("req1".to_string(), "L1".to_string())),
            ],
            formula,
        };
        
        // Create a simple NS for context
        let mut ns = NS::<String, String, String, String>::new("G1".to_string());
        ns.add_request("req1".to_string(), "L1".to_string());
        
        // Translate to NS-level invariant
        let ns_invariant = translate_petri_proof_to_ns(proof, &ns);
        
        // Check that we have an invariant for global state G1
        assert!(ns_invariant.global_invariants.contains_key(&"G1".to_string()));
        
        // The invariant for G1 should have substituted G1 = 1 and mapped Local to (req, Left(L))
        let g1_invariant = &ns_invariant.global_invariants[&"G1".to_string()];
        assert_eq!(g1_invariant.variables.len(), 1); // Only the local state variable remains
        assert_eq!(
            g1_invariant.variables[0],
            RequestStatePair("req1".to_string(), RequestState::InFlight("L1".to_string()))
        );
    }
}