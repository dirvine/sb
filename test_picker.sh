#!/bin/bash

# Create a test environment
echo "Testing sb file picker functionality..."

# Create a simple test file if it doesn't exist
if [ ! -f test_file.txt ]; then
    echo "test content" > test_file.txt
fi

# Run sb in background
echo "Starting sb..."
./target/debug/sb test_file.txt &
SB_PID=$!

# Give it time to start
sleep 1

# Check if it's running
if ! ps -p $SB_PID > /dev/null; then
    echo "Error: sb failed to start"
    exit 1
fi

echo "sb is running with PID $SB_PID"
echo "Please test the following:"
echo "1. Press Ctrl+I to open file picker"
echo "2. Check the status bar for debug messages"
echo "3. Press 'q' to quit"
echo ""
echo "Press Enter to kill the app when done testing..."
read

# Kill the app
kill $SB_PID 2>/dev/null
echo "Test complete"