# VAD (语音活动检测) 使用指南

本文档介绍如何在 STT 模块中使用 VAD (Voice Activity Detection) 功能来跳过静音段，提高转录效率。

## 概述

VAD 功能可以自动检测音频中的语音活动，跳过静音段的转录处理，从而：

- 减少不必要的计算开销
- 提高转录效率
- 避免处理纯静音音频
- 节省系统资源

## 基本用法

### 1. 在 WhisperConfig 中启用 VAD

```rust
use stt::whisper::WhisperConfig;

// 启用 VAD 并设置阈值
let config = WhisperConfig::new("path/to/model.bin")
    .with_vad(true)                    // 启用 VAD
    .with_vad_threshold(0.01);         // 设置 VAD 阈值

// 或者使用链式调用
let config = WhisperConfig::new("path/to/model.bin")
    .with_language("zh".to_string())
    .with_vad(true)
    .with_vad_threshold(0.005);        // 更敏感的阈值
```

### 2. 在流式转录中使用 VAD

```rust
use stt::streaming::{StreamingConfig, create_custom_streaming_transcriber};
use stt::audio::AudioConfig;
use std::time::Duration;

// 创建流式转录配置
let streaming_config = StreamingConfig {
    enable_vad: true,
    vad_threshold: 0.005,
    transcription_interval: Duration::from_millis(500),
    ..Default::default()
};

// 创建转录器
let transcriber = create_custom_streaming_transcriber(
    "path/to/model.bin",
    streaming_config,
    AudioConfig::whisper_optimized(),
)?;
```

## VAD 配置参数

### enable_vad

- **类型**: `bool`
- **默认值**: `false`
- **说明**: 是否启用 VAD 功能

```rust
// 启用 VAD
config.with_vad(true);

// 禁用 VAD（默认）
config.with_vad(false);
```

### vad_threshold

- **类型**: `f32`
- **范围**: `0.0 - 1.0`
- **默认值**: `0.01`
- **说明**: VAD 检测阈值，值越小越敏感

```rust
// 高敏感度（容易检测到语音）
config.with_vad_threshold(0.001);

// 中等敏感度（推荐）
config.with_vad_threshold(0.01);

// 低敏感度（只检测明显的语音）
config.with_vad_threshold(0.05);
```

## 阈值选择指南

### 环境因素

| 环境类型 | 推荐阈值 | 说明 |
|---------|---------|------|
| 安静环境 | 0.005 - 0.01 | 背景噪音很少，可以使用较低阈值 |
| 一般环境 | 0.01 - 0.02 | 有一定背景噪音，使用中等阈值 |
| 嘈杂环境 | 0.02 - 0.05 | 背景噪音较多，需要较高阈值 |
| 极嘈杂环境 | 0.05+ | 或考虑禁用 VAD |

### 音频特征

| 音频特征 | 推荐阈值 | 说明 |
|---------|---------|------|
| 清晰语音 | 0.005 - 0.01 | 语音清晰，可以使用较低阈值 |
| 轻声说话 | 0.001 - 0.005 | 音量较小，需要更敏感的检测 |
| 大声说话 | 0.01 - 0.03 | 音量较大，可以使用较高阈值 |
| 音乐/歌声 | 0.005 - 0.02 | 根据音乐类型调整 |

## 实际示例

### 示例 1: 文件转录with VAD

```rust
use stt::whisper::{WhisperConfig, WhisperTranscriber};
use stt::audio::utils::read_wav_file;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建启用 VAD 的配置
    let config = WhisperConfig::new("models/ggml-base.bin")
        .with_language("zh".to_string())
        .with_vad(true)
        .with_vad_threshold(0.01);
    
    // 创建转录器
    let transcriber = WhisperTranscriber::new(config)?;
    
    // 读取音频文件
    let audio_data = read_wav_file("audio/speech.wav")?;
    
    // 执行转录
    let result = transcriber.transcribe_audio_data(&audio_data).await?;
    
    if result.text.trim().is_empty() {
        println!("检测到静音，跳过转录");
    } else {
        println!("转录结果: {}", result.text);
    }
    
    Ok(())
}
```

