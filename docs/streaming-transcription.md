# 流式转录 (Streaming Transcription)

本文档介绍如何使用 STT 模块的流式转录功能，实现实时语音识别。

## 概述

流式转录允许您实时处理音频流，无需等待完整音频文件。它支持：

- 实时音频处理
- 语音活动检测 (VAD)
- 增量转录结果
- 可配置的缓冲和聚合策略

## 基本用法

### 1. 创建流式转录器

```rust
use rs_voice_toolkit_stt::streaming::{
    StreamingTranscriber, StreamingConfig, 
    create_streaming_transcriber, create_custom_streaming_transcriber
};
use rs_voice_toolkit_stt::audio::AudioConfig;
use rs_voice_toolkit_stt::whisper::WhisperConfig;
use std::time::Duration;

// 使用默认配置（推荐）
let mut transcriber = create_streaming_transcriber("fixtures/models/ggml-tiny.bin")?;

// 或使用自定义配置
let whisper_config = WhisperConfig::new("fixtures/models/ggml-tiny.bin");
let streaming_config = StreamingConfig {
    buffer_duration: Duration::from_secs(10),
    transcription_interval: Duration::from_millis(500),
    min_audio_length: Duration::from_millis(100),
    max_audio_length: Duration::from_secs(30),
    enable_vad: true,
    vad_threshold: 0.005,
    silence_timeout: Duration::from_secs(2),
    local_agreement_n: 3,
};
let audio_config = AudioConfig::whisper_optimized();

let mut transcriber = StreamingTranscriber::new(
    whisper_config,
    streaming_config,
    audio_config,
)?;
```

### 2. 启动流式转录

```rust
use rs_voice_toolkit_stt::streaming::{StreamingEvent, create_streaming_transcriber};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut transcriber = create_streaming_transcriber("fixtures/models/ggml-tiny.bin")?;
    
    // 启动流式转录
    let mut event_rx = transcriber.start_streaming().await?;
    
    // 处理转录事件
    let event_handler = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                StreamingEvent::Transcription(result) => {
                    println!("转录结果: {}", result.text);
                }
                StreamingEvent::SpeechStart => {
                    println!("检测到语音开始");
                }
                StreamingEvent::SpeechEnd => {
                    println!("检测到语音结束");
                }
                StreamingEvent::Silence => {
                    println!("检测到静音");
                }
                StreamingEvent::Error(err) => {
                    eprintln!("转录错误: {}", err);
                }
            }
        }
    });
    
    // 推送音频数据（f32 格式）
    let audio_samples: Vec<f32> = load_audio_samples(); // 您的音频数据
    transcriber.push_audio(&audio_samples)?;
    
    // 等待处理完成
    sleep(Duration::from_secs(5)).await;
    
    // 停止转录
    transcriber.stop_streaming();
    
    // 等待事件处理完成
    let _ = event_handler.await;
    
    Ok(())
}
```

### 3. 处理音频流

```rust
// 处理 f32 格式的音频样本（推荐）
transcriber.push_audio(&f32_samples)?;

// VAD 控制方法
transcriber.set_vad_enabled(true);  // 启用 VAD
transcriber.set_vad_threshold(0.01); // 设置 VAD 阈值
let vad_enabled = transcriber.is_vad_enabled(); // 检查 VAD 状态

// 获取缓冲区状态
let (duration, sample_count) = transcriber.buffer_info();
println!("缓冲区: {:.2}s, {} 样本", duration.as_secs_f64(), sample_count);

// 清空缓冲区
transcriber.clear_buffer();
```

## 配置选项

### StreamingConfig

```rust
pub struct StreamingConfig {
    /// 缓冲区最大持续时间
    pub buffer_duration: Duration,
    
    /// 转录间隔
    pub transcription_interval: Duration,
    
    /// 最小音频长度（用于转录）
    pub min_audio_length: Duration,
    
    /// 最大音频长度（用于转录）
    pub max_audio_length: Duration,
    
    /// 启用语音活动检测 (VAD)
    pub enable_vad: bool,
    
    /// VAD 阈值，用于检测语音/静音
    pub vad_threshold: f32,
    
    /// 静音超时时间
    pub silence_timeout: Duration,
    
    /// 本地一致性检查的窗口大小
    pub local_agreement_n: usize,
}
```

