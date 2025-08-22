# Publishing Guide for rs-voice-toolkit

## 📋 Prerequisites

1. **Crates.io Account**: Make sure you have an account on crates.io
2. **Login**: Run `cargo login` with your API token
3. **Git Clean**: Ensure all changes are committed and pushed

## 🚀 Publishing Steps

### Step 1: Publish Sub-crates (in order)

1. **Audio Crate**:
   ```bash
   cd audio
   cargo publish
   cd ..
   ```

2. **STT Crate**:
   ```bash
   cd stt
   cargo publish
   cd ..
   ```

3. **TTS Crate**:
   ```bash
   cd tts
   cargo publish
   cd ..
   ```

### Step 2: Publish Main Crate

```bash
cd voice-toolkit
cargo publish
```

## ⚠️ Important Notes

- **Dependency Order**: Must publish in order: audio → stt → tts → voice-toolkit
- **Version Consistency**: All crates should use the same version (currently 0.3.0)
- **Documentation**: Docs will be available at https://docs.rs/voice-toolkit/0.3.0

## 🔧 Configuration

All sub-crates have `publish = false` to prevent accidental publishing. You need to temporarily remove this line before publishing each crate, then restore it afterwards.

## 📦 After Publishing

Users can install with:
```bash
cargo install voice-toolkit
```

Or add to Cargo.toml:
```toml
[dependencies]
voice-toolkit = "0.3.0"
```

## 🎯 Features Available

- `stt`: Speech-to-Text functionality (default)
- `tts`: Text-to-Speech functionality  
- `audio`: Audio processing utilities (default)
- `streaming`: Real-time streaming transcription