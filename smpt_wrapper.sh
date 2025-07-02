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