### 默认配置

```rust
impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            buffer_duration: Duration::from_secs(10),
            transcription_interval: Duration::from_millis(500),
            min_audio_length: Duration::from_millis(100),
            max_audio_length: Duration::from_secs(30),
            enable_vad: true,
            vad_threshold: 0.005,
            silence_timeout: Duration::from_secs(2),
            local_agreement_n: 3,
        }
    }
}
```

## 事件类型

### StreamingEvent

```rust
pub enum StreamingEvent {
    /// 转录结果
    Transcription(TranscriptionResult),
    
    /// 检测到语音开始
    SpeechStart,
    
    /// 检测到语音结束
    SpeechEnd,
    
    /// 检测到静音
    Silence,
    
    /// 转录错误
    Error(String),
}

pub struct TranscriptionResult {
    /// 转录文本
    pub text: String,
    
    /// 置信度分数
    pub confidence: f32,
    
    /// 音频时长
    pub duration: Duration,
    
    /// 时间戳
    pub timestamp: std::time::Instant,
    
    /// 语言检测结果（可选）
    pub language: Option<String>,
}
```

## 性能优化建议

### 1. VAD 配置

```rust
// 对于安静环境，使用更严格的阈值
config.vad_threshold = 0.01;

// 对于嘈杂环境，使用更宽松的阈值
config.vad_threshold = 0.001;

// 禁用 VAD（处理所有音频）
config.enable_vad = false;
```

### 2. 转录间隔

```rust
// 更快的响应（更高的 CPU 使用）
config.transcription_interval = Duration::from_millis(250);

// 更低的 CPU 使用（稍慢的响应）
config.transcription_interval = Duration::from_millis(1000);
```

### 3. 本地一致性

```rust
// 更快的输出（可能不太稳定）
config.local_agreement_n = 1;

// 更稳定的输出（稍慢）
config.local_agreement_n = 3;
```

### 4. 缓冲区管理

```rust
// 较小的缓冲区（更低的内存使用）
config.buffer_duration = Duration::from_secs(5);

// 较大的缓冲区（更好的上下文）
config.buffer_duration = Duration::from_secs(15);
```

## 示例：文件流式转录

```rust
use rs_voice_toolkit_stt::streaming::{create_streaming_transcriber, StreamingEvent};
use rs_voice_toolkit_stt::audio::load_audio_file;
use tokio::time::{sleep, Duration};
use hound::WavReader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut transcriber = create_streaming_transcriber("fixtures/models/ggml-tiny.bin")?;
    let mut event_rx = transcriber.start_streaming().await?;
    
    // 启动事件处理任务
    let event_handler = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                StreamingEvent::Transcription(result) => {
                    println!("转录: {} (置信度: {:.2})", result.text, result.confidence);
                }
                StreamingEvent::SpeechStart => {
                    println!("语音开始");
                }
                StreamingEvent::SpeechEnd => {
                    println!("语音结束");
                }
                StreamingEvent::Silence => {
                    println!("静音检测");
                }
                StreamingEvent::Error(err) => {
                    eprintln!("错误: {}", err);
                    break;
                }
            }
        }
    });
    
    // 读取 WAV 文件并分块推送
    let mut reader = WavReader::open("fixtures/audio/jfk.wav")?;
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / 32768.0)
        .collect();
    
    // 分块推送音频（模拟实时流）
    let chunk_size = 1600; // 100ms @ 16kHz
    for chunk in samples.chunks(chunk_size) {
        transcriber.push_audio(chunk)?;
        sleep(Duration::from_millis(100)).await; // 模拟实时流
    }
    
    // 等待处理完成
    sleep(Duration::from_secs(2)).await;
    
    // 停止转录
    transcriber.stop_streaming();
    
    // 等待事件处理完成
    let _ = event_handler.await;
    
    Ok(())
}
```

## 故障排除

### 无转录输出

1. **检查 VAD 设置**：如果启用了 VAD，可能音频被误判为静音
   ```rust
   // 临时禁用 VAD 进行测试
   transcriber.set_vad_enabled(false);
   
   // 或调整 VAD 阈值
   transcriber.set_vad_threshold(0.001); // 更敏感
   ```

