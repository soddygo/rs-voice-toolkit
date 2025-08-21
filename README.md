rs-voice-toolkit
=================

Rust 语音处理工具库（Workspace），专注于简洁、通用、易集成：
- STT：基于 Whisper 的语音转文本
- TTS：后续通过 Index-TTS（二阶段）
- Audio：基于 ez-ffmpeg 的轻量音频处理

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

模块：
- stt/：语音转文本（文件与实时接口原型）
- audio/：音频探测/转码/重采样（基于 ez-ffmpeg）
- tts/：文本转语音（Index-TTS，引擎可选）

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


