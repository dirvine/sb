#!/bin/bash

# Test script for sb file picker functionality
echo "Testing sb file picker status bar and commands"
echo ""
echo "1. Run: ./target/debug/sb test_file.txt"
echo "2. Press F2 (or Ctrl+I) to open file picker"
echo "3. Check if you see:"
echo "   - A cyan-bordered popup window labeled 'File Picker'"
echo "   - Files listed with 📁 or 📄 icons"
echo "   - Yellow status bar at bottom with commands"
echo "4. Try these keys:"
echo "   - ↑↓ to navigate files"
echo "   - D to delete"
echo "   - P for parent directory"
echo "   - S for git status"
echo "   - ESC to cancel"
echo ""
echo "Building sb first..."
cargo build
echo ""
echo "Starting sb now..."
./target/debug/sb test_file.txt