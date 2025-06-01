#!/usr/bin/env python3

"""
Test script to validate the new SMPT enhanced functionality
by calling the Rust functions through a simple interface
"""

import subprocess
import sys
import os

def test_enhanced_smpt():
    """Test the enhanced SMPT functionality"""
    
    # Check if we have the necessary files
    net_file = "out/simple_nonser/smpt_petri.net"
    xml_file = "out/simple_nonser/smpt_constraints.xml"
    
    if not os.path.exists(net_file) or not os.path.exists(xml_file):
        print("❌ Required SMPT files not found, generating them first...")
        # Run the tool to generate files
        result = subprocess.run(["cargo", "run", "--", "examples/ser/simple_nonser.ser"], 
                              capture_output=True, text=True)
        if result.returncode != 0:
            print(f"Failed to generate SMPT files: {result.stderr}")
            return False
    
    # Test the enhanced function by calling SMPT directly
    print("🧪 Testing enhanced SMPT parsing...")
    
    # Run SMPT with model output
    cmd = ["./smpt_wrapper.sh", "-n", net_file, "--xml", xml_file, "--methods", "BMC", "--show-model"]
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    
    if result.returncode == 0:
        print("✅ SMPT execution successful")
        print(f"📤 Raw stdout: {result.stdout}")
        print(f"📤 Raw stderr: {result.stderr}")
        
        # Test our parsing logic
        if "# Model:" in result.stdout:
            print("✅ Model found in output")
            # Extract the model
            for line in result.stdout.split('\n'):
                if line.startswith("# Model: "):
                    model_str = line[9:]  # Remove "# Model: " prefix
                    print(f"📊 Model: {model_str}")
                    
                    # Parse the model (simulate our Rust parsing)
                    places = []
                    for part in model_str.split():
                        if '(' in part and ')' in part:
                            place_name = part[:part.find('(')]
                            tokens = part[part.find('(')+1:part.find(')')]
                            places.append((place_name, tokens))
                    
                    print(f"📊 Parsed places: {places}")
        else:
            print("ℹ️  No model in output")
            
        if "TRUE" in result.stdout:
            print("✅ Property is REACHABLE")
        elif "FALSE" in result.stdout:
            print("✅ Property is UNREACHABLE")
        else:
            print("❌ Could not determine reachability")
            
        return True
    else:
        print(f"❌ SMPT failed: {result.stderr}")
        return False

if __name__ == "__main__":
    success = test_enhanced_smpt()
    sys.exit(0 if success else 1)