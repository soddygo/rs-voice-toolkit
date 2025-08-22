rs-voice-toolkit
=================

[![Crates.io](https://img.shields.io/crates/v/voice-toolkit.svg)](https://crates.io/crates/voice-toolkit)
[![Documentation](https://docs.rs/voice-toolkit/badge.svg)](https://docs.rs/voice-toolkit)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

**Version: 0.3.0** - Unified Voice Processing Toolkit

A comprehensive Rust toolkit for voice processing with unified API, featuring Speech-to-Text (STT), Text-to-Speech (TTS), and audio utilities.

## 🚀 Core Features

### 🎤 Speech-to-Text (STT)
- **Whisper Integration**: High-quality speech recognition using OpenAI's Whisper models
- **File Transcription**: Process audio files with automatic format conversion
- **Streaming Support**: Real-time audio streaming with low latency
- **VAD Integration**: Voice Activity Detection for efficient processing
- **Performance Monitoring**: Built-in benchmarking and performance metrics

### 🔊 Text-to-Speech (TTS)  
- **Index-TTS Engine**: High-quality speech synthesis
- **Extensible Architecture**: Support for multiple TTS engines
- **Flexible Output**: Generate audio to memory buffers or files
- **Multi-format Support**: Output in various audio formats

### 🎵 Audio Processing
- **Format Conversion**: Convert between audio formats (WAV, MP3, FLAC, M4A, OGG)
- **Resampling**: High-quality audio resampling with rubato library
- **Whisper Compatibility**: Automatic conversion to Whisper-compatible format (mono, 16kHz, 16-bit PCM)
- **Metadata Extraction**: Probe audio files for technical information
- **FFmpeg Integration**: Leverages system FFmpeg for robust format support

## 📦 Feature Flags

The toolkit supports optional features via Cargo feature flags:

```toml
[dependencies]
voice-toolkit = { version = "0.3.0", features = ["stt", "tts", "audio"] }
```

### Available Features:
- **`stt`**: Speech-to-Text functionality (enabled by default)
- **`tts`**: Text-to-Speech functionality
- **`audio`**: Audio processing utilities (enabled by default)
- **`streaming`**: Real-time streaming transcription (requires `stt`)

### Default Features:
The default feature set includes `stt` and `audio` for comprehensive voice processing capabilities.

快速开始
--------

依赖：
- Rust（stable）
- FFmpeg（ez-ffmpeg 依赖）
  - macOS: brew install ffmpeg
  - Windows: vcpkg 安装 FFmpeg（参考 ez-ffmpeg）
  - Linux: 使用发行版包管理器
- Whisper 模型（如 ggml-base.bin）

参考：ez-ffmpeg 仓库 https://github.com/YeautyYE/ez-ffmpeg

构建：

```bash
cargo build
```

运行 STT 示例（文件转录）：

```bash
# 示例：stt/examples/transcribe_file.rs
cargo run -p stt --example transcribe_file -- <模型路径> <音频文件>

# 例如：
# cargo run -p stt --example transcribe_file -- models/ggml-base.bin samples/hello.wav
```

备注：示例会自动使用 audio 子库将音频转换为 Whisper 兼容的 mono/16k/PCM16 WAV 后再进行转录。

## 模块架构

### 核心模块
- **stt/** (`rs-voice-toolkit-stt`): 语音转文本
  - 文件转录和流式转录
  - VAD（语音活动检测）集成
  - 性能监控和基线测试
- **audio/** (`rs-voice-toolkit-audio`): 音频处理
  - 音频格式检测和元数据提取
  - 格式转换和重采样
  - Whisper 兼容预处理
- **tts/** (`rs-voice-toolkit-tts`): 文本转语音
  - Index-TTS 引擎集成
  - 可扩展引擎架构
  - 内存和文件输出选项
- **voice-toolkit/**: 统一接口库
  - 所有模块的统一导出
  - 简化的 API 接口

更多设计与任务见 specs/design.md 与 specs/tasks.md。

测试样例（fixtures）：
- 运行 `./fixtures/get-fixtures.sh` 下载最小模型与样例音频
- 然后执行：

```bash
cargo run -p stt --example transcribe_file -- fixtures/models/ggml-tiny.bin fixtures/audio/jfk.wav
```

TTS 示例（Index-TTS）：

```bash
# 需要已安装 index-tts 并在 PATH，或将可执行路径作为第三个参数传入
cargo run -p tts --example synthesize -- "你好，世界" out.wav [/path/to/index-tts]
```

Streaming 示例：

```bash
cargo run -p stt --features streaming --example streaming_transcribe -- fixtures/models/ggml-tiny.bin fixtures/audio/jfk.wav
```

性能基准示例（RTF）：

```bash
cargo run -p stt --example bench_transcribe -- fixtures/models/ggml-tiny.bin fixtures/audio/jfk.wav 3
```


