# Streaming (流式转录) 使用指南

本文档介绍如何使用 rs-voice-toolkit 的流式转录功能进行实时语音识别。

## 概述

流式转录模块提供实时语音转文本功能，支持：
- 实时音频流处理
- 可选的语音活动检测 (VAD)
- 滑动窗口和重叠策略
- 异步事件驱动架构
- 可配置的转录参数

## 基本用法

### 创建流式转录器

```rust
use stt::{create_streaming_transcriber, StreamingConfig, AudioConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model_path = PathBuf::from("path/to/whisper-model.bin");
    
    // 使用默认配置
    let mut transcriber = create_streaming_transcriber(
        model_path,
        AudioConfig::whisper_optimized(),
    )?;
    
    // 启动流式转录
    let mut event_receiver = transcriber.start_streaming().await?;
    
    Ok(())
}
```

### 自定义配置

```rust
use stt::{create_custom_streaming_transcriber, StreamingConfig};
use std::time::Duration;

let custom_config = StreamingConfig {
    enable_vad: true,                                    // 启用语音活动检测
    vad_threshold: 0.5,                                 // VAD 阈值
    buffer_duration: Duration::from_secs(30),           // 音频缓冲区时长
    min_audio_length: Duration::from_millis(500),       // 最小音频长度
    transcription_interval: Duration::from_millis(500), // 转录间隔
    silence_timeout: Duration::from_secs(2),            // 静音超时
    local_agreement_n: 3,                               // 本地一致性确认次数
    ..Default::default()
};

let mut transcriber = create_custom_streaming_transcriber(
    model_path,
    custom_config,
    AudioConfig::whisper_optimized(),
)?;
```

## 配置参数详解

### StreamingConfig 参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `enable_vad` | `bool` | `true` | 是否启用语音活动检测 |
| `vad_threshold` | `f32` | `0.5` | VAD 检测阈值 (0.0-1.0) |
| `buffer_duration` | `Duration` | `30s` | 音频缓冲区最大时长 |
| `min_audio_length` | `Duration` | `1s` | 触发转录的最小音频长度 |
| `transcription_interval` | `Duration` | `1s` | 转录检查间隔 |
| `silence_timeout` | `Duration` | `3s` | 静音超时时间 |
| `local_agreement_n` | `usize` | `3` | 本地一致性确认次数 |

### VAD (语音活动检测)

```rust
// 启用 VAD，跳过静音段
let config = StreamingConfig {
    enable_vad: true,
    vad_threshold: 0.3,  // 较低阈值，更敏感
    ..Default::default()
};

// 禁用 VAD，处理所有音频
let config = StreamingConfig {
    enable_vad: false,
    ..Default::default()
};
```

### 分块策略

```rust
// 快速响应配置（低延迟）
let low_latency_config = StreamingConfig {
    min_audio_length: Duration::from_millis(500),
    transcription_interval: Duration::from_millis(300),
    local_agreement_n: 1,  // 单次确认
    ..Default::default()
};

// 高质量配置（高准确率）
let high_quality_config = StreamingConfig {
    min_audio_length: Duration::from_secs(2),
    transcription_interval: Duration::from_secs(1),
    local_agreement_n: 5,  // 多次确认
    ..Default::default()
};
```

## 事件处理

### StreamingEvent 类型

```rust
use stt::StreamingEvent;

while let Some(event) = event_receiver.recv().await {
    match event {
        StreamingEvent::Transcription(result) => {
            println!("转录结果: {}", result.text);
            println!("置信度: {:.3}", result.confidence);
            println!("实时因子: {:.3}", result.real_time_factor());
        },
        StreamingEvent::SpeechStart => {
            println!("检测到语音开始");
        },
        StreamingEvent::SpeechEnd => {
            println!("语音结束");
        },
        StreamingEvent::Silence => {
            println!("检测到静音");
        },
        StreamingEvent::Error(err) => {
            eprintln!("转录错误: {}", err);
        },
    }
}
```

## 音频输入

### 推送音频数据

