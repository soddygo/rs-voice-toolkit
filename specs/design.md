## Rust 语音工具库技术方案（Design）

### 目标与非目标
- 目标：
  - 构建一个简洁、通用、易集成的 Rust 语音处理库（lib 优先），提供 STT 与后续可扩展的 TTS 能力。
  - 面向多工程复用，最小依赖、合理默认值、清晰稳定的 API。
  - 跨平台支持（macOS、Linux、Windows），离线可用。
- 非目标：
  - 不提供重量级服务端框架；不强制网络依赖。
  - 不在首版实现复杂的 GUI/可视化与大型模型管理工具。

### 架构总览（Workspace 独立模块）
- stt（独立库）：语音转文本核心能力（文件与实时），内置必要的音频预处理入口。
- tts（独立库，二阶段）：文本转语音能力，优先集成 Index-TTS 命令行，后续可扩展。
- audio（独立库）：轻量音频处理能力（探测、转码、重采样），基于 `ez-ffmpeg` 封装。
- specs：规范与文档。

依赖关键点：
- STT：`whisper-rs`（Whisper C/C++ 实现绑定），离线识别，多语言；音频需转换为 mono/16k/PCM16 或 float32 兼容格式。
- 音频：`ez-ffmpeg` 用于格式探测与转码、重采样，减少直接调用 ffmpeg 的心智负担（参考仓库 `ez-ffmpeg`：[GitHub 链接](https://github.com/YeautyYE/ez-ffmpeg)）。
- TTS：首选 `index-tts` 外部 CLI（中文自然度优先），采用进程调用，接口抽象以便未来切换/并存 Piper/Coqui 等方案。

### 关键设计原则
- 简洁 API：函数名清晰、参数最少、默认值合理；复杂性向内收敛。
- 通用性：面向库的通用嵌入式使用场景，避免与具体上层框架耦合。
- 稳定性：最小可行集合（MVP）先行，确保 STT 文件转录与预处理稳定可靠。
- 可演进：为实时转录与多引擎 TTS 预留扩展点与可替换边界。

### 模块职责与边界
1) stt（语音转文本）
   - 文件转录：输入多格式音频路径 → （audio 模块转码/重采样）→ Whisper 推理 → 文本结果。
   - 内存数据转录：输入采样数据（float32/PCM16）+ 采样率 → Whisper 推理 → 文本结果。
   - 实时转录（二阶段）：面向音频块（chunk）接口，维护滑动窗口、重叠策略、结果合并。
   - 配置最小化：`SttConfig`（模型路径、语言、线程数、是否翻译、温度等提供默认值）。

2) audio（音频处理）
   - 探测：读取媒体信息（采样率、通道、编码、时长）。
   - 转码与重采样：统一提供 `ensure_whisper_compatible` 能力（输出 mono/16k/PCM16/或 f32）。
   - 轻包装 `ez-ffmpeg`，对上层暴露稳定、简洁函数；隐藏复杂参数。

3) tts（文本转语音）
   - 首版：Index-TTS 引擎（进程调用），返回内存音频数据或写文件。
   - 引擎选择：`TtsEngineType`（首期仅 `IndexTts`），接口抽象但不引入过度层次；后续可并存 `Piper/Coqui`。
   - 配置最小化：`TtsConfig`（可执行路径、语言、说话人、采样率）。

### API 形态（接口概览）
为保持规范，这里仅列出接口形态（无实现代码）：
- stt：
  - `transcribe_file(path, model_path, [config]) -> Result<Transcription, SttError>`
  - `transcribe_samples(samples, sample_rate, [config]) -> Result<Transcription, SttError>`
  - `StreamingTranscriber::new(model_path, [config])`
  - `StreamingTranscriber::process_chunk(chunk) -> Result<Option<PartialText>, SttError>`
  - `StreamingTranscriber::finalize() -> Result<Transcription, SttError>`
- audio：
  - `probe(input_path) -> Result<AudioMeta, AudioError>`
  - `ensure_whisper_compatible(input_path, output_path?) -> Result<CompatibleWav, AudioError>`
  - `resample(input, from_rate, to_rate) -> Result<Resampled, AudioError>`
- tts：
  - `synthesize_to_memory(text, [config]) -> Result<AudioBytes, TtsError>`
  - `synthesize_to_file(text, output_path, [config]) -> Result<(), TtsError>`

说明：方括号参数为可选配置，均提供合理默认值，保证开箱即用。

### 实时转录设计（二阶段）
- 输入：固定时长 chunk（建议 0.5s），与上一个 chunk 以 0.2s 重叠。
- 缓冲：环形缓冲 + 滑动窗口；基于帧对齐（如 10/20ms）。
- 结果确认：LocalAgreement-n（n=2/3 可调）合并与去重；根据 VAD（可选）抑制静音段处理。
- 时延控制：动态调整 chunk 长度与重叠度；在延迟与准确度之间平衡。
- 线程/异步：音频采集（或输入）与推理解耦；通道（mpsc）连接，防止阻塞；可配置 `n_threads`。

