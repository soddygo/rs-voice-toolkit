#!/bin/bash

# Publish script for rs-voice-toolkit workspace
# Publishes sub-crates first, then main crate in correct dependency order

set -e

echo "🚀 Publishing rs-voice-toolkit v0.3.0..."

# Remove publish = false temporarily from sub-crates
sed -i '' 's/^publish = false$/# publish = false/' audio/Cargo.toml
sed -i '' 's/^publish = false$/# publish = false/' stt/Cargo.toml  
sed -i '' 's/^publish = false$/# publish = false/' tts/Cargo.toml

# Add version numbers to dependencies for publishing
sed -i '' 's|path = "\.\./audio"|version = "0.3.0", path = "../audio"|' stt/Cargo.toml

# Publish in dependency order: audio -> stt -> tts -> voice-toolkit
echo "📦 Publishing audio crate..."
cd audio && cargo publish --allow-dirty && cd ..

echo "📦 Publishing STT crate..."
cd stt && cargo publish --allow-dirty && cd ..

echo "📦 Publishing TTS crate..."
cd tts && cargo publish --allow-dirty && cd ..

echo "📦 Publishing main voice-toolkit crate..."
cd voice-toolkit && cargo publish --allow-dirty && cd ..

# Restore original configuration
echo "🔄 Restoring original configuration..."
sed -i '' 's/^# publish = false$/publish = false/' audio/Cargo.toml
sed -i '' 's/^# publish = false$/publish = false/' stt/Cargo.toml
sed -i '' 's/^# publish = false$/publish = false/' tts/Cargo.toml
sed -i '' 's|version = "0\.3\.0", path = "\.\./audio"|path = "../audio"|' stt/Cargo.toml

echo "✅ All crates published successfully!"
echo "📚 Documentation will be available at: https://docs.rs/voice-toolkit/0.3.0"