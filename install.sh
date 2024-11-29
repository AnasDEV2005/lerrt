#!/bin/bash

# Name of the executable
EXECUTABLE_NAME="time-blocking-helper"

# Build the executable (ensure Rust is installed)
echo "Building the project..."
cargo build --release || { echo "Build failed!"; exit 1; }

# Define the target directory
TARGET_DIR="/usr/local/bin"

# Copy the executable
echo "Installing $EXECUTABLE_NAME to $TARGET_DIR..."
sudo cp "target/release/$EXECUTABLE_NAME" "$TARGET_DIR/" || { echo "Failed to copy the executable!"; exit 1; }

# Ensure it is executable
sudo chmod +x "$TARGET_DIR/$EXECUTABLE_NAME"

echo "$EXECUTABLE_NAME installed successfully. You can run it from anywhere using '$EXECUTABLE_NAME'."
