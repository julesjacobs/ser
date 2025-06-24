#!/usr/bin/env python3
import subprocess
import json
import os
import re
from pathlib import Path
from collections import defaultdict

def run_single_example(json_file, timeout=5):
    """Run a single example with timeout"""
    try:
        result = subprocess.run(
            ["cargo", "run", "--", json_file],
            capture_output=True,
            text=True,
            timeout=timeout
        )
        return result.stdout, result.stderr, result.returncode
    except subprocess.TimeoutExpired:
        return "", "TIMEOUT", -1
    except Exception as e:
        return "", f"ERROR: {str(e)}", -1

def analyze_output(stdout, stderr):
    """Analyze output to categorize the issue"""
    issue = {
        "category": None,
        "details": None,
        "serializable": None,
        "proof_status": None
    }
    
    # Check serializability result
    if "✅ The network system IS serializable" in stdout:
        issue["serializable"] = True
    elif "❌ The network system is NOT serializable" in stdout:
        issue["serializable"] = False
    
    # Check for timeout
    if "TIMEOUT" in stderr:
        issue["category"] = "timeout"
        return issue
    
    # Check for panic
    if "thread 'main' panicked" in stderr:
        panic_match = re.search(r"panicked at (.+)", stderr)
        if panic_match:
            panic_location = panic_match.group(1)
            if "Variable not found in mapping" in stderr:
                issue["category"] = "panic_variable_not_found"
                issue["details"] = panic_location
            else:
                issue["category"] = "panic_other"
                issue["details"] = panic_location
        return issue
    
    # Check proof status
    if "✅ Proof certificate is VALID" in stdout:
        issue["proof_status"] = "valid"
        issue["category"] = "ok"
    elif "❌ Proof certificate is INVALID" in stdout:
        issue["proof_status"] = "invalid"
        
        # Find specific issue
        if "Invariant for global state" in stdout and "does not imply serializability" in stdout:
            issue["category"] = "invalid_proof_invariant_outside_serializable"
            # Extract which values are outside
            outside_match = re.search(r"Values outside serializable set: ([^\n]+)", stdout)
            if outside_match:
                issue["details"] = outside_match.group(1)
        elif "Initial state does not satisfy the invariant" in stdout:
            issue["category"] = "invalid_proof_initial_state"
        else:
            issue["category"] = "invalid_proof_other"
    elif "Failed to parse proof certificate" in stdout:
        parse_match = re.search(r'ParseError \{ message: "([^"]+)"', stdout)
        if parse_match:
            issue["category"] = "proof_parse_error"
            issue["details"] = parse_match.group(1)
    elif issue["serializable"] is not None:
        # Result determined but no proof verification
        issue["category"] = "no_proof_verification"
    else:
        issue["category"] = "unknown"
    
    return issue

def main():
    json_dir = Path("examples/json/small")
    json_files = sorted(json_dir.glob("*.json"))
    
    issues_by_category = defaultdict(list)
    total_files = len(json_files)
    
    print(f"Analyzing {total_files} JSON files for proof generation issues...")
    print("=" * 80)
    
    for i, json_file in enumerate(json_files):
        if (i + 1) % 10 == 0:
            print(f"Progress: {i+1}/{total_files} files analyzed...")
        
        stdout, stderr, returncode = run_single_example(str(json_file))
        issue = analyze_output(stdout, stderr)
        
        issues_by_category[issue["category"]].append({
            "file": json_file.name,
            "issue": issue,
            "returncode": returncode
        })
    
    # Print summary
    print("\n" + "=" * 80)
    print("SUMMARY BY ISSUE CATEGORY")
    print("=" * 80)
    
    categories_ordered = [
        ("ok", "✅ Correct (proof valid)"),
        ("invalid_proof_invariant_outside_serializable", "❌ Invalid proof: invariant outside serializable set"),
        ("invalid_proof_initial_state", "❌ Invalid proof: initial state check failed"),
        ("invalid_proof_other", "❌ Invalid proof: other reason"),
        ("panic_variable_not_found", "💥 Panic: Variable not found in mapping"),
        ("panic_other", "💥 Panic: other reason"),
        ("proof_parse_error", "⚠️  Proof parse error"),
        ("no_proof_verification", "⚠️  No proof verification performed"),
        ("timeout", "⏱️  Timeout"),
        ("unknown", "❓ Unknown issue")
    ]
    
    for category, description in categories_ordered:
        files = issues_by_category.get(category, [])
        if files:
            print(f"\n{description}: {len(files)} files")
            print("-" * 40)
            
            # Show first few examples
            for entry in files[:5]:
                print(f"  • {entry['file']}")
                if entry['issue'].get('details'):
                    print(f"    Details: {entry['issue']['details']}")
            
            if len(files) > 5:
                print(f"  ... and {len(files) - 5} more")
    
    # Special analysis for invalid proofs
    invalid_proofs = issues_by_category.get("invalid_proof_invariant_outside_serializable", [])
    if invalid_proofs:
        print("\n" + "=" * 80)
        print("DETAILED ANALYSIS: Invalid Proofs (Invariant Outside Serializable Set)")
        print("=" * 80)
        
        # Group by the details (what values are outside)
        by_details = defaultdict(list)
        for entry in invalid_proofs:
            details = entry['issue'].get('details', 'Unknown')
            by_details[details].append(entry['file'])
        
        for details, files in by_details.items():
            print(f"\n{details}:")
            for f in files[:3]:
                print(f"  - {f}")
            if len(files) > 3:
                print(f"  ... and {len(files) - 3} more")
    
    # Save detailed results
    all_issues = []
    for category, entries in issues_by_category.items():
        for entry in entries:
            all_issues.append({
                "file": entry["file"],
                "category": category,
                "serializable": entry["issue"]["serializable"],
                "proof_status": entry["issue"]["proof_status"],
                "details": entry["issue"].get("details"),
                "returncode": entry["returncode"]
            })
    
    with open("proof_issues_summary.json", "w") as f:
        json.dump(all_issues, f, indent=2)
    
    print(f"\n\nDetailed results saved to proof_issues_summary.json")
    
    # Print final statistics
    total_ok = len(issues_by_category.get("ok", []))
    total_issues = total_files - total_ok
    
    print(f"\nFINAL STATISTICS:")
    print(f"  Total files: {total_files}")
    print(f"  OK (valid proofs): {total_ok} ({total_ok/total_files*100:.1f}%)")
    print(f"  Issues found: {total_issues} ({total_issues/total_files*100:.1f}%)")

if __name__ == "__main__":
    main()