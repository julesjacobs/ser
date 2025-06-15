# Proof Certificate Implementation Plan

## Overview
This plan outlines the implementation of proof certificate support in the serializability checker, including extracting traces and proof invariants from SMPT output, making them flow through the analysis pipeline, and combining proofs from multiple disjuncts.

## Phase 1: Make ProofInvariant Generic (Foundation)

### 1.1 Modify ProofInvariant to be generic ProofInvariant<T>
- Update `src/proof_parser.rs` to make ProofInvariant, Formula, AffineExpr, and Constraint generic over T
- Change AffineExpr to use HashMap instead of BTreeMap to avoid Ord constraints
- Add trait bounds: `T: Clone + Debug + Display + Eq + Hash`
- Parser continues to return ProofInvariant<String>
- Add conversion method to map variable types: `impl<T> ProofInvariant<T> { fn map<U>(self, f: impl Fn(T) -> U) -> ProofInvariant<U> }`

### 1.2 Update proofinvariant_to_presburger.rs for generic types
- Make eliminate_forward/backward generic
- Update function signatures to work with ProofInvariant<T>
- Keep existing tests working with ProofInvariant<String>

## Phase 2: SMPT Integration with Proof Parsing (Parse at the source)

### 2.1 Parse proof certificates immediately in SMPT module
- In `run_smpt_internal`, when we have `SmptVerificationOutcome::Unreachable { proof_certificate }`:
  - Call `parse_proof_file(&proof_certificate)` to get ProofInvariant<String>
  - Store parsed ProofInvariant in the outcome instead of raw string
- Update SmptVerificationOutcome to include parsed proof:
  ```rust
  Unreachable {
      proof_certificate: Option<String>, // Keep raw for debugging
      parsed_proof: Option<ProofInvariant<String>>,
  }
  ```

## Phase 3: Handle Existential Variables in Proofs

### 3.1 Add function to handle existential quantification
- Add to `proofinvariant_to_presburger.rs`:
  ```rust
  pub fn existentially_quantify<T>(proof: ProofInvariant<Either<usize, T>>, existential_vars: &[usize]) -> ProofInvariant<T>
  ```
- This function should:
  - Filter out existential variables (Left(i)) from the variable list
  - Wrap the formula in Exists quantifiers for each existential variable
  - Map remaining variables from Either<usize, T> to just T

### 3.2 Add reverse mapping function
- Add function to map ProofInvariant back from Either<usize, P> to P after existential quantification

## Phase 4: Update Return Types (Propagate proofs up the stack)

### 4.1 Change Decision enum to carry proof/trace data
```rust
pub enum Decision<P> {
    Yes { trace: Vec<usize> },
    No { proof: Option<ProofInvariant<P>> },
}
```

### 4.2 Update function signatures bottom-up
- Start with `can_reach_constraint_set` in smpt.rs - return the parsed proof
- Update `can_reach_constraint_set_with_debug` to propagate proof/trace
- Update `can_reach_quantified_set` to:
  - Handle ProofInvariant<Either<usize, P>> from SMPT
  - Apply existential quantification for Left(i) variables
  - Return ProofInvariant<P>
- Update `can_reach_presburger` to collect and combine proofs from disjuncts
- Update top-level `is_petri_reachability_set_subset_of_semilinear_new`

## Phase 5: Implement Recursive Pruning with Proof Modification

### 5.1 Create recursive pruning that modifies proofs
- Replace `filter_bidirectional_reachable` with a recursive version that:
  - Takes a ProofInvariant as input
  - Does 1 forward step, collecting removed places
  - Calls `eliminate_forward` on the proof for removed places
  - Does 1 backward step, collecting removed places  
  - Calls `eliminate_backward` on the proof for removed places
  - Recurses up to 10 times or until fixed point
  - Returns the modified ProofInvariant

### 5.2 Thread proof through the pruning process
- In `can_reach_constraint_set_with_debug`:
  - Start with an initial ProofInvariant (universe over all places)
  - Pass it through recursive pruning
  - Use the modified proof when calling SMPT
  - Combine SMPT's proof with the pruning-modified proof

## Phase 6: Combine Disjunct Proofs

### 6.1 Implement proof combination in can_reach_presburger
- When checking multiple disjuncts:
  - If all are unreachable, AND their proofs together
  - If any is reachable, return its trace
- Handle ProofInvariant variable renaming for consistency
- Return combined proof in Decision::No

## Phase 7: Testing and Polish

### 7.1 Comprehensive testing strategy
- Unit tests for generic ProofInvariant operations
- Test proof parsing with actual SMPT output files
- Integration tests for full pipeline with known .ser examples
- Verify proofs are correctly modified during pruning
- Test disjunct combination logic

## Key Improvements in This Plan:
1. **Parse early**: We parse the proof certificate immediately when SMPT returns it, avoiding string manipulation later
2. **Generic from the start**: Making ProofInvariant generic early allows clean type flow
3. **Bottom-up updates**: Starting from SMPT and working up ensures each layer is testable
4. **Proof threading**: The proof flows through pruning and gets modified appropriately
5. **Clear data flow**: Proof/trace data flows cleanly from SMPT → pruning → combination → final result

Each phase builds on the previous one and can be tested independently, ensuring we maintain a working system throughout the implementation.