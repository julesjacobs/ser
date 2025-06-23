#!/usr/bin/env python3
"""
Random Network System (NS) Generator

Generates random Network System JSON files for testing the serializability checker.
"""

import json
import random
import argparse
import os
import subprocess
import tempfile
import shutil

class NSGenerator:
    def __init__(self, num_global_states: int, num_local_states: int, 
                 num_requests: int, num_responses: int, num_transitions: int,
                 include_empty_state: bool = True):
        self.num_global_states = num_global_states
        self.num_local_states = num_local_states
        self.num_requests = num_requests
        self.num_responses = num_responses
        self.num_transitions = num_transitions
        self.include_empty_state = include_empty_state
        
        # Generate state names
        self.global_states = [f"G{i}" for i in range(num_global_states)]
        self.local_states = [f"L{i}" for i in range(num_local_states)]
        
        # Include empty state with low probability
        if include_empty_state and random.random() < 0.2:
            self.global_states.append("_")
            self.local_states.append("_")
        
        # Generate request and response names
        self.request_names = [f"Req{i}" for i in range(num_requests)]
        self.response_names = [f"Resp{i}" for i in range(num_responses)]
        
    def generate_ns(self) -> dict:
        """Generate a random Network System"""
        # Choose initial global state
        initial_global = random.choice(self.global_states)
        
        # Generate requests: [request_name, initial_local_state]
        requests = []
        for req_name in self.request_names:
            local_state = random.choice(self.local_states)
            requests.append([req_name, local_state])
        
        # Generate responses: [local_state, response_name]
        responses = []
        for resp_name in self.response_names:
            local_state = random.choice(self.local_states)
            responses.append([local_state, resp_name])
        
        # Generate transitions: [local_from, global_from, local_to, global_to]
        transitions = []
        seen_transitions = set()
        
        for _ in range(self.num_transitions):
            # Try to generate unique transitions
            attempts = 0
            while attempts < 100:
                local_from = random.choice(self.local_states)
                global_from = random.choice(self.global_states)
                local_to = random.choice(self.local_states)
                global_to = random.choice(self.global_states)
                
                transition = (local_from, global_from, local_to, global_to)
                if transition not in seen_transitions:
                    seen_transitions.add(transition)
                    transitions.append(list(transition))
                    break
                attempts += 1
        
        return {
            "initial_global": initial_global,
            "requests": requests,
            "responses": responses,
            "transitions": transitions
        }
    
    def generate_nonser_pattern(self) -> dict:
        """Generate NS with patterns likely to be non-serializable.
        
        The key insight: non-serializable systems allow concurrent executions
        to produce outputs that no sequential execution can produce.
        """
        # We need at least 2 requests to have interference
        if self.num_requests < 2:
            self.num_requests = 2
            self.request_names = [f"Req{i}" for i in range(self.num_requests)]
        
        # Ensure we have enough states
        if self.num_global_states < 2:
            self.num_global_states = 2
            self.global_states = [f"G{i}" for i in range(self.num_global_states)]
        
        if self.num_local_states < 3:
            self.num_local_states = 3
            self.local_states = [f"L{i}" for i in range(self.num_local_states)]
        
        # Pattern: State observation interference
        # One request modifies state, another observes it
        # Concurrent execution can see intermediate states
        
        initial_global = self.global_states[0]
        
        # Two interfering requests
        requests = [
            [self.request_names[0], self.local_states[0]],  # Modifier
            [self.request_names[1], self.local_states[0]]   # Observer
        ]
        
        # Responses that reveal the observed state
        responses = []
        if self.num_responses >= 3:
            # Different responses for different observations
            responses.append([self.local_states[1], self.response_names[0]])  # Saw initial
            responses.append([self.local_states[2], self.response_names[1]])  # Saw modified
            responses.append([self.local_states[1], self.response_names[2]])  # Modified done
        else:
            # Minimum responses
            for i in range(self.num_responses):
                responses.append([self.local_states[min(i+1, len(self.local_states)-1)], 
                                self.response_names[i]])
        
        transitions = []
        
        # Modifier request: changes global state
        transitions.append([
            self.local_states[0], self.global_states[0],  # Start at G0
            self.local_states[2], self.global_states[1]   # Change to G1
        ])
        
        if len(self.local_states) > 2:
            transitions.append([
                self.local_states[2], self.global_states[1],  # At G1
                self.local_states[1], self.global_states[0]   # Back to G0
            ])
        
        # Observer request: reads different values based on global state
        transitions.append([
            self.local_states[0], self.global_states[0],  # See G0
            self.local_states[1], self.global_states[0]   # Report "initial"
        ])
        
        transitions.append([
            self.local_states[0], self.global_states[1],  # See G1  
            self.local_states[2], self.global_states[1]   # Report "modified"
        ])
        
        # Add a few more transitions to create interesting paths
        if self.num_transitions > len(transitions):
            # Add some cycles or additional paths
            if len(self.global_states) > 2:
                transitions.append([
                    self.local_states[1], self.global_states[1],
                    self.local_states[0], self.global_states[2]
                ])
            
            # Self-loops to allow waiting
            transitions.append([
                self.local_states[0], self.global_states[0],
                self.local_states[0], self.global_states[0]
            ])
        
        # Randomly add more transitions if needed
        while len(transitions) < self.num_transitions:
            local_from = random.choice(self.local_states)
            global_from = random.choice(self.global_states)
            local_to = random.choice(self.local_states)
            global_to = random.choice(self.global_states)
            
            # Avoid exact duplicates
            transition = [local_from, global_from, local_to, global_to]
            if transition not in transitions:
                transitions.append(transition)
        
        return {
            "initial_global": initial_global,
            "requests": requests,
            "responses": responses,
            "transitions": transitions[:self.num_transitions]
        }
    
    def generate_complex_ns(self) -> dict:
        """Generate a more complex NS with patterns similar to real examples"""
        # Use more meaningful state names for complex examples
        if self.num_global_states <= 5:
            global_states = ["Init", "Active", "Processing", "Done", "Error"][:self.num_global_states]
        else:
            global_states = self.global_states
            
        if self.num_local_states <= 6:
            local_states = ["Ready", "Running", "Waiting", "Complete", "Failed", "Idle"][:self.num_local_states]
        else:
            local_states = self.local_states
        
        # Override with meaningful names
        self.global_states = global_states
        self.local_states = local_states
        
        # Generate with some realistic patterns
        ns = self.generate_ns()
        
        # Add some structure - ensure at least one path from initial state
        if ns["transitions"] and ns["requests"]:
            # Add a transition from the first request's local state
            first_req_local = ns["requests"][0][1]
            ns["transitions"].insert(0, [
                first_req_local,
                ns["initial_global"],
                random.choice(self.local_states),
                random.choice(self.global_states)
            ])
        
        return ns

