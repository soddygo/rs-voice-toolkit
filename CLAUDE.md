# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Voice Toolkit is a comprehensive Rust library for voice processing including Speech-to-Text (STT), Text-to-Speech (TTS), and audio utilities. It provides a unified API interface built on top of OpenAI Whisper models for high-quality speech recognition.

### Architecture

The project follows a modular workspace structure with separate crates for each functionality:

- **voice-toolkit/**: Main unified interface library providing convenient APIs and unified error handling
- **stt/**: Speech-to-Text module (rs-voice-toolkit-stt) based on Whisper models
- **tts/**: Text-to-Speech module (rs-voice-toolkit-tts) with Index-TTS engine
- **audio/**: Audio processing module (rs-voice-toolkit-audio) for format conversion and preprocessing

### Module Dependencies

```
voice-toolkit (main interface)
├── stt (depends on audio)
├── tts (standalone)
└── audio (foundation layer)
```

## Common Development Commands

### Building and Testing

```bash
# Build all workspace crates
cargo build --workspace

# Build in release mode
cargo build --release --workspace

# Run tests for all crates
cargo test --workspace

# Run tests for specific crate
cargo test -p rs-voice-toolkit-stt
cargo test -p rs-voice-toolkit-audio
cargo test -p rs-voice-toolkit-tts

# Check compilation without building
cargo check --workspace

# Run linting
cargo clippy --workspace

# Format code
cargo fmt --all

# Generate documentation
cargo doc --no-deps --workspace
```

### Running Examples

```bash
# STT examples
cargo run -p rs-voice-toolkit-stt --example transcribe_file -- models/ggml-base.bin audio/hello.wav
cargo run -p rs-voice-toolkit-stt --example bench_transcribe -- models/ggml-base.bin audio/hello.wav 3

# TTS examples
cargo run -p rs-voice-toolkit-tts --example synthesize -- "你好，世界" output.wav

# Unified examples
cargo run --example usage_examples
cargo run --example streaming_examples --features streaming
```

### Feature Flags

The library uses feature flags for modular functionality:

```toml
# Basic STT + Audio
voice-toolkit = { version = "0.15.0", features = ["stt", "audio"] }

# Full functionality
voice-toolkit = { version = "0.15.0", features = ["stt", "tts", "audio"] }

# With streaming support
voice-toolkit = { version = "0.15.0", features = ["stt", "audio", "streaming"] }

# With GPU acceleration
voice-toolkit = { version = "0.15.0", features = ["stt", "audio", "cuda"] }
```

Available features:
- `stt`: Speech-to-Text functionality (default)
- `tts`: Text-to-Speech functionality
- `audio`: Audio processing utilities (default)
- `streaming`: Real-time streaming transcription
- `cuda`: NVIDIA GPU acceleration
- `vulkan`: Cross-platform GPU acceleration
- `metal`: Apple Silicon GPU acceleration

## Key Technical Details

### Error Handling Architecture

The project uses a unified error handling system through the main `voice-toolkit` crate:

```rust
use voice_toolkit::{Error, Result};

// All functions return Result<T, Error>
fn process_audio() -> Result<()> {
    // Processing logic
    Ok(())
}
```

Error types are automatically converted from sub-modules:
- `Error::Audio`: Audio processing errors
- `Error::Stt`: Speech recognition errors
- `Error::Tts`: Text-to-speech errors
- `Error::Io`: File I/O errors
- `Error::Other`: Uncategorized errors

### Async/Await Pattern

All I/O operations and heavy processing are asynchronous using tokio:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let result = transcribe_file_unified("model.bin", "audio.wav").await?;
    Ok(())
}
```

### Time Units

Important: The codebase uses milliseconds (u64) for time measurements, not `std::time::Duration`:

```rust
// TranscriptionResult.processing_time is in milliseconds
let processing_time_ms = result.processing_time;
let processing_time_seconds = processing_time_ms as f64 / 1000.0;

// AudioMeta.duration_ms is Option<u64>
let duration_seconds = metadata.duration_ms.unwrap_or(0) as f64 / 1000.0;
```

### Audio Processing Pipeline

1. **Format Detection**: `audio::probe()` returns `AudioMeta` with format information
2. **Format Conversion**: `audio::ensure_whisper_compatible()` converts to Whisper-compatible format
3. **STT Processing**: Whisper models require 16kHz, mono, 16-bit PCM audio

### Whisper Model Integration

- Model files are .bin format (whisper.cpp compatible)
- Supported models: tiny, base, small, medium, large
- Models are loaded on first use and cached
- GPU acceleration available through feature flags

### Performance Considerations

- **Model Loading**: First call loads model, subsequent calls reuse instance
- **VAD Integration**: Voice Activity Detection skips silent portions
- **Batch Processing**: Process multiple files to reduce initialization overhead
- **Streaming**: Use streaming API for real-time applications

## Development Guidelines

### Code Style

- Follow standard Rust formatting (cargo fmt)
- Use comprehensive documentation comments for public APIs
- Prefer Result<T, Error> over panic! for error handling
- Use async/await for all I/O operations

### Adding New Features

1. **STT Features**: Add to `stt/src/lib.rs` with comprehensive documentation
2. **TTS Features**: Add to `tts/src/lib.rs` following engine trait pattern
3. **Audio Features**: Add to `audio/src/lib.rs` with format support
4. **Unified API**: Expose through `voice-toolkit/src/lib.rs` with feature gates

### Testing Strategy

- Unit tests go in each crate's `tests/` module
- Integration tests go in separate files under `tests/`
- Examples serve as both documentation and tests
- Use `#[tokio::test]` for async tests

### Version Management

Use the provided Makefile for version management:

```bash
# Bump all versions to next minor
make bump-version

# Publish all crates in correct order
make publish-all
```

Publishing order: audio → tts → stt → voice-toolkit (main)

## External Dependencies

### Core Dependencies

- **whisper-rs**: Whisper model integration
- **ffmpeg-sidecar**: Audio format conversion with auto-download
- **rubato**: High-quality audio resampling
- **hound**: WAV file processing
- **tokio**: Async runtime

### Optional Dependencies

- **Index-TTS**: TTS engine (external binary)
- **FFmpeg**: System dependency for audio processing

## System Requirements

- Rust 1.70+
- FFmpeg (system dependency)
- Whisper model files (.bin format)
- Optional: GPU acceleration support

## Common Pitfalls

1. **Time Units**: Remember processing_time is milliseconds, not Duration
2. **Async Context**: All STT/TTS operations require async runtime
3. **Feature Flags**: Ensure required features are enabled in Cargo.toml
4. **Model Files**: Whisper models must be downloaded separately
5. **FFmpeg Dependency**: System FFmpeg installation required for audio conversion