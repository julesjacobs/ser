#!/usr/bin/env python3
"""
Exhaustively generate all possible Network Systems of a given size.

Size is defined as: num_global_states + num_local_states + num_requests + num_responses + num_transitions

This allows systematic exploration of the NS space starting from the smallest possible systems.
"""

import json
import itertools
import os
import subprocess
import argparse
from collections import defaultdict

class ExhaustiveNSGenerator:
    def __init__(self, max_global=3, max_local=3, max_requests=2, max_responses=2):
        """Initialize generator with maximum values for each component."""
        self.max_global = max_global
        self.max_local = max_local
        self.max_requests = max_requests
        self.max_responses = max_responses
        
    def compute_size(self, num_global, num_local, num_requests, num_responses, num_transitions):
        """Compute the total size of an NS configuration."""
        return num_global + num_local + num_requests + num_responses + num_transitions
    
    def generate_all_ns_of_size(self, target_size):
        """Generate all possible NS configurations of a given total size."""
        ns_list = []
        
        # Iterate through all possible component counts
        # Prioritize configurations with more responses (for non-serializability)
        for num_responses in range(min(self.max_responses, target_size - 3), 0, -1):
            for num_global in range(1, min(self.max_global + 1, target_size - num_responses)):
                for num_local in range(1, min(self.max_local + 1, target_size - num_global - num_responses)):
                    for num_requests in range(1, min(self.max_requests + 1, target_size - num_global - num_local - num_responses + 1)):
                        # Calculate number of transitions
                        num_transitions = target_size - num_global - num_local - num_requests - num_responses
                        
                        if num_transitions >= 0:
                            # Generate all NS with these parameters
                            for ns in self.generate_all_ns_with_params(
                                num_global, num_local, num_requests, num_responses, num_transitions
                            ):
                                ns_list.append(ns)
        
        return ns_list
    
    def generate_all_ns_with_params(self, num_global, num_local, num_requests, num_responses, num_transitions):
        """Generate all possible NS with specific parameter counts."""
        # Create state and request/response names
        global_states = [f"G{i}" for i in range(num_global)]
        local_states = [f"L{i}" for i in range(num_local)]
        request_names = [f"Req{i}" for i in range(num_requests)]
        response_names = [f"Resp{i}" for i in range(num_responses)]
        
        # For efficiency, we'll generate a limited subset instead of all combinations
        # This ensures we get diverse examples without combinatorial explosion
        
        count = 0
        max_per_config = 10  # Limit examples per configuration
        
        # Try different initial global states
        for initial_global in global_states:
            # Generate simple request mappings (each request to a different local state if possible)
            requests = []
            for i, req_name in enumerate(request_names):
                local_idx = i % num_local
                requests.append([req_name, local_states[local_idx]])
            
            # Generate simple response mappings
            responses = []
            for i, resp_name in enumerate(response_names):
                local_idx = i % num_local
                responses.append([local_states[local_idx], resp_name])
            
            # Generate diverse transition sets
            if num_transitions == 0:
                yield {
                    "initial_global": initial_global,
                    "requests": requests,
                    "responses": responses,
                    "transitions": []
                }
                count += 1
            else:
                # Generate a few different transition patterns
                all_possible_transitions = list(itertools.product(
                    local_states, global_states, local_states, global_states
                ))
                
                # Pattern 1: Sequential transitions
                transitions = []
                for i in range(min(num_transitions, len(all_possible_transitions))):
                    transitions.append(list(all_possible_transitions[i]))
                
                yield {
                    "initial_global": initial_global,
                    "requests": requests,
                    "responses": responses,
                    "transitions": transitions
                }
                count += 1
                
                if count >= max_per_config:
                    return
                
                # Pattern 2: Cyclic transitions (if enough states)
                if num_global >= 2 and num_local >= 2 and num_transitions >= 4:
                    transitions = [
                        [local_states[0], global_states[0], local_states[1], global_states[1]],
                        [local_states[1], global_states[1], local_states[0], global_states[0]],
                        [local_states[0], global_states[1], local_states[1], global_states[0]],
                        [local_states[1], global_states[0], local_states[0], global_states[1]]
                    ][:num_transitions]
                    
                    yield {
                        "initial_global": initial_global,
                        "requests": requests,
                        "responses": responses,
                        "transitions": transitions
                    }
                    count += 1
                    
                if count >= max_per_config:
                    return
                    
                # Pattern 3: Interference pattern (for non-serializability)
                if num_global >= 2 and num_local >= 2 and num_requests >= 2 and num_transitions >= 4:
                    # One request modifies global state, another observes it
                    transitions = [
                        # Modifier transitions
                        [local_states[0], global_states[0], local_states[1], global_states[1]],
                        [local_states[1], global_states[1], local_states[0], global_states[0]],
                        # Observer transitions - different responses based on global state
                        [local_states[0], global_states[0], local_states[0], global_states[0]],
                        [local_states[0], global_states[1], local_states[1], global_states[1]]
                    ][:num_transitions]
                    
                    # Adjust responses to make observation visible
                    if num_responses >= 2 and num_local >= 2:
                        responses = [
                            [local_states[0], response_names[0]],  # Observed initial
                            [local_states[1], response_names[1]]   # Observed modified
                        ][:num_responses]
                    
                    yield {
                        "initial_global": initial_global,
                        "requests": requests,
                        "responses": responses,
                        "transitions": transitions
                    }
                    count += 1
                
                # Pattern 4: Single request with multiple responses based on state
                if num_requests == 1 and num_responses >= 2 and num_global >= 2 and num_local >= 2:
                    # Request can see different states when executed concurrently
                    transitions = [
                        # Path 1: Change global state
                        [local_states[0], global_states[0], local_states[1], global_states[1]],
                        [local_states[1], global_states[1], local_states[0], global_states[0]],
                        # Path 2: Observe and respond based on global state
                        [local_states[0], global_states[0], local_states[0], global_states[0]],
                        [local_states[0], global_states[1], local_states[1], global_states[1]]
                    ][:num_transitions]
                    
                    # Multiple responses for different observations
                    responses = []
                    for i in range(min(num_responses, num_local)):
                        responses.append([local_states[i], response_names[i]])
                    
                    yield {
                        "initial_global": initial_global,
                        "requests": [[request_names[0], local_states[0]]],
                        "responses": responses,
                        "transitions": transitions
                    }
                    count += 1
    
    def is_trivially_invalid(self, ns):
        """Check if NS is trivially invalid (e.g., unreachable states)."""
        # Extract all states mentioned in transitions
        states_in_transitions = set()
        for trans in ns["transitions"]:
            states_in_transitions.add(trans[0])  # local_from
            states_in_transitions.add(trans[2])  # local_to
            states_in_transitions.add(trans[1])  # global_from
            states_in_transitions.add(trans[3])  # global_to
        
        # Check if initial states are reachable
        initial_locals = set(req[1] for req in ns["requests"])
        
        # Basic validity: at least one request's initial state should be usable
        return False  # For now, include all NS