2. **检查音频格式**：确保音频是单声道 16kHz f32 格式
   ```rust
   // 检查缓冲区状态
   let (duration, sample_count) = transcriber.buffer_info();
   println!("缓冲区: {:.2}s, {} 样本", duration.as_secs_f64(), sample_count);
   ```

3. **检查最小音频长度**：音频片段可能太短
   ```rust
   let config = StreamingConfig {
       min_audio_length: Duration::from_millis(50),
       ..Default::default()
   };
   ```

### 转录延迟过高

1. **减少转录间隔**：
   ```rust
   let config = StreamingConfig {
       transcription_interval: Duration::from_millis(250),
       ..Default::default()
   };
   ```

2. **调整缓冲区大小**：
   ```rust
   let config = StreamingConfig {
       buffer_duration: Duration::from_secs(5),
       ..Default::default()
   };
   ```

3. **检查系统资源**：确保 CPU 和内存充足

### 内存使用过高

1. **减少缓冲区持续时间**
   ```rust
   let config = StreamingConfig {
       buffer_duration: Duration::from_secs(5), // 减少到 5 秒
       ..Default::default()
   };
   ```

2. **增加转录频率以及时清理缓冲区**
3. **定期调用 `clear_buffer()`**
   ```rust
   // 定期清理缓冲区
   if buffer_duration > Duration::from_secs(8) {
       transcriber.clear_buffer();
   }
   ```

### 转录质量差

1. **调整 VAD 阈值**：
   ```rust
   transcriber.set_vad_threshold(0.01); // 更敏感的阈值
   ```

2. **增加本地一致性要求**：
   ```rust
   let config = StreamingConfig {
       local_agreement_n: 5, // 增加一致性要求
       ..Default::default()
   };
   ```

3. **使用更大的模型**：考虑使用 `ggml-base.bin` 或 `ggml-small.bin`

4. **检查音频质量**：确保音频清晰，噪音较少

## 注意事项

### 线程安全

`StreamingTranscriber` 不是线程安全的，如需在多线程环境中使用，请使用适当的同步机制：

```rust
use std::sync::{Arc, Mutex};

let transcriber = Arc::new(Mutex::new(create_streaming_transcriber("model.bin")?));
```

### 异步处理

所有转录操作都是异步的，确保正确处理 `Future` 和 `Stream`：

```rust
// 正确的异步处理
let mut event_rx = transcriber.start_streaming().await?;
let event_handler = tokio::spawn(async move {
    while let Some(event) = event_rx.recv().await {
        // 处理事件
    }
});

// 等待处理完成
let _ = event_handler.await;
```

### 内存管理

- **定期监控缓冲区状态**：
  ```rust
  let (duration, sample_count) = transcriber.buffer_info();
  if duration > Duration::from_secs(10) {
      transcriber.clear_buffer();
  }
  ```

- **合理设置缓冲区大小**：
  ```rust
  let config = StreamingConfig {
      buffer_duration: Duration::from_secs(8), // 根据需要调整
      ..Default::default()
  };
  ```

### 错误处理

始终处理 `StreamingEvent::Error` 事件，并实现适当的错误恢复机制：

```rust
match event {
    StreamingEvent::Error(err) => {
        eprintln!("转录错误: {}", err);
        // 实现错误恢复逻辑
        transcriber.clear_buffer();
        // 或重新启动转录
    }
    _ => {}
}
```

### 资源清理

使用完毕后调用 `stop_streaming()` 确保资源正确释放：

```rust
// 停止转录
transcriber.stop_streaming();

// 等待异步任务完成
let _ = event_handler.await;
```

### 性能建议

1. **选择合适的模型大小**：
   - `ggml-tiny.bin`: 快速，适合实时应用
   - `ggml-base.bin`: 平衡性能和质量
   - `ggml-small.bin`: 更高质量，但速度较慢

2. **调整配置参数**：
   - 实时应用：较短的 `transcription_interval`
   - 批处理：较长的 `buffer_duration`
   - 噪音环境：调整 `vad_threshold`

3. **监控系统资源**：
   - CPU 使用率
   - 内存占用
   - 实时因子 (RTF)