def run_serializability_checker(filepath: str) -> tuple[bool, str, bool, bool | None]:
    """Run the serializability checker on a file.
    
    Returns: (success, output, verification_failed, is_serializable)
        - success: True if the command ran successfully
        - output: The output from the command
        - verification_failed: True if proof/trace verification failed
        - is_serializable: True if serializable, False if not, None if unknown
    """
    try:
        # Get the path to the cargo binary (3 levels up from this script)
        script_dir = os.path.dirname(os.path.abspath(__file__))
        project_root = os.path.dirname(os.path.dirname(os.path.dirname(script_dir)))
        
        # Run the checker
        result = subprocess.run(
            ['cargo', 'run', '--', filepath],
            cwd=project_root,
            capture_output=True,
            text=True,
            timeout=30
        )
        
        output = result.stdout + result.stderr
        
        # Check for verification failures
        verification_failed = False
        if "Proof certificate is INVALID" in output:
            verification_failed = True
        elif "Trace validation FAILED" in output:
            verification_failed = True
        elif "ERROR" in output and "proof" in output.lower():
            verification_failed = True
        elif "ERROR" in output and "trace" in output.lower():
            verification_failed = True
        
        # Determine serializability
        is_serializable = None
        if "Proof (Serializable)" in output or "✅ Proof certificate is VALID" in output:
            is_serializable = True
        elif "Trace (Not serializable)" in output or "✅ Trace validation PASSED" in output:
            is_serializable = False
        # Legacy format support
        elif "Serializable: true" in output:
            is_serializable = True
        elif "Serializable: false" in output:
            is_serializable = False
        
        return result.returncode == 0, output, verification_failed, is_serializable
        
    except subprocess.TimeoutExpired:
        return False, "Timeout", False, None
    except Exception as e:
        return False, str(e), False, None

