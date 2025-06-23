#!/usr/bin/env python3
"""
Find all non-serializable NS up to a given size.

Uses the intermediate state observation pattern as a template,
but explores variations systematically.
"""

import json
import os
import subprocess
import itertools
from collections import defaultdict

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
            timeout=5
        )
        
        output = result.stdout + result.stderr
        
        if "CounterExample (Not serializable)" in output or "NOT SERIALIZABLE" in output:
            return False
        elif "Proof (Serializable)" in output or "SERIALIZABLE" in output:
            return True
        else:
            return None
            
    except:
        return None

def generate_ns_variations(max_size=15, max_states=3, max_requests=2):
    """Generate NS variations focusing on patterns likely to be non-serializable."""
    
    for size in range(8, max_size + 1):
        # Try different allocations of the size budget
        # Prioritize: multiple responses, few requests, enough transitions
        
        for num_global in range(2, min(max_states + 1, size - 5)):  # At least 2 global states
            for num_requests in range(1, min(max_requests + 1, 3)):  # 1 or 2 requests
                for num_responses in range(2, min(5, size - num_global - num_requests - 2)):  # Multiple responses
                    
                    remaining = size - num_global - num_requests - num_responses
                    
                    # Split remaining between local states and transitions
                    for num_local in range(2, min(max_states + 1, remaining)):
                        num_transitions = remaining - num_local
                        
                        if num_transitions < 4:  # Need at least 4 transitions
                            continue
                        
                        # Generate NS with these parameters
                        yield from generate_specific_ns(
                            num_global, num_local, num_requests, num_responses, num_transitions
                        )

def generate_specific_ns(num_global, num_local, num_requests, num_responses, num_transitions):
    """Generate specific NS configurations with given parameters."""
    
    global_states = [f"G{i}" for i in range(num_global)]
    local_states = [f"L{i}" for i in range(num_local)]
    request_names = [f"R{i}" for i in range(num_requests)]
    response_names = [f"resp{i}" for i in range(num_responses)]
    
    # Focus on single request with multiple responses
    if num_requests == 1 and num_responses >= 2:
        
        # Basic pattern: state changes with observations
        requests = [[request_names[0], local_states[0]]]
        
        # Map different local states to different responses
        responses = []
        for i in range(num_responses):
            local_idx = min(i + 1, num_local - 1)  # Skip L0 (starting state)
            responses.append([local_states[local_idx], response_names[i]])
        
        # Generate transition patterns
        transitions = []
        
        # Pattern 1: Classic intermediate state observation
        if num_global >= 2 and num_local >= 3 and num_transitions >= 4:
            # State modification path: L0,G0 -> Lx,G1 -> L0,G0
            mid_local = local_states[min(2, num_local - 1)]
            transitions.append([local_states[0], global_states[0], mid_local, global_states[1]])
            transitions.append([mid_local, global_states[1], local_states[0], global_states[0]])
            
            # Observation transitions: different responses for different global states
            # L0,G0 -> L1,G0 (saw initial state)
            transitions.append([local_states[0], global_states[0], local_states[1], global_states[0]])
            
            # L0,G1 -> L2,G1 (saw intermediate state) - this is key!
            if num_local >= 3 and num_responses >= 2:
                transitions.append([local_states[0], global_states[1], local_states[2], global_states[1]])
        
        # Fill remaining transitions
        while len(transitions) < num_transitions:
            # Add self-loops or other transitions
            i = len(transitions) % num_global
            j = len(transitions) % num_local
            transitions.append([
                local_states[j], global_states[i],
                local_states[(j + 1) % num_local], global_states[(i + 1) % num_global]
            ])
        
        yield {
            "initial_global": global_states[0],
            "requests": requests,
            "responses": responses,
            "transitions": transitions[:num_transitions]
        }
    
    # Two requests pattern
    elif num_requests == 2 and num_responses >= 2:
        requests = [
            [request_names[0], local_states[0]],
            [request_names[1], local_states[min(1, num_local - 1)]]
        ]
        
        responses = []
        for i in range(num_responses):
            local_idx = min(i, num_local - 1)
            responses.append([local_states[local_idx], response_names[i]])
        
        # Simple interference pattern
        transitions = []
        if num_transitions >= 4:
            # Request 0 modifies state
            transitions.append([local_states[0], global_states[0], 
                              local_states[min(1, num_local-1)], global_states[min(1, num_global-1)]])
            # Request 1 observes state
            transitions.append([local_states[min(1, num_local-1)], global_states[0],
                              local_states[0], global_states[0]])
            transitions.append([local_states[min(1, num_local-1)], global_states[min(1, num_global-1)],
                              local_states[min(2, num_local-1)], global_states[min(1, num_global-1)]])
            # Return transition
            transitions.append([local_states[min(1, num_local-1)], global_states[min(1, num_global-1)],
                              local_states[0], global_states[0]])
        
        yield {
            "initial_global": global_states[0],
            "requests": requests,
            "responses": responses,
            "transitions": transitions[:num_transitions]
        }

