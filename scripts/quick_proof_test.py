#!/usr/bin/env python3
import subprocess
import re

# Test just a few specific examples
test_files = [
    "examples/json/small/size06_ns0007_ser.json",
    "examples/json/small/size06_ns0021_ser.json", 
    "examples/json/small/size08_ns0069_ser.json",
    "examples/json/small/pattern_minimal_nonser.json"
]

for i, file_path in enumerate(test_files):
    print(f"\n{'='*60}")
    print(f"[{i+1}/{len(test_files)}] Testing: {file_path}")
    print('='*60)
    
    try:
        result = subprocess.run(
            ["cargo", "run", "--", file_path],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Extract key information
        output = result.stdout + "\n" + result.stderr
        
        # Check result
        if "✅ The network system IS serializable" in output:
            print("Result: SERIALIZABLE")
        elif "❌ The network system is NOT serializable" in output:
            print("Result: NOT SERIALIZABLE")
        else:
            print("Result: UNKNOWN")
        
        # Check proof status
        if "✅ Proof certificate is VALID" in output:
            print("Proof: VALID")
        elif "❌ Proof certificate is INVALID" in output:
            print("Proof: INVALID")
            
            # Find reason
            if "Invariant for global state" in output and "does not imply serializability" in output:
                print("Issue: Invariant contains values outside serializable set")
                
                # Extract details
                outside_match = re.search(r"Values outside serializable set: ([^\n]+)", output)
                if outside_match:
                    print(f"Details: {outside_match.group(1)}")
        
        # Check for errors
        if "thread 'main' panicked" in result.stderr:
            panic_match = re.search(r"panicked at (.+)", result.stderr)
            if panic_match:
                print(f"ERROR: Panic at {panic_match.group(1)}")
        
        if "Failed to parse proof certificate" in output:
            parse_match = re.search(r'ParseError \{ message: "([^"]+)"', output)
            if parse_match:
                print(f"ERROR: Parse error - {parse_match.group(1)}")
                
    except subprocess.TimeoutExpired:
        print("ERROR: TIMEOUT")
    except Exception as e:
        print(f"ERROR: {str(e)}")