def main():
    parser = argparse.ArgumentParser(description='Generate random Network System JSON files')
    parser.add_argument('--count', type=int, default=5, help='Number of NS files to generate')
    parser.add_argument('--max-global-states', type=int, default=4, help='Maximum number of global states')
    parser.add_argument('--max-local-states', type=int, default=5, help='Maximum number of local states')
    parser.add_argument('--max-requests', type=int, default=3, help='Maximum number of requests')
    parser.add_argument('--max-responses', type=int, default=3, help='Maximum number of responses')
    parser.add_argument('--max-transitions', type=int, default=10, help='Maximum number of transitions')
    parser.add_argument('--complex', action='store_true', help='Generate more complex/realistic NS')
    parser.add_argument('--nonser-pattern', action='store_true', help='Generate patterns likely to be non-serializable')
    parser.add_argument('--seed', type=int, help='Random seed for reproducibility')
    parser.add_argument('--test', action='store_true', 
                        help='Run serializability checker on generated files')
    parser.add_argument('--filter', choices=['failed', 'serializable', 'non-serializable', 'all'],
                        default='all', help='Filter which files to keep when using --test')
    parser.add_argument('--test-and-filter', action='store_true',
                        help='Shortcut for --test --filter=failed (legacy option)')
    
    args = parser.parse_args()
    
    # Handle legacy --test-and-filter option
    if args.test_and_filter:
        args.test = True
        args.filter = 'failed'
    
    if args.seed is not None:
        random.seed(args.seed)
    
    # Ensure output directory exists
    output_dir = os.path.dirname(os.path.abspath(__file__))
    
    generated_files = []
    verification_failed_files = []
    serializable_files = []
    non_serializable_files = []
    
    for i in range(args.count):
        # Randomize parameters within bounds
        num_global = random.randint(2, args.max_global_states)
        num_local = random.randint(2, args.max_local_states)
        num_requests = random.randint(1, args.max_requests)
        num_responses = random.randint(1, args.max_responses)
        num_transitions = random.randint(1, args.max_transitions)
        
        generator = NSGenerator(
            num_global_states=num_global,
            num_local_states=num_local,
            num_requests=num_requests,
            num_responses=num_responses,
            num_transitions=num_transitions
        )
        
        # Generate NS
        if args.nonser_pattern:
            ns = generator.generate_nonser_pattern()
        elif args.complex:
            ns = generator.generate_complex_ns()
        else:
            ns = generator.generate_ns()
        
        # Save to temporary file first if testing
        if args.test:
            temp_fd, temp_path = tempfile.mkstemp(suffix='.json')
            try:
                with os.fdopen(temp_fd, 'w') as f:
                    json.dump(ns, f, indent=2)
                
                # Run the serializability checker
                success, output, verification_failed, is_serializable = run_serializability_checker(temp_path)
                
                # Determine filename based on serializability
                suffix = ""
                if is_serializable is True:
                    suffix = "_ser"
                    serializable_files.append(f"random_ns_{i+1:03d}{suffix}.json")
                elif is_serializable is False:
                    suffix = "_nonser"
                    non_serializable_files.append(f"random_ns_{i+1:03d}{suffix}.json")
                
                filename = f"random_ns_{i+1:03d}{suffix}.json"
                
                # Determine if we should keep this file
                keep_file = False
                status = ""
                
                if not success:
                    # Command failed to run
                    status = "(checker failed)"
                    keep_file = True  # Keep files that cause checker failures
                elif verification_failed:
                    # Verification failed - this is what we're looking for
                    status = "(VERIFICATION FAILED)"
                    keep_file = True
                    verification_failed_files.append(filename)
                else:
                    # Everything passed - usually we don't want these
                    if "Serializable: true" in output:
                        status = "(serializable, proof valid)"
                    elif "Serializable: false" in output:
                        status = "(not serializable, trace valid)"
                    else:
                        status = "(unknown result)"
                    keep_file = args.filter == 'all'
                
                # Apply additional filtering based on serializability
                if args.filter == 'serializable' and is_serializable is not True:
                    keep_file = False
                elif args.filter == 'non-serializable' and is_serializable is not False:
                    keep_file = False
                
                if keep_file:
                    # Move temp file to final location
                    filepath = os.path.join(output_dir, filename)
                    shutil.move(temp_path, filepath)
                    generated_files.append(filename)
                    
                    # Print summary
                    print(f"Generated {filename}: {status}")
                    print(f"  Global states: {num_global}")
                    print(f"  Local states: {num_local}")
                    print(f"  Requests: {num_requests}")
                    print(f"  Responses: {num_responses}")
                    print(f"  Transitions: {len(ns['transitions'])}")
                    print()
                else:
                    # Delete temp file
                    os.unlink(temp_path)
                    print(f"Skipped (valid verification): G={num_global}, L={num_local}, R={num_requests}/{num_responses}, T={len(ns['transitions'])}")
                    
            finally:
                # Clean up temp file if it still exists
                if os.path.exists(temp_path):
                    os.unlink(temp_path)
        else:
            # Normal mode - just save the file
            filename = f"random_ns_{i+1:03d}.json"
            filepath = os.path.join(output_dir, filename)
            
            with open(filepath, 'w') as f:
                json.dump(ns, f, indent=2)
            
            generated_files.append(filename)
            
            # Print summary
            print(f"Generated {filename}:")
            print(f"  Global states: {num_global}")
            print(f"  Local states: {num_local}")
            print(f"  Requests: {num_requests}")
            print(f"  Responses: {num_responses}")
            print(f"  Transitions: {len(ns['transitions'])}")
            print()
    
    print(f"\nGenerated {len(generated_files)} NS files in {output_dir}")
    
    if args.test:
        print(f"\nResults:")
        print(f"  Serializable: {len(serializable_files)}")
        print(f"  Non-serializable: {len(non_serializable_files)}")
        print(f"  Verification failures: {len(verification_failed_files)}")
        
        if verification_failed_files and args.filter in ['failed', 'all']:
            print("\nVerification failed files:")
            for f in verification_failed_files:
                print(f"  - {f}")

if __name__ == "__main__":
    main()