#!/bin/bash

# Script to install SMPT (Satisfiability Modulo Petri Nets) tool

echo "🔧 Installing SMPT (Satisfiability Modulo Petri Nets)..."
echo

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is not installed. Please install Python 3.7+ first."
    exit 1
fi

echo "✅ Python 3 found: $(python3 --version)"

# Check if pip is available
if ! command -v pip3 &> /dev/null && ! python3 -m pip --version &> /dev/null; then
    echo "❌ pip is not available. Please install pip first."
    exit 1
fi

echo "✅ pip found"

# Install Z3 solver
echo "📦 Installing Z3 solver..."
python3 -m pip install --user z3-solver

if [ $? -eq 0 ]; then
    echo "✅ Z3 solver installed successfully"
else
    echo "❌ Failed to install Z3 solver"
    exit 1
fi

# Try to install SMPT via pip first (easier method)
echo "📦 Attempting to install SMPT via pip..."
python3 -m pip install --user smpt

if [ $? -eq 0 ]; then
    echo "✅ SMPT installed successfully via pip"
else
    echo "⚠️  pip install failed, trying manual installation..."
    
    # Clone and install manually
    echo "📦 Cloning SMPT repository..."
    if [ -d "SMPT" ]; then
        echo "🔄 SMPT directory already exists, updating..."
        cd SMPT
        git pull
    else
        git clone https://github.com/nicolasAmat/SMPT.git
        cd SMPT
    fi
    
    if [ $? -ne 0 ]; then
        echo "❌ Failed to clone SMPT repository"
        exit 1
    fi
    
    echo "🔨 Building SMPT..."
    python3 setup.py bdist_wheel
    
    if [ $? -ne 0 ]; then
        echo "❌ Failed to build SMPT"
        exit 1
    fi
    
    echo "📦 Installing SMPT wheel..."
    python3 -m pip install --user dist/smpt-*.whl
    
    if [ $? -ne 0 ]; then
        echo "❌ Failed to install SMPT wheel"
        exit 1
    fi
    
    cd ..
    echo "✅ SMPT installed successfully via manual build"
fi

# Test installation
echo
echo "🧪 Testing SMPT installation..."

# Test local installation first
if [ -f "./smpt_wrapper.sh" ]; then
    ./smpt_wrapper.sh --help > /dev/null 2>&1
    if [ $? -eq 0 ]; then
        echo "✅ SMPT is working correctly via local installation!"
        echo
        echo "🎉 Installation complete! You can now use SMPT with the ser tool."
        echo
        echo "Test with: cargo run -- --check-smpt"
        exit 0
    fi
fi

# Fall back to testing global installation
python3 -m smpt --help > /dev/null 2>&1

if [ $? -eq 0 ]; then
    echo "✅ SMPT is working correctly via global installation!"
    echo
    echo "📊 SMPT version info:"
    python3 -m smpt --version 2>/dev/null || echo "Version info not available"
    echo
    echo "🎉 Installation complete! You can now use SMPT with the ser tool."
    echo
    echo "Test with: cargo run -- --check-smpt"
else
    echo "❌ SMPT installation verification failed"
    echo "But local virtual environment installation may have succeeded."
    echo "Try running: cargo run -- --check-smpt"
fi