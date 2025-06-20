#!/bin/bash

# SMPT Wrapper Script
# This script activates the SMPT virtual environment and runs SMPT

SMPT_DIR="/Users/jules/git/ser/SMPT"
VENV_DIR="$SMPT_DIR/smpt_venv"

if [ ! -d "$VENV_DIR" ]; then
    echo "Error: SMPT virtual environment not found at $VENV_DIR"
    echo "Please run the install_smpt.sh script first."
    exit 1
fi

# Activate virtual environment and run SMPT
cd "$SMPT_DIR"
source "$VENV_DIR/bin/activate"
python -m smpt "$@"