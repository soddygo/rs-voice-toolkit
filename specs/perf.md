性能基线（STT）
===============

说明：记录在本机环境对示例音频的基准结果（平均 RTF、耗时、内存使用等综合指标）。

环境：
- 设备：Apple Silicon
- OS：macOS 14+
- Rust：stable
- 模型：ggml-tiny.bin（Whisper）
- 音频：fixtures/audio/jfk.wav

测试命令：

```bash
# 基本性能测试
cargo run -p stt --example bench_transcribe -- fixtures/models/ggml-tiny.bin fixtures/audio/jfk.wav 5

# 完整性能基线测试（包含内存、置信度等指标）
cargo run -p stt --example performance_baseline -- fixtures/models/ggml-tiny.bin fixtures/audio/jfk.wav 5
```

输出指标：
- 平均 RTF：处理时间 / 音频时长
- 平均耗时：每次转录耗时的平均值（毫秒）
- 内存变化：转录过程中的内存增量（MB）
- 平均置信度：识别结果的置信度评分
- 文本长度：识别文本的字符数
- 段落数：识别结果的段落数量

性能基准：
- **RTF 基准**: <0.3（优秀）, 0.3-1.0（良好）, ≥1.0（需优化）
- **置信度基准**: >0.8（优秀）, 0.6-0.8（良好）, ≤0.6（需优化）
- **内存基准**: <50MB（正常）, 50-100MB（注意）, ≥100MB（需优化）

当前基线（ggml-tiny.bin + jfk.wav）：
- 平均 RTF: 0.019 ✅
- 平均置信度: 0.834 ✅
- 平均内存变化: 14MB ✅

下一步：
- 补充 base/small/medium 模型的对比测试
- 记录多线程配置下的性能表现
- 添加持续运行状态下的性能波动监控
- 建立 CI 性能回归检测机制


