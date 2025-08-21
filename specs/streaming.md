Streaming 使用说明
=================

目标：在保证实时性的前提下提升稳定性，减少重复与抖动。

接口与配置
-----------
- 入口：`StreamingTranscriber`
- 关键配置：
  - `buffer_duration`：缓冲的最大时长（秒）
  - `transcription_interval`：转录触发周期（毫秒）
  - `min_audio_length`：最小可转录音频时长（毫秒）
  - `enable_vad` / `vad_threshold`：是否启用 VAD 与阈值
  - `local_agreement_n`：LocalAgreement 窗口大小 n（≥2）

示例命令
--------

```bash
cargo run -p stt --features streaming --example streaming_transcribe -- \
  fixtures/models/ggml-tiny.bin fixtures/audio/jfk.wav \
  --chunk-ms=500 --n=3

# 关闭 VAD：
cargo run -p stt --features streaming --example streaming_transcribe -- \
  fixtures/models/ggml-tiny.bin fixtures/audio/jfk.wav --no-vad
```

策略建议
--------
- 块大小（chunk-ms）：
  - 300–700ms 较平衡；过小会增加开销，过大增加延迟
- LocalAgreement-n：
  - 2/3 常用；更大 n 更稳定但输出更慢
- VAD 阈值：
  - 0.008–0.02 常见区间，需结合音源噪声调整

实现要点
--------
- 通过 mpsc 通道解耦音频入队与推理，避免阻塞
- LocalAgreement-n 前缀一致性：仅输出新增被确认的前缀
- 滑动窗口缓冲，转录完成后可根据静音超时清空缓冲

下一步
------
- 引入更精确的 VAD（如 webrtc-vad）
- 智能拼接段落与标点修复
- 输出时间戳与增量标记