### 音频处理方案（基于 ez-ffmpeg）
- 统一入口：`audio::ensure_whisper_compatible` 完成格式探测、重采样与声道转换。
- 处理策略：
  - 输入任意主流格式（mp3/aac/wav/flac/m4a 等）→ 输出 mono/16k/WAV（PCM16）。
  - 对于实时场景，优先输入 f32 流；必要时在内存中重采样，避免文件落地。
- 参考：`ez-ffmpeg` 提供安全易用的 FFmpeg 接口，降低集成复杂度（仓库：[链接](https://github.com/YeautyYE/ez-ffmpeg)）。

### TTS 方案（首版 Index-TTS、可扩展）
- 当前：
  - 通过子进程调用 `index-tts`（需在 PATH 或提供可执行路径）。
  - 支持输出到内存或文件的 WAV（默认 22050Hz，后续可配置）。
  - 以中文高自然度为首要目标，接口保持与后续引擎兼容。
- 扩展：
  - 预留 `TtsEngineType` 枚举；新增引擎时仅需实现同形接口并注册。
  - Piper/Coqui 作为后续备选，适用于无外部 CLI 或多平台一体化需求。

### 错误模型与日志
- 错误类型：`SttError`、`AudioError`、`TtsError`，均为精确分类（配置错误、I/O 错误、执行错误、推理错误等）。
- 返回方式：所有对外 API 均返回 `Result<..., ...>`；不 panic。
- 日志：`log` + 可选实现（如 `env_logger`），区分 info/warn/error/debug；默认降噪。

### 配置与默认值
- STT：
  - `model_path`（默认 `models/ggml-base.bin` 可覆盖）
  - `language`（默认自动检测）
  - `n_threads`（默认 CPU 合理 cores）
  - 其他参数（translate、temperature 等）提供默认值。
- TTS：
  - `executable_path`（index-tts 路径，默认查 PATH）
  - `language`、`speaker`、`sample_rate`（合理默认）
- Audio：
  - 统一目标格式 mono/16k/WAV；允许覆盖。

### 性能与资源
- Whisper 通常为 CPU 推理，可通过：
  - 使用小模型/量化模型；
  - 重用上下文/状态（state）以减少初始化开销；
  - 合理设置 `n_threads`；
  - 监控实时因子（RTF），作为调优指标。
- 内存控制：目标 < 1GB（含模型）；长音频分段处理；释放不再使用的缓冲。

### 依赖与系统要求
- Crates：`whisper-rs`、`ez-ffmpeg`、`tokio`、`serde`、`thiserror`、（可选）`hound`/`rubato`。
- 系统依赖：
  - FFmpeg（由 `ez-ffmpeg` 使用）
  - Index-TTS 可执行文件（可选安装，走 PATH）
- 平台：macOS 14+/Linux/Windows；静态/动态链接 FFmpeg 按平台与特性控制。

### 打包与发布
- Workspace 下各模块独立发布；遵循语义化版本。
- Feature flags：
  - `stt/streaming` 启用实时功能。
  - `tts/index` 启用 Index-TTS 集成（默认开启或可选）。
  - `audio/ffmpeg-static`（按需，为 Windows 提供静态链接选项）。

### 测试与质量
- 单元测试：
  - stt：短音频样例、边界参数、错误路径。
  - audio：多格式转码、采样率/声道正确性、容错。
  - tts：在存在 index-tts 时进行可用性验证（按需打标跳过）。
- 集成测试：端到端（文件 → 文本）与（文本 → 音频）。
- 性能测试：评估 RTF、内存、延迟；回归基线。
- 跨平台 CI：验证三大平台构建与基本用例。

### 安全与隐私
- 离线处理，不上传数据；遵循最小权限原则（文件、进程）。
- 临时文件与缓冲区及时清理；错误不泄露敏感路径。

### 路线图（对应 requirements）
1. 第一阶段（优先）：STT 文件转录稳定版 + 音频预处理（ez-ffmpeg）。
2. 第二阶段：实时转录（流式接口、缓冲/重叠/合并策略、可选 VAD）。
3. 第三阶段：TTS 集成（Index-TTS），保持与未来引擎兼容的接口形态。
4. 后续：性能优化、多引擎并存、WASM/移动端探索。

### 使用与集成（接口级概览）
- 仅 STT：依赖 `stt`；调用 `transcribe_file` 或 `transcribe_samples`。
- 仅 TTS：依赖 `tts`；调用 `synthesize_to_memory` 或 `synthesize_to_file`。
- 仅音频处理：依赖 `audio`；调用 `probe` / `ensure_whisper_compatible`。

以上设计遵循“简洁、通用、易集成”的核心价值，并以库的形态服务于上层工程；同时对实时与多引擎演进提供清晰边界和可持续路径。文档中外部依赖 `ez-ffmpeg` 参考其官方仓库以确保一致性与可维护性（[链接](https://github.com/YeautyYE/ez-ffmpeg)）。


