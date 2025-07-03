#!/usr/bin/env python3
import subprocess
import re
import sys

# Test specific examples that showed issues
test_files = [
    "examples/json/small/size06_ns0007_ser.json",
    "examples/json/small/size06_ns0021_ser.json",
    "examples/json/small/size06_ns0022_ser.json",
    "examples/json/small/size08_ns0069_ser.json",
    "examples/json/small/pattern_minimal_nonser.json",
    "examples/json/small/pattern_write_skew_ser.json"
]

def extract_proof_info(output):
    """Extract key information about proof verification"""
    info = {
        "serializable": None,
        "proof_valid": None,
        "error": None,
        "invariant_issue": None
    }
    
    if "✅ The network system IS serializable" in output:
        info["serializable"] = True
    elif "❌ The network system is NOT serializable" in output:
        info["serializable"] = False
    
    if "✅ Proof certificate is VALID" in output:
        info["proof_valid"] = True
    elif "❌ Proof certificate is INVALID" in output:
        info["proof_valid"] = False
        
        # Look for specific error
        if "Invariant for global state" in output and "does not imply serializability" in output:
            info["invariant_issue"] = "Invariant contains values outside serializable set"
        elif "Initial state does not satisfy the invariant" in output:
            info["invariant_issue"] = "Initial state check failed"
    
    # Check for panic
    if "thread 'main' panicked" in output:
        panic_match = re.search(r"panicked at (.+)", output)
        if panic_match:
            info["error"] = f"Panic: {panic_match.group(1)}"
    
    # Check for parse errors
    if "Failed to parse proof certificate" in output:
        parse_match = re.search(r'ParseError \{ message: "([^"]+)"', output)
        if parse_match:
            info["error"] = f"Parse error: {parse_match.group(1)}"
    
    return info

def run_test(file_path):
    """Run the serializability checker and extract results"""
    try:
        result = subprocess.run(
            ["cargo", "run", "--", file_path],
            capture_output=True,
            text=True,
            timeout=2
        )
        
        full_output = result.stdout + "\n" + result.stderr
        info = extract_proof_info(full_output)
        info["returncode"] = result.returncode
        
        return info, full_output
        
    except subprocess.TimeoutExpired:
        return {"error": "TIMEOUT", "returncode": -1}, ""
    except Exception as e:
        return {"error": str(e), "returncode": -1}, ""

def main():
    print("Testing specific examples with proof generation issues...")
    print("=" * 80)
    
    for file_path in test_files:
        print(f"\nTesting: {file_path}")
        print("-" * 40)
        
        info, output = run_test(file_path)
        
        print(f"Serializable: {info.get('serializable', 'Unknown')}")
        print(f"Proof Valid: {info.get('proof_valid', 'Unknown')}")
        print(f"Return Code: {info.get('returncode', 'Unknown')}")
        
        if info.get("error"):
            print(f"Error: {info['error']}")
        
        if info.get("invariant_issue"):
            print(f"Invariant Issue: {info['invariant_issue']}")
        
        # For problematic cases, show relevant output
        if info.get("proof_valid") is False or info.get("error"):
            print("\nRelevant output:")
            
            # Find and print the invariant issue details
            lines = output.split('\n')
            in_verification = False
            for i, line in enumerate(lines):
                if "Proof Certificate Verification:" in line:
                    in_verification = True
                elif in_verification and ("=" * 20 in line or "─" * 20 in line):
                    break
                elif in_verification:
                    print(f"  {line}")

if __name__ == "__main__":
    main()