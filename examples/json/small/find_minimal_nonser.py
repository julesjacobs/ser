#!/usr/bin/env python3
"""
Find the minimal non-serializable Network System.

This script systematically searches for the smallest NS that is non-serializable.
"""

import json
import os
import subprocess

def test_ns(ns, filename="test_ns.json"):
    """Test if an NS is serializable."""
    try:
        # Save NS to file
        with open(filename, 'w') as f:
            json.dump(ns, f, indent=2)
        
        # Run checker
        script_dir = os.path.dirname(os.path.abspath(__file__))
        project_root = os.path.dirname(os.path.dirname(os.path.dirname(script_dir)))
        
        result = subprocess.run(
            ['cargo', 'run', '--', filename],
            cwd=project_root,
            capture_output=True,
            text=True,
            timeout=10
        )
        
        output = result.stdout + result.stderr
        
        if result.returncode != 0:
            return None, output
        
        # Check result
        if "CounterExample (Not serializable)" in output or "NOT SERIALIZABLE" in output:
            return False, output
        elif "Proof (Serializable)" in output or "SERIALIZABLE" in output:
            return True, output
        else:
            return None, output
            
    except Exception as e:
        return None, str(e)
    finally:
        # Clean up
        if os.path.exists(filename):
            os.remove(filename)

def compute_size(ns):
    """Compute the size of an NS."""
    num_global = len(set([ns["initial_global"]] + 
                        [t[1] for t in ns["transitions"]] + 
                        [t[3] for t in ns["transitions"]]))
    num_local = len(set([r[1] for r in ns["requests"]] + 
                       [r[0] for r in ns["responses"]] +
                       [t[0] for t in ns["transitions"]] + 
                       [t[2] for t in ns["transitions"]]))
    return num_global + num_local + len(ns["requests"]) + len(ns["responses"]) + len(ns["transitions"])

# Known minimal non-serializable pattern
minimal_candidates = [
    # Pattern 1: Simple observation of intermediate state
    {
        "initial_global": "G0",
        "requests": [
            ["writer", "L0"],
            ["reader", "L0"]
        ],
        "responses": [
            ["L1", "done"],
            ["L1", "saw_G0"],
            ["L2", "saw_G1"]
        ],
        "transitions": [
            # Writer: change state and back
            ["L0", "G0", "L2", "G1"],
            ["L2", "G1", "L1", "G0"],
            # Reader: observe different states
            ["L0", "G0", "L1", "G0"],
            ["L0", "G1", "L2", "G1"]
        ]
    },
    
    # Pattern 2: Even more minimal
    {
        "initial_global": "A",
        "requests": [
            ["P", "s"],
            ["Q", "s"]
        ],
        "responses": [
            ["t", "done"],
            ["t", "sawA"],
            ["u", "sawB"]
        ],
        "transitions": [
            ["s", "A", "u", "B"],
            ["u", "B", "t", "A"],
            ["s", "A", "t", "A"],
            ["s", "B", "u", "B"]
        ]
    },
    
    # Pattern 3: Absolutely minimal attempt
    {
        "initial_global": "X",
        "requests": [
            ["P", "s"],
            ["Q", "s"]
        ],
        "responses": [
            ["t", "a"],
            ["u", "b"]
        ],
        "transitions": [
            ["s", "X", "t", "Y"],
            ["t", "Y", "s", "X"],
            ["s", "X", "s", "X"],
            ["s", "Y", "u", "Y"]
        ]
    }
]

def main():
    print("Searching for minimal non-serializable Network Systems...\n")
    
    found_nonser = []
    
    for i, ns in enumerate(minimal_candidates):
        size = compute_size(ns)
        print(f"Testing candidate {i+1} (size={size})...")
        
        is_ser, output = test_ns(ns)
        
        if is_ser is False:
            print(f"  ✅ NON-SERIALIZABLE!")
            found_nonser.append((size, ns))
            
            # Save it
            filename = f"minimal_nonser_{i+1}_size{size}.json"
            with open(filename, 'w') as f:
                json.dump(ns, f, indent=2)
            print(f"  Saved as {filename}")
            
        elif is_ser is True:
            print(f"  ❌ Serializable")
        else:
            print(f"  ⚠️  Error or unknown")
    
    if found_nonser:
        print(f"\nFound {len(found_nonser)} non-serializable NS!")
        min_size = min(size for size, _ in found_nonser)
        print(f"Minimal size: {min_size}")
        
        for size, ns in sorted(found_nonser):
            if size == min_size:
                print(f"\nMinimal non-serializable NS (size {size}):")
                print(json.dumps(ns, indent=2))
                break
    else:
        print("\nNo non-serializable NS found in candidates.")
        print("Try adding more patterns!")

if __name__ == "__main__":
    main()