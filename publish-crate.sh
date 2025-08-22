#!/bin/bash

# Helper script to publish a single crate
# Usage: ./publish-crate.sh <crate-name>

set -e

CRATE=$1

if [ -z "$CRATE" ]; then
    echo "Usage: ./publish-crate.sh <crate-name>"
    echo "Available crates: audio, stt, tts, voice-toolkit"
    exit 1
fi

if [ ! -d "$CRATE" ]; then
    echo "Error: Crate directory '$CRATE' not found"
    exit 1
fi

echo "🚀 Preparing to publish $CRATE..."

# For sub-crates, temporarily remove publish = false
if [ "$CRATE" != "voice-toolkit" ]; then
    echo "📝 Temporarily enabling publishing for $CRATE..."
    sed -i '' 's/^publish = false$/# publish = false/' $CRATE/Cargo.toml
fi

# For STT crate, add version dependency
if [ "$CRATE" = "stt" ]; then
    echo "📦 Adding version dependency for STT crate..."
    sed -i '' 's|path = "\.\./audio"|version = "0.3.0", path = "../audio"|' stt/Cargo.toml
fi

echo "📦 Publishing $CRATE v0.3.0..."
cd $CRATE && cargo publish
cd ..

echo "✅ $CRATE published successfully!"

# Restore original configuration
if [ "$CRATE" != "voice-toolkit" ]; then
    echo "🔄 Restoring original configuration for $CRATE..."
    sed -i '' 's/^# publish = false$/publish = false/' $CRATE/Cargo.toml
fi

if [ "$CRATE" = "stt" ]; then
    echo "🔄 Restoring STT dependency configuration..."
    sed -i '' 's|version = "0\.3\.0", path = "\.\./audio"|path = "../audio"|' stt/Cargo.toml
fi

echo "🎉 Done!"

if [ "$CRATE" = "voice-toolkit" ]; then
    echo "📚 Documentation: https://docs.rs/voice-toolkit/0.3.0"
    echo "📦 Crate: https://crates.io/crates/voice-toolkit"
fi