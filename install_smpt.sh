#!/bin/bash

# Script to install SMPT (Satisfiability Modulo Petri Nets) tool
# This script creates a local virtual environment to avoid affecting the global Python setup

set -e  # Exit on any error

echo "🔧 Installing SMPT (Satisfiability Modulo Petri Nets)..."
echo "   This will create a local virtual environment for SMPT to avoid conflicts."
echo

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is not installed. Please install Python 3.7+ first."
    exit 1
fi

echo "✅ Python 3 found: $(python3 --version)"

# Check if venv is available
if ! python3 -c "import venv" &> /dev/null; then
    echo "❌ Python venv module is not available. Please install python3-venv package."
    exit 1
fi

echo "✅ Python venv module found"

# Create SMPT directory if it doesn't exist
if [ ! -d "SMPT" ]; then
    mkdir SMPT
    echo "📁 Created SMPT directory"
fi

cd SMPT

# Create virtual environment
if [ ! -d "smpt_venv" ]; then
    echo "🔨 Creating virtual environment..."
    python3 -m venv smpt_venv
    echo "✅ Virtual environment created"
else
    echo "✅ Virtual environment already exists"
fi

# Activate virtual environment
echo "⚡ Activating virtual environment..."
source smpt_venv/bin/activate

# Upgrade pip and install basic tools
echo "📦 Upgrading pip and installing basic tools..."
python -m pip install --upgrade pip setuptools wheel

# Install dependencies
echo "📦 Installing Z3 solver..."
python -m pip install z3-solver

echo "📦 Installing sexpdata (required by SMPT)..."
python -m pip install sexpdata

# Try to install SMPT via pip first (easier method)
echo "📦 Attempting to install SMPT via pip..."
if python -m pip install smpt; then
    echo "✅ SMPT installed successfully via pip"
    INSTALL_METHOD="pip"
else
    echo "⚠️  pip install failed, trying manual installation..."
    
    # Clone and install manually
    echo "📦 Cloning SMPT repository..."
    if [ -d "SMPT-repo" ]; then
        echo "🔄 SMPT repository already exists, updating..."
        cd SMPT-repo
        git pull
    else
        git clone https://github.com/nicolasAmat/SMPT.git SMPT-repo
        cd SMPT-repo
    fi
    
    echo "🔨 Building SMPT..."
    python setup.py bdist_wheel
    
    echo "📦 Installing SMPT wheel..."
    python -m pip install dist/smpt-*.whl
    
    cd ..
    echo "✅ SMPT installed successfully via manual build"
    INSTALL_METHOD="manual"
fi

# Test installation
echo
echo "🧪 Testing SMPT installation..."
if python -m smpt --help > /dev/null 2>&1; then
    echo "✅ SMPT is working correctly in virtual environment!"
    echo "   Installation method: $INSTALL_METHOD"
    
    # Get version info
    echo "📊 SMPT version info:"
    python -m smpt --help | head -5
    
    # Create wrapper script
    echo "🔗 Creating wrapper script..."
    cd ..  # Back to project root
    
    cat > smpt_wrapper.sh << 'EOF'
#!/bin/bash
# Wrapper script to run SMPT from virtual environment

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if virtual environment exists
VENV_PATH="$SCRIPT_DIR/SMPT/smpt_venv"
if [ ! -d "$VENV_PATH" ]; then
    echo "❌ SMPT virtual environment not found at $VENV_PATH"
    echo "   Run ./install_smpt.sh to install SMPT"
    exit 1
fi

# Activate virtual environment and run SMPT
source "$VENV_PATH/bin/activate"
python -m smpt "$@"
EOF
    
    chmod +x smpt_wrapper.sh
    echo "✅ Wrapper script created: smpt_wrapper.sh"
    
    # Test wrapper script
    echo "🧪 Testing wrapper script..."
    if ./smpt_wrapper.sh --help > /dev/null 2>&1; then
        echo "✅ Wrapper script is working correctly!"
    else
        echo "⚠️  Wrapper script test failed, but SMPT should still work in virtual environment"
    fi
    
    echo
    echo "🎉 Installation complete!"
    echo
    echo "📁 Files created:"
    echo "   SMPT/                 - SMPT installation directory"
    echo "   SMPT/smpt_venv/       - Python virtual environment"
    echo "   smpt_wrapper.sh       - Wrapper script to run SMPT"
    echo
    echo "💡 These files are automatically excluded from git commits."
    echo
    echo "🚀 Test with: cargo test test_manual_smpt_integration"
    echo "   Or run:   cargo run -- examples/ser/simple_ser.ser"
    
else
    echo "❌ SMPT installation verification failed"
    echo "Check the installation logs above for errors."
    exit 1
fi