### 示例 2: 动态调整 VAD 阈值

```rust
use stt::whisper::{WhisperConfig, WhisperTranscriber};
use stt::vad::SimpleVad;

// 预检测音频中的语音活动
fn detect_optimal_threshold(audio_samples: &[f32]) -> f32 {
    let test_thresholds = [0.001, 0.005, 0.01, 0.02, 0.05];
    
    for &threshold in &test_thresholds {
        let vad = SimpleVad::new(threshold);
        if vad.detect_speech(audio_samples) {
            return threshold;
        }
    }
    
    0.001 // 如果都检测不到，使用最敏感的阈值
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let audio_data = stt::audio::utils::read_wav_file("audio/speech.wav")?;
    
    // 动态检测最佳阈值
    let optimal_threshold = detect_optimal_threshold(&audio_data.samples);
    println!("使用 VAD 阈值: {}", optimal_threshold);
    
    // 创建配置
    let config = WhisperConfig::new("models/ggml-base.bin")
        .with_vad(true)
        .with_vad_threshold(optimal_threshold);
    
    let transcriber = WhisperTranscriber::new(config)?;
    let result = transcriber.transcribe_audio_data(&audio_data).await?;
    
    println!("转录结果: {}", result.text);
    Ok(())
}
```

## 性能影响

### VAD 启用时的性能特征

- **CPU 使用**: VAD 检测本身消耗很少的 CPU 资源
- **内存使用**: 几乎无额外内存开销
- **延迟**: VAD 检测延迟通常 < 1ms
- **准确性**: 可能会误判极轻声的语音为静音

### 性能优化建议

1. **合理设置阈值**: 避免过于敏感或过于迟钝
2. **结合环境调整**: 根据实际使用环境调整参数
3. **监控误判率**: 定期检查 VAD 的准确性
4. **备用方案**: 对重要音频可以禁用 VAD 确保完整转录

## 故障排除

### 常见问题

#### 1. VAD 过于敏感，静音也被检测为语音

**解决方案**:
- 提高 `vad_threshold` 值
- 检查音频是否有背景噪音
- 考虑音频预处理（降噪）

```rust
// 提高阈值
config.with_vad_threshold(0.02); // 从 0.01 提高到 0.02
```

#### 2. VAD 过于迟钝，轻声语音被跳过

**解决方案**:
- 降低 `vad_threshold` 值
- 检查音频音量是否过低
- 考虑音频预处理（音量标准化）

```rust
// 降低阈值
config.with_vad_threshold(0.005); // 从 0.01 降低到 0.005
```

#### 3. VAD 检测不稳定

**解决方案**:
- 检查音频质量和格式
- 确保音频采样率为 16kHz
- 使用单声道音频
- 考虑音频预处理

```rust
// 确保音频格式正确
let audio_config = AudioConfig {
    sample_rate: 16000,
    channels: 1,
    format: AudioFormat::F32,
};
```

### 调试技巧

#### 1. 启用详细日志

```rust
env_logger::init();
// 或
env::set_var("RUST_LOG", "debug");
```

#### 2. 手动测试 VAD

```rust
use stt::vad::SimpleVad;

let vad = SimpleVad::new(0.01);
let has_speech = vad.detect_speech(&audio_samples);
println!("VAD 检测结果: {}", has_speech);

// 检测语音段
let segments = vad.detect_speech_segments(&audio_samples);
println!("检测到 {} 个语音段", segments.len());
for (start, end) in segments {
    println!("语音段: {} - {} 样本", start, end);
}
```

## 最佳实践

1. **测试不同阈值**: 在实际使用环境中测试多个阈值值
2. **监控性能**: 定期检查 VAD 的准确性和性能影响
3. **环境适配**: 根据不同使用场景调整配置
4. **备用策略**: 为重要音频提供禁用 VAD 的选项
5. **用户控制**: 允许用户自定义 VAD 参数

## 相关文档

- [STT 使用指南](stt-usage.md)
- [流式转录文档](streaming-transcription.md)
- [性能基线文档](stt-performance-baseline.md)