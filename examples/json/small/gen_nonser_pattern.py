#!/usr/bin/env python3
"""
Generate non-serializable Network Systems using the intermediate state observation pattern.

Key insight: In serial execution, a request only sees global states at its start and end.
In concurrent execution, a request can observe intermediate global states created by other
requests that are executing simultaneously.
"""

import json
import os
import subprocess
import itertools

def run_serializability_checker(filepath):
    """Run the serializability checker on a file."""
    try:
        script_dir = os.path.dirname(os.path.abspath(__file__))
        project_root = os.path.dirname(os.path.dirname(os.path.dirname(script_dir)))
        
        result = subprocess.run(
            ['cargo', 'run', '--', filepath],
            cwd=project_root,
            capture_output=True,
            text=True,
            timeout=30
        )
        
        output = result.stdout + result.stderr
        
        if result.returncode != 0:
            return None, True, output
        
        # Check result
        if "Proof (Serializable)" in output or "✅ Proof certificate is VALID" in output:
            return True, False, output
        elif "CounterExample (Not serializable)" in output or "✅ Trace validation PASSED" in output:
            return False, False, output
        else:
            return None, True, output
            
    except subprocess.TimeoutExpired:
        return None, True, "Timeout"
    except Exception as e:
        return None, True, str(e)

def generate_intermediate_state_pattern(num_intermediate_states=1):
    """
    Generate NS where requests can observe intermediate states.
    
    Pattern:
    - Initial state: S0
    - Final states after serial execution: S0 (if no request completes) or S0 (if request completes and returns)
    - Intermediate states during execution: S1, S2, ... (created temporarily)
    - Responses reveal which state was observed
    """
    
    # Create states
    states = [f"S{i}" for i in range(num_intermediate_states + 1)]
    initial_state = states[0]
    
    # Single request type that modifies state and returns
    requests = [["Req", "L0"]]
    
    # Responses for each possible observed state
    responses = []
    for i, state in enumerate(states):
        responses.append([f"L_saw_{state}", f"saw_{state}"])
    
    # Transitions
    transitions = []
    
    # Forward path: L0 -> L1 -> L2 -> ... -> L0 (modifying global state along the way)
    for i in range(len(states)):
        if i < len(states) - 1:
            # Move to next local state and next global state
            transitions.append([
                f"L{i}", states[i],
                f"L{i+1}", states[i+1]
            ])
        else:
            # Return to initial local and global state
            transitions.append([
                f"L{i}", states[i],
                "L0", states[0]
            ])
    
    # Observation transitions: from L0, observe current global state
    for i, state in enumerate(states):
        transitions.append([
            "L0", state,
            f"L_saw_{state}", state
        ])
    
    ns = {
        "initial_global": initial_state,
        "requests": requests,
        "responses": responses,
        "transitions": transitions
    }
    
    return ns

def generate_write_skew_pattern():
    """
    Generate a write skew pattern where two requests each check a condition
    and modify different parts of the state, but their concurrent execution
    violates an invariant that would be maintained in serial execution.
    """
    
    # States represent invariant: at least one of A or B must be true
    states = ["BothTrue", "ATrue", "BTrue", "BothFalse"]
    
    requests = [
        ["ReqA", "checkA"],
        ["ReqB", "checkB"]
    ]
    
    responses = [
        ["done", "modified"],
        ["done", "kept_same"]
    ]
    
    transitions = [
        # ReqA: if B is true, set A to false
        ["checkA", "BothTrue", "done", "BTrue"],    # B is true, so set A false
        ["checkA", "BTrue", "done", "BTrue"],       # B already true, keep it
        ["checkA", "ATrue", "done", "ATrue"],       # B is false, must keep A true
        
        # ReqB: if A is true, set B to false  
        ["checkB", "BothTrue", "done", "ATrue"],    # A is true, so set B false
        ["checkB", "ATrue", "done", "ATrue"],       # A already true, keep it
        ["checkB", "BTrue", "done", "BTrue"],       # A is false, must keep B true
        
        # The problem: concurrent execution can lead to BothFalse
        # If both start at BothTrue, both see the other as true, both set themselves false
        ["checkA", "ATrue", "done", "BothFalse"],   # Intermediate state from ReqB
        ["checkB", "BTrue", "done", "BothFalse"],   # Intermediate state from ReqA
    ]
    
    return {
        "initial_global": "BothTrue",
        "requests": requests,
        "responses": responses,
        "transitions": transitions
    }