def main():
    output_dir = os.path.dirname(os.path.abspath(__file__))
    
    print("Searching for non-serializable Network Systems...\n")
    
    stats = defaultdict(lambda: {"total": 0, "nonser": 0})
    all_nonser = []
    
    count = 0
    for ns in generate_ns_variations(max_size=15, max_states=4, max_requests=2):
        count += 1
        
        # Compute size
        size = (len(set([ns['initial_global']] + [t[1] for t in ns['transitions']] + [t[3] for t in ns['transitions']])) +
               len(set([r[1] for r in ns['requests']] + [r[0] for r in ns['responses']] + 
                      [t[0] for t in ns['transitions']] + [t[2] for t in ns['transitions']])) +
               len(ns['requests']) + len(ns['responses']) + len(ns['transitions']))
        
        # Save and test
        filename = f"test_ns_{count:04d}.json"
        filepath = os.path.join(output_dir, filename)
        
        with open(filepath, 'w') as f:
            json.dump(ns, f, indent=2)
        
        is_ser = run_serializability_checker(filepath)
        
        stats[size]["total"] += 1
        
        if is_ser is False:
            stats[size]["nonser"] += 1
            all_nonser.append((size, ns, filename))
            
            # Rename to indicate non-serializable
            new_filename = f"found_nonser_size{size:02d}_{stats[size]['nonser']:03d}.json"
            new_filepath = os.path.join(output_dir, new_filename)
            os.rename(filepath, new_filepath)
            
            print(f"✅ Found non-serializable NS! Size: {size}, Saved as: {new_filename}")
        else:
            # Remove serializable ones
            os.remove(filepath)
        
        if count % 100 == 0:
            print(f"Tested {count} configurations...")
    
    # Print summary
    print(f"\n{'='*60}")
    print("SUMMARY")
    print(f"{'='*60}")
    print(f"Total NS tested: {count}")
    print(f"Total non-serializable found: {len(all_nonser)}")
    
    print("\nBreakdown by size:")
    for size in sorted(stats.keys()):
        s = stats[size]
        if s["nonser"] > 0:
            print(f"  Size {size}: {s['nonser']}/{s['total']} non-serializable ({s['nonser']/s['total']*100:.1f}%)")
    
    if all_nonser:
        min_size = min(size for size, _, _ in all_nonser)
        print(f"\nMinimal non-serializable size: {min_size}")
        
        # Show minimal examples
        print("\nMinimal non-serializable examples:")
        for size, ns, filename in sorted(all_nonser)[:3]:
            if size == min_size:
                print(f"\n{filename} (size {size}):")
                print(f"  Global states: {len(set([ns['initial_global']] + [t[1] for t in ns['transitions']] + [t[3] for t in ns['transitions']]))}")
                print(f"  Requests: {len(ns['requests'])}")
                print(f"  Responses: {len(ns['responses'])}")
                print(f"  Transitions: {len(ns['transitions'])}")

if __name__ == "__main__":
    main()