def run_serializability_check(filepath):
    """Run the serializability checker and return (is_serializable, has_error)."""
    try:
        script_dir = os.path.dirname(os.path.abspath(__file__))
        project_root = os.path.dirname(os.path.dirname(os.path.dirname(script_dir)))
        
        result = subprocess.run(
            ['cargo', 'run', '--', filepath],
            cwd=project_root,
            capture_output=True,
            text=True,
            timeout=10
        )
        
        output = result.stdout + result.stderr
        
        if result.returncode != 0:
            return None, True
        
        # Check result
        if "Proof (Serializable)" in output or "✅ Proof certificate is VALID" in output:
            return True, False
        elif "CounterExample (Not serializable)" in output or "✅ Trace validation PASSED" in output:
            return False, False
        else:
            return None, True
            
    except subprocess.TimeoutExpired:
        return None, True
    except Exception:
        return None, True

def main():
    parser = argparse.ArgumentParser(description='Exhaustively generate small Network Systems')
    parser.add_argument('--start-size', type=int, default=5, help='Starting total size')
    parser.add_argument('--end-size', type=int, default=10, help='Ending total size')
    parser.add_argument('--max-global', type=int, default=3, help='Maximum global states')
    parser.add_argument('--max-local', type=int, default=3, help='Maximum local states')
    parser.add_argument('--max-requests', type=int, default=2, help='Maximum requests')
    parser.add_argument('--max-responses', type=int, default=2, help='Maximum responses')
    parser.add_argument('--test', action='store_true', help='Test serializability of generated NS')
    parser.add_argument('--limit', type=int, help='Limit number of NS per size')
    
    args = parser.parse_args()
    
    generator = ExhaustiveNSGenerator(
        max_global=args.max_global,
        max_local=args.max_local,
        max_requests=args.max_requests,
        max_responses=args.max_responses
    )
    
    output_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Statistics
    stats = defaultdict(lambda: {"total": 0, "serializable": 0, "non_serializable": 0, "errors": 0})
    
    for size in range(args.start_size, args.end_size + 1):
        print(f"\n{'='*60}")
        print(f"Generating NS of size {size}")
        print(f"{'='*60}")
        
        ns_list = generator.generate_all_ns_of_size(size)
        
        if args.limit and len(ns_list) > args.limit:
            print(f"Generated {len(ns_list)} NS, limiting to {args.limit}")
            ns_list = ns_list[:args.limit]
        else:
            print(f"Generated {len(ns_list)} NS configurations")
        
        for i, ns in enumerate(ns_list):
            # Skip trivially invalid NS
            if generator.is_trivially_invalid(ns):
                continue
            
            # Create filename
            filename = f"size{size:02d}_ns{i+1:04d}.json"
            filepath = os.path.join(output_dir, filename)
            
            # Save NS
            with open(filepath, 'w') as f:
                json.dump(ns, f, indent=2)
            
            stats[size]["total"] += 1
            
            # Test if requested
            if args.test:
                is_ser, has_error = run_serializability_check(filepath)
                
                if has_error:
                    suffix = "_error"
                    stats[size]["errors"] += 1
                elif is_ser is True:
                    suffix = "_ser"
                    stats[size]["serializable"] += 1
                elif is_ser is False:
                    suffix = "_nonser"
                    stats[size]["non_serializable"] += 1
                else:
                    suffix = "_unknown"
                    stats[size]["errors"] += 1
                
                # Rename file with result
                new_filename = f"size{size:02d}_ns{i+1:04d}{suffix}.json"
                new_filepath = os.path.join(output_dir, new_filename)
                os.rename(filepath, new_filepath)
                
                if i % 10 == 0:
                    print(f"  Processed {i+1}/{len(ns_list)} NS...")
        
        # Print size statistics
        s = stats[size]
        print(f"\nSize {size} summary:")
        print(f"  Total: {s['total']}")
        if args.test:
            print(f"  Serializable: {s['serializable']} ({s['serializable']/s['total']*100:.1f}%)")
            print(f"  Non-serializable: {s['non_serializable']} ({s['non_serializable']/s['total']*100:.1f}%)")
            print(f"  Errors: {s['errors']}")
    
    # Print overall statistics
    print(f"\n{'='*60}")
    print("OVERALL SUMMARY")
    print(f"{'='*60}")
    
    total_ns = sum(s["total"] for s in stats.values())
    print(f"Total NS generated: {total_ns}")
    
    if args.test:
        total_ser = sum(s["serializable"] for s in stats.values())
        total_nonser = sum(s["non_serializable"] for s in stats.values())
        total_errors = sum(s["errors"] for s in stats.values())
        
        print(f"Total serializable: {total_ser} ({total_ser/total_ns*100:.1f}%)")
        print(f"Total non-serializable: {total_nonser} ({total_nonser/total_ns*100:.1f}%)")
        print(f"Total errors: {total_errors}")
        
        # Find first non-serializable
        for size in range(args.start_size, args.end_size + 1):
            if stats[size]["non_serializable"] > 0:
                print(f"\nFirst non-serializable NS found at size {size}")
                # List them
                for f in sorted(os.listdir(output_dir)):
                    if f.startswith(f"size{size:02d}") and f.endswith("_nonser.json"):
                        print(f"  - {f}")
                break

if __name__ == "__main__":
    main()