```rust
// 从文件读取音频
use stt::audio::utils::read_wav_file;

let audio = read_wav_file("input.wav")?;
let samples = audio.samples;

// 分块推送
let chunk_size = 8000; // 0.5秒 @ 16kHz
for chunk in samples.chunks(chunk_size) {
    transcriber.push_audio(chunk)?;
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

### 实时音频流

```rust
// 模拟实时音频流
loop {
    let audio_chunk = capture_audio_from_microphone(); // 自定义函数
    transcriber.push_audio(&audio_chunk)?;
    tokio::time::sleep(Duration::from_millis(50)).await;
}
```

## 完整示例

```rust
use stt::{
    create_custom_streaming_transcriber, 
    StreamingConfig, 
    StreamingEvent, 
    AudioConfig
};
use stt::audio::utils::read_wav_file;
use std::path::PathBuf;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置
    let model_path = PathBuf::from("models/ggml-tiny.bin");
    let audio_path = PathBuf::from("test.wav");
    
    let config = StreamingConfig {
        enable_vad: true,
        vad_threshold: 0.3,
        min_audio_length: Duration::from_millis(800),
        transcription_interval: Duration::from_millis(500),
        local_agreement_n: 2,
        ..Default::default()
    };
    
    // 创建转录器
    let mut transcriber = create_custom_streaming_transcriber(
        model_path,
        config,
        AudioConfig::whisper_optimized(),
    )?;
    
    // 启动流式转录
    let mut event_receiver = transcriber.start_streaming().await?;
    
    // 启动事件处理任务
    let event_handler = tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            match event {
                StreamingEvent::Transcription(result) => {
                    if !result.text.trim().is_empty() {
                        println!("[转录] {} (置信度: {:.3})", 
                                result.text, result.confidence);
                    }
                },
                StreamingEvent::SpeechStart => println!("[事件] 语音开始"),
                StreamingEvent::SpeechEnd => println!("[事件] 语音结束"),
                StreamingEvent::Silence => println!("[事件] 静音"),
                StreamingEvent::Error(err) => eprintln!("[错误] {}", err),
            }
        }
    });
    
    // 读取并推送音频
    let audio = read_wav_file(&audio_path)?;
    let chunk_size = 8000; // 0.5秒块
    
    for chunk in audio.samples.chunks(chunk_size) {
        transcriber.push_audio(chunk)?;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // 等待处理完成
    tokio::time::sleep(Duration::from_secs(3)).await;
    transcriber.stop_streaming();
    
    // 等待事件处理完成
    let _ = event_handler.await;
    
    Ok(())
}
```

## 性能优化建议

### 延迟优化

1. **减少转录间隔**：设置较短的 `transcription_interval`
2. **降低最小音频长度**：减少 `min_audio_length`
3. **减少确认次数**：设置 `local_agreement_n = 1`
4. **启用 VAD**：跳过静音段，减少不必要的推理

```rust
let low_latency_config = StreamingConfig {
    transcription_interval: Duration::from_millis(200),
    min_audio_length: Duration::from_millis(400),
    local_agreement_n: 1,
    enable_vad: true,
    vad_threshold: 0.4,
    ..Default::default()
};
```

### 准确率优化

1. **增加音频长度**：提供更多上下文
2. **多次确认**：增加 `local_agreement_n`
3. **调整 VAD 阈值**：避免截断语音

```rust
let high_accuracy_config = StreamingConfig {
    min_audio_length: Duration::from_secs(2),
    local_agreement_n: 5,
    vad_threshold: 0.2,  // 更保守的阈值
    ..Default::default()
};
```

### 资源优化

1. **控制缓冲区大小**：避免内存过度使用
2. **合理设置超时**：及时清理静音段
3. **选择合适的模型**：平衡速度和准确率

```rust
let resource_optimized_config = StreamingConfig {
    buffer_duration: Duration::from_secs(15),  // 较小缓冲区
    silence_timeout: Duration::from_secs(1),   // 快速超时
    ..Default::default()
};
```

## 故障排除

### 常见问题

1. **转录延迟过高**
   - 检查 `transcription_interval` 设置
   - 确认模型大小是否合适
   - 验证硬件性能

2. **转录结果不准确**
   - 增加 `min_audio_length`
   - 提高 `local_agreement_n`
   - 检查音频质量和采样率

3. **频繁的语音开始/结束事件**
   - 调整 VAD 阈值
   - 增加 `silence_timeout`
   - 检查音频中的噪声

4. **内存使用过高**
   - 减少 `buffer_duration`
   - 检查音频推送频率
   - 确认及时调用 `stop_streaming()`

### 调试技巧

```rust
// 启用详细日志
env_logger::init();

// 监控性能指标
let result = transcriber.get_stats(); // 假设的 API
println!("缓冲区使用: {}%", result.buffer_usage);
println!("平均延迟: {}ms", result.avg_latency);
```

## 最佳实践

1. **根据场景选择配置**：实时对话 vs 文件转录
2. **合理设置缓冲区**：平衡内存使用和延迟
3. **监控性能指标**：RTF、延迟、内存使用
4. **处理错误事件**：实现重试和降级策略
5. **及时清理资源**：调用 `stop_streaming()` 释放资源

## 集成示例

### Web 服务集成

```rust
// 使用 axum 创建 WebSocket 端点
use axum::{extract::ws::WebSocket, response::Response};

async fn websocket_handler(socket: WebSocket) {
    // 创建流式转录器
    let mut transcriber = create_streaming_transcriber(/* ... */)?;
    let mut event_receiver = transcriber.start_streaming().await?;
    
    // 处理 WebSocket 消息和转录事件
    // ...
}
```

### 命令行工具

```rust
// 实时麦克风转录
use cpal::{traits::*, Stream};

fn main() {
    let device = cpal::default_host().default_input_device().unwrap();
    let config = device.default_input_config().unwrap();
    
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // 推送音频到转录器
            transcriber.push_audio(data).unwrap();
        },
        |err| eprintln!("音频流错误: {}", err),
        None,
    ).unwrap();
    
    stream.play().unwrap();
    // ...
}
```

---

*最后更新: 2024年12月*
*版本: v0.1.0*