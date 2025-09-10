#!/bin/bash

# Kill any existing processes on ports 8081 and 8083
echo "Checking for existing processes on ports 8081 and 8083..."
lsof -ti:8081 | xargs kill -9 2>/dev/null
lsof -ti:8083 | xargs kill -9 2>/dev/null

# Store the original directory
ORIGINAL_DIR=$(pwd)

# Start the matrix multiplication server in the background
echo "Starting matrix multiplication server on http://127.0.0.1:8081"
cd "$ORIGINAL_DIR/matrix-multiplication" && cargo run &
MATRIX_MULT_PID=$!

# Wait a moment to ensure the first server starts
sleep 3

# Start the matrix printer server in the background
echo "Starting matrix printer server on http://127.0.0.1:8083"
cd "$ORIGINAL_DIR/matrix-printer" && cargo run &
MATRIX_PRINTER_PID=$!

# Function to handle script termination
cleanup() {
  echo "Stopping servers..."
  kill $MATRIX_MULT_PID $MATRIX_PRINTER_PID 2>/dev/null
  exit 0
}

# Register the cleanup function for when the script is terminated
trap cleanup INT TERM

echo "Both servers are running. Press Ctrl+C to stop."
echo "- Matrix multiplication UI: http://127.0.0.1:8081"
echo "- Matrix printer UI: http://127.0.0.1:8083"

# Keep the script running
wait

# Made with Bob