def generate_minimal_nonser():
    """
    Generate the minimal possible non-serializable NS.
    Requirements:
    - At least 2 global states (initial and intermediate)
    - At least 1 request
    - At least 2 responses (to distinguish observations)
    - Enough transitions for state change and observation
    """
    
    return {
        "initial_global": "G0",
        "requests": [["R", "L0"]],
        "responses": [
            ["L1", "saw_G0"],
            ["L2", "saw_G1"]
        ],
        "transitions": [
            # State change path
            ["L0", "G0", "L3", "G1"],  # Create intermediate state
            ["L3", "G1", "L0", "G0"],  # Return to initial
            
            # Observation paths
            ["L0", "G0", "L1", "G0"],  # See initial state
            ["L0", "G1", "L2", "G1"],  # See intermediate state
        ]
    }

def main():
    output_dir = os.path.dirname(os.path.abspath(__file__))
    
    patterns = [
        ("minimal", generate_minimal_nonser()),
        ("intermediate_1", generate_intermediate_state_pattern(1)),
        ("intermediate_2", generate_intermediate_state_pattern(2)),
        ("intermediate_3", generate_intermediate_state_pattern(3)),
        ("write_skew", generate_write_skew_pattern()),
    ]
    
    found_nonser = []
    
    print("Testing intermediate state observation patterns...\n")
    
    for name, ns in patterns:
        filename = f"pattern_{name}.json"
        filepath = os.path.join(output_dir, filename)
        
        # Save NS
        with open(filepath, 'w') as f:
            json.dump(ns, f, indent=2)
        
        print(f"Testing {name} pattern...")
        print(f"  States: {len(set([ns['initial_global']] + [t[1] for t in ns['transitions']] + [t[3] for t in ns['transitions']]))}")
        print(f"  Requests: {len(ns['requests'])}")
        print(f"  Responses: {len(ns['responses'])}")
        print(f"  Transitions: {len(ns['transitions'])}")
        
        # Test serializability
        is_ser, has_error, output = run_serializability_checker(filepath)
        
        if has_error:
            print(f"  ⚠️  Error during checking")
        elif is_ser is False:
            print(f"  ✅ NON-SERIALIZABLE!")
            found_nonser.append((name, ns))
            
            # Rename to indicate non-serializable
            new_filename = f"pattern_{name}_nonser.json"
            new_filepath = os.path.join(output_dir, new_filename)
            os.rename(filepath, new_filepath)
            
            # Extract counterexample if present
            if "COUNTEREXAMPLE request/responses:" in output:
                for line in output.split('\n'):
                    if "COUNTEREXAMPLE request/responses:" in line:
                        print(f"     Counterexample: {line.split(':', 1)[1].strip()}")
                        break
        elif is_ser is True:
            print(f"  ❌ Serializable")
            os.rename(filepath, os.path.join(output_dir, f"pattern_{name}_ser.json"))
        else:
            print(f"  ⚠️  Unknown result")
        
        print()
    
    if found_nonser:
        print(f"\n🎉 Found {len(found_nonser)} non-serializable patterns!")
        print("\nSummary of non-serializable patterns:")
        for name, ns in found_nonser:
            size = (len(set([ns['initial_global']] + [t[1] for t in ns['transitions']] + [t[3] for t in ns['transitions']])) +
                   len(set([r[1] for r in ns['requests']] + [r[0] for r in ns['responses']] + 
                          [t[0] for t in ns['transitions']] + [t[2] for t in ns['transitions']])) +
                   len(ns['requests']) + len(ns['responses']) + len(ns['transitions']))
            print(f"  - {name}: size {size}")
    else:
        print("\n❌ No non-serializable patterns found. The patterns may need adjustment.")

if __name__ == "__main__":
    main()