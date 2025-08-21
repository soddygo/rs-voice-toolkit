## 开发任务清单（Tasks）

本清单对齐 `specs/requirements.md` 与 `specs/design.md`，以“简洁、通用、易集成”为目标，按阶段推进。所有模块为独立库（workspace 子工程）。

说明：
- 状态标记：[ ] 待办、[~] 进行中、[x] 完成
- 优先级：P0（最高）、P1、P2、P3
- 若无特别说明，默认目标平台为 macOS、Linux、Windows，CI 全平台验证

---

### 里程碑与优先级总览
- 里程碑 M1（P0）：STT 文件转录 MVP 可用、离线运行、核心 API 稳定
- 里程碑 M2（P1）：实时转录（基础流式接口、稳定性达标）
- 里程碑 M3（P1）：TTS（Index-TTS 集成，接口稳定）
- 里程碑 M4（P2）：性能与跨平台完善、发布与文档

---

### 基础准备与工程化
- [x] P0 工程初始化与依赖梳理（workspace 子项目：`stt/`、`audio/`、`tts/`、`specs/`）
  - 验收：`cargo build` 在本地成功；workspace 成员与文档一致（已移除 `video`）
- [x] P0 统一日志方案与最小错误模型骨架（`thiserror` + `log`）
  - 验收：核心模块暴露 `Result<_, Error>`；无 panic；有基础日志
- [x] P0 Code style：`rustfmt`、`clippy` 通过；基础 `.editorconfig`
  - 验收：CI 中 `cargo fmt -- --check` / `cargo clippy -D warnings` 通过（已接入 fmt 与 clippy）

---

### 模块：audio（基于 ez-ffmpeg 的轻量封装）
- [x] P0 集成 `ez-ffmpeg`，提供最小 API：`probe`、`ensure_whisper_compatible`
  - 依赖：系统安装 FFmpeg（macOS: `brew install ffmpeg`；Windows: vcpkg；Linux: 发行版包）
  - 参考：`ez-ffmpeg` 官方仓库（`https://github.com/YeautyYE/ez-ffmpeg`）
  - 验收：输入 mp3/aac/flac/m4a/wav → 输出 mono/16k/WAV（PCM16），单测覆盖（已接入转换流程，待补单测）
- [x] P1 内存流重采样（可选），避免文件落地（为实时做准备）
  - 验收：提供 samples → samples 的重采样函数，采样率/通道正确
  - 状态：已使用 rubato 库实现高质量重采样，支持流式处理
- [x] P1 鲁棒性：错误分类（解封装失败、编解码不支持、采样率不匹配）
  - 验收：单测覆盖常见失败路径，错误信息可读

---

### 模块：stt（Whisper 基础文件转录 MVP）
- [x] P0 `SttConfig` 与默认值（模型路径、语言自动检测、n_threads 等）
  - 验收：`Default` 可用；序列化/反序列化基本用例通过
- [x] P0 `transcribe_file(path, model_path, [config])`
  - 依赖：`audio::ensure_whisper_compatible`
  - 验收：编译通过，接口可用；后续补充端到端样例验证
- [x] P0 `transcribe_samples(samples, sample_rate, [config])`
  - 验收：编译通过，接口可用；后续补充端到端样例验证
- [x] P0 端到端样例（docs 或 examples）：文件 → 文本
  - 验收：示例能在三平台构建与运行（具备离线模型）（已添加 `stt/examples/transcribe_file.rs`，已验证可用）
- [ ] P1 性能与资源基线（小模型/量化模型、RTF、内存）
  - 验收：记录指标，形成性能基线文档
  - 进展：已添加示例 `stt/examples/bench_transcribe.rs`（输出平均 RTF/耗时）
  - 文档：新增 `specs/perf.md`，记录基准方法与后续计划

---

### 模块：stt（实时转录基础）
- [~] P1 `StreamingTranscriber` 原型（chunk 接口、滑动窗口、重叠策略）
  - 验收：固定 0.5s 输入、0.2s 重叠；产出稳定的部分文本或空
