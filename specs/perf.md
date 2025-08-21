性能基线（STT）
===============

说明：记录在本机环境对示例音频的基准结果（平均 RTF、耗时）。

环境：
- 设备：Apple Silicon
- OS：macOS 14+
- Rust：stable
- 模型：ggml-tiny.bin（Whisper）
- 音频：fixtures/audio/jfk.wav

测试命令：

```bash
cargo run -p stt --example bench_transcribe -- fixtures/models/ggml-tiny.bin fixtures/audio/jfk.wav 5
```

输出指标：
- 平均 RTF：处理时间 / 音频时长
- 平均耗时：每次转录耗时的平均值（毫秒）

下一步：
- 补充 base/small 模型的对比
- 记录多线程配置、CPU 频率、持续运行状态下的性能波动
- 加入 CI 侧的回归提醒（可选）


