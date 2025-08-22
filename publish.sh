#!/bin/bash

# Script to prepare voice-toolkit for publishing

# Backup original Cargo.toml
cp voice-toolkit/Cargo.toml voice-toolkit/Cargo.toml.backup

# Replace path dependencies with version dependencies for publishing
sed -i '' 's|path = "\.\./[^"]*"|version = "0.3.0"|g' voice-toolkit/Cargo.toml

# Publish the crate
echo "Publishing voice-toolkit v0.3.0..."
cd voice-toolkit && cargo publish

# Restore original Cargo.toml
cd ..
mv voice-toolkit/Cargo.toml.backup voice-toolkit/Cargo.toml

echo "Publish completed!"
echo "Note: You need to publish the sub-crates (stt, tts, audio) to crates.io first."