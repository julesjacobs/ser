#!/bin/bash
set -e

# Move to the extension directory
cd "$(dirname "$0")"

# Install dependencies
npm install

# Package the extension
npm run package

echo -e "\nVSIX package created. To install, run:"
echo "code --install-extension ser-lang-*.vsix"