- [x] P1 结果合并与 LocalAgreement-n（n 可调），防重复与截断
  - 验收：合并策略单测覆盖，边界段处理合理（已添加 `StreamingAggregator` 与单测）
- [ ] P1 可选 VAD 开关，跳过静音段
  - 验收：静音段显著减少推理次数；延迟可控
- [ ] P1 内部异步解耦（采集/输入 vs 推理），避免阻塞
  - 验收：在高吞吐输入下无明显堆积；有背压与降级策略
  - 进展：已添加示例 `stt/examples/streaming_transcribe.rs`（支持 --no-vad / --n / --chunk-ms）；输入已通过 mpsc 通道解耦
  - 文档：新增 `specs/streaming.md`，补充参数与策略建议

---

### 模块：tts（Index-TTS 第一实现）
- [x] P1 `TtsConfig`（`executable_path`、`language`、`speaker`、`sample_rate` 默认）
  - 验收：若未显式提供路径，可查找 PATH；不可用时给出清晰错误
- [x] P1 `synthesize_to_memory(text, [config])`（调用外部 `index-tts`）
  - 验收：返回 WAV 字节，默认 22050Hz；含基本异常处理与日志
- [x] P1 `synthesize_to_file(text, output_path, [config])`
  - 验收：在目标路径生成合法 WAV；若覆盖/目录缺失有明确错误
- [ ] P2 可扩展引擎边界（类型枚举与构造），仅保留最薄抽象，避免过度设计
  - 验收：接口可并存 Piper/Coqui，当前仅注册 Index-TTS

---

### 跨模块文档与示例
- [x] P0 README（workspace 根）：定位、模块说明、快速开始（含 FFmpeg 与模型准备）
  - 验收：`cargo build` 前置说明清晰，三平台安装指引
- [x] P0 stt 用法文档：文件/内存转录示例、性能建议
  - 验收：可复制粘贴运行；有常见错误排查（已创建 `docs/stt-usage.md`）
- [ ] P1 streaming 用法文档：分块策略、窗口、VAD、延迟建议
  - 验收：描述完整、示例可运行
- [ ] P1 tts 用法文档：Index-TTS 安装与调用、输出格式
  - 验收：示例可运行（若未安装则文档注明跳过方式）

---

### 测试与质量保障
- [x] P0 单元测试：audio/stt 覆盖核心路径与错误路径
  - 验收：`cargo test` 通过；覆盖率基线记录
- [x] P0 CI（GitHub Actions 或等效）：三平台 matrix、fmt/clippy/test
  - 验收：PR 必须绿；缓存依赖避免冗长构建（已添加工作流并通过构建）
- [ ] P1 集成测试：端到端 文件→文本；文本→音频（在存在 index-tts 时）
  - 验收：按标签可跳过 TTS 测试；文档说明如何启用
- [ ] P1 性能测试：记录 RTF、内存、延迟；形成历史比较
  - 验收：有基线与波动阈值；回归能被发现

---

### 交付与发布
- [ ] P1 版本化与语义化（各子库）：`0.1.0` 起步，Changelog 维护
  - 验收：发布前 checklist（构建、测试、示例、文档）
- [ ] P2 License/版权与模型许可提示（README 与 crate 元数据）
  - 验收：显式声明；遵循依赖方许可
- [ ] P2 crates.io 发布准备（可选）：描述、关键词、仓库主页、示例链接
  - 验收：`cargo publish --dry-run` 通过

---

### 运维与支持（可选）
- [ ] P2 错误上报与最小诊断信息（日志等级/字段规范）
  - 验收：用户能快速定位常见问题（依赖未装、路径错误、格式不支持）
- [ ] P3 讨论/Issue 模板：Bug/Feature/Question 模板
  - 验收：新建 Issue 指南可用

---

### 启动建议（日常开发流程）
1) 拉取依赖并构建：`cargo build`（三平台确保通过）
2) 本地跑测试：`cargo test`；修复 clippy 提示
3) 开发顺序建议：audio P0 → stt 文件转录 P0 → 文档示例 P0 → streaming P1 → tts P1
4) 每完成一小步：更新文档与变更记录；提交 PR 走 CI


