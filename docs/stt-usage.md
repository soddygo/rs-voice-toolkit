# STT (Speech-to-Text) 用法文档

本文档介绍如何使用 `stt` 模块进行语音转文本处理，包括文件转录、内存转录示例以及性能优化建议。

## 目录

- [快速开始](#快速开始)
- [基础用法](#基础用法)
  - [文件转录](#文件转录)
  - [内存转录](#内存转录)
  - [配置选项](#配置选项)
- [高级用法](#高级用法)
  - [自定义配置](#自定义配置)
  - [批量处理](#批量处理)
  - [结果过滤](#结果过滤)
- [性能优化](#性能优化)
- [常见错误排查](#常见错误排查)
- [示例代码](#示例代码)

## 快速开始

### 前置条件

1. **模型文件**: 下载 Whisper 模型文件（如 `ggml-tiny.bin`, `ggml-base.bin` 等）
2. **音频文件**: 支持常见格式（WAV, MP3, FLAC, M4A 等）

### 最简单的使用方式

```rust
use stt::transcribe_file;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model_path = PathBuf::from("models/ggml-base.bin");
    let audio_path = PathBuf::from("audio/speech.wav");
    
    let result = transcribe_file(&model_path, &audio_path).await?;
    
    println!("转录结果: {}", result.text);
    println!("音频时长: {:.2}秒", result.audio_duration as f64 / 1000.0);
    println!("实时因子: {:.3}", result.real_time_factor());
    
    Ok(())
}
```

## 基础用法

### 文件转录

#### 简单文件转录

```rust
use stt::{transcribe_file, transcribe_file_with_language};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> stt::SttResult<()> {
    let model_path = PathBuf::from("models/ggml-base.bin");
    let audio_path = PathBuf::from("audio/chinese_speech.wav");
    
    // 自动检测语言
    let result = transcribe_file(&model_path, &audio_path).await?;
    println!("自动检测结果: {}", result.text);
    
    // 指定语言（提高准确性和速度）
    let result_zh = transcribe_file_with_language(&model_path, &audio_path, "zh").await?;
    println!("中文转录结果: {}", result_zh.text);
    
    Ok(())
}
```

#### 处理转录结果

```rust
use stt::transcribe_file;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> stt::SttResult<()> {
    let result = transcribe_file("models/ggml-base.bin", "audio/speech.wav").await?;
    
    // 基本信息
    println!("完整文本: {}", result.text);
    println!("检测语言: {:?}", result.language);
    println!("段数: {}", result.segments.len());
    
    // 性能指标
    println!("音频时长: {:.2}秒", result.audio_duration as f64 / 1000.0);
    println!("处理时间: {:.2}秒", result.processing_time as f64 / 1000.0);
    println!("实时因子: {:.3}", result.real_time_factor());
    println!("平均置信度: {:.3}", result.average_confidence());
    
    // 分段信息
    for (i, segment) in result.segments.iter().enumerate() {
        println!(
            "段 {}: [{:.2}s - {:.2}s] 置信度:{:.3} 文本: {}",
            i + 1,
            segment.start_time as f64 / 1000.0,
            segment.end_time as f64 / 1000.0,
            segment.confidence,
            segment.text
        );
    }
    
    Ok(())
}
```

### 内存转录

当你已经有音频数据在内存中时，可以直接进行转录而无需文件操作：

```rust
use stt::{WhisperTranscriber, WhisperConfig, AudioData, AudioFormat};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> stt::SttResult<()> {
    // 创建转录器
    let config = WhisperConfig::new("models/ggml-base.bin")
        .with_language("zh")
        .with_threads(4);
    
    let transcriber = WhisperTranscriber::new(config)?;
    
    // 假设你有音频样本数据
    let samples: Vec<f32> = load_audio_samples(); // 你的音频加载函数
    let audio_data = AudioData {
        samples,
        sample_rate: 16000,
        channels: 1,
        format: AudioFormat::F32,
    };
    
    // 转录音频数据
    let result = transcriber.transcribe_audio_data(&audio_data).await?;
    println!("转录结果: {}", result.text);
    
    Ok(())
}

fn load_audio_samples() -> Vec<f32> {
    // 这里应该是你的音频加载逻辑
    // 例如从网络、数据库或其他来源加载
    vec![0.0; 16000] // 示例：1秒的静音
}
```

### 配置选项

`WhisperConfig` 提供了丰富的配置选项：

```rust
use stt::{WhisperConfig, WhisperTranscriber};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> stt::SttResult<()> {
    let config = WhisperConfig::new("models/ggml-base.bin")
        .with_language("zh")           // 指定语言
        .with_translate(false)         // 是否翻译为英文
        .with_threads(8)               // 线程数（根据CPU核心数调整）
        .with_temperature(0.0)         // 温度参数（0.0更确定，1.0更随机）
        .with_initial_prompt("这是一段中文语音"); // 初始提示
    
    let transcriber = WhisperTranscriber::new(config)?;
    
    // 使用配置好的转录器
    let result = transcriber.transcribe_file("audio/speech.wav").await?;
    println!("转录结果: {}", result.text);
    
    Ok(())
}
```

## 高级用法

### 自定义配置

```rust
use stt::{WhisperConfig, WhisperTranscriber};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> stt::SttResult<()> {
    // 高质量配置（适合准确性要求高的场景）
    let high_quality_config = WhisperConfig {
        model_path: PathBuf::from("models/ggml-large.bin"),
        language: Some("zh".to_string()),
        translate: false,
        n_threads: 8,
        print_timestamps: true,
        print_progress: false,
        print_special: false,
        temperature: 0.0,
        max_segment_length: Some(30000), // 30秒最大段长度
        initial_prompt: Some("这是一段高质量的中文语音录音".to_string()),
    };
    
    // 快速配置（适合实时性要求高的场景）
    let fast_config = WhisperConfig {
        model_path: PathBuf::from("models/ggml-tiny.bin"),
        language: Some("zh".to_string()),
        translate: false,
        n_threads: 4,
        print_timestamps: false,
        print_progress: false,
        print_special: false,
        temperature: 0.2,
        max_segment_length: Some(10000), // 10秒最大段长度
        initial_prompt: None,
    };
    
    let transcriber = WhisperTranscriber::new(high_quality_config)?;
    let result = transcriber.transcribe_file("audio/speech.wav").await?;
    
    println!("高质量转录: {}", result.text);
    
    Ok(())
}
```

### 批量处理

```rust
use stt::{WhisperTranscriber, WhisperConfig};
use std::path::{Path, PathBuf};
use tokio::fs;

#[tokio::main]
async fn main() -> stt::SttResult<()> {
    let config = WhisperConfig::new("models/ggml-base.bin")
        .with_language("zh")
        .with_threads(6);
    
    let transcriber = WhisperTranscriber::new(config)?;
    
    // 批量处理目录中的音频文件
    let audio_dir = Path::new("audio_files");
    let mut entries = fs::read_dir(audio_dir).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if matches!(ext.to_str(), Some("wav") | Some("mp3") | Some("flac")) {
                println!("处理文件: {}", path.display());
                
                match transcriber.transcribe_file(&path).await {
                    Ok(result) => {
                        println!("  转录结果: {}", result.text);
                        println!("  RTF: {:.3}", result.real_time_factor());
                        
                        // 保存结果到文本文件
                        let txt_path = path.with_extension("txt");
                        fs::write(&txt_path, &result.text).await?;
                        println!("  已保存到: {}", txt_path.display());
                    }
                    Err(e) => {
                        eprintln!("  转录失败: {}", e);
                    }
                }
            }
        }
    }
    
    Ok(())
}
```

### 结果过滤

```rust
use stt::transcribe_file;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> stt::SttResult<()> {
    let result = transcribe_file("models/ggml-base.bin", "audio/speech.wav").await?;
    
    // 过滤低置信度的结果
    let high_confidence_result = result.filter_by_confidence(0.7);
    println!("高置信度文本: {}", high_confidence_result.text);
    
    // 只保留长度超过特定阈值的段
    let long_segments: Vec<_> = result.segments
        .iter()
        .filter(|seg| seg.text.trim().len() > 10)
        .collect();
    
    println!("长段落数量: {}", long_segments.len());
    
    // 按置信度排序
    let mut sorted_segments = result.segments.clone();
    sorted_segments.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    
    println!("最高置信度段落: {}", sorted_segments[0].text);
    
    Ok(())
}
```

## 性能优化

### 模型选择建议

| 模型 | 大小 | 速度 | 准确性 | 适用场景 |
|------|------|------|--------|----------|
| ggml-tiny.bin | ~39MB | 最快 | 较低 | 实时转录、快速原型 |
| ggml-base.bin | ~142MB | 快 | 中等 | 一般应用、平衡性能 |
| ggml-small.bin | ~466MB | 中等 | 较高 | 高质量转录 |
| ggml-medium.bin | ~1.5GB | 慢 | 高 | 专业转录 |
| ggml-large.bin | ~2.9GB | 最慢 | 最高 | 最高质量要求 |

### 性能调优参数

```rust
use stt::{WhisperConfig, WhisperTranscriber};

// 实时性优先配置
let realtime_config = WhisperConfig::new("models/ggml-tiny.bin")
    .with_language("zh")    // 指定语言避免检测开销
    .with_threads(4)        // 根据CPU核心数调整
    .with_temperature(0.0); // 降低随机性

// 准确性优先配置
let accuracy_config = WhisperConfig::new("models/ggml-large.bin")
    .with_language("zh")
    .with_threads(8)        // 更多线程
    .with_temperature(0.0)
    .with_initial_prompt("这是一段清晰的中文语音");

// 内存优化配置
let memory_config = WhisperConfig::new("models/ggml-base.bin")
    .with_threads(2)        // 减少线程数
    .with_language("zh");   // 避免多语言模型开销
```

### 性能监控

```rust
use stt::transcribe_file;
use std::time::Instant;

#[tokio::main]
async fn main() -> stt::SttResult<()> {
    let start = Instant::now();
    let result = transcribe_file("models/ggml-base.bin", "audio/speech.wav").await?;
    let total_time = start.elapsed();
    
    // 性能指标
    println!("=== 性能报告 ===");
    println!("音频时长: {:.2}秒", result.audio_duration as f64 / 1000.0);
    println!("处理时间: {:.2}秒", total_time.as_secs_f64());
    println!("实时因子: {:.3}", result.real_time_factor());
    println!("平均置信度: {:.3}", result.average_confidence());
    
    // RTF < 1.0 表示可以实时处理
    if result.real_time_factor() < 1.0 {
        println!("✅ 可以实时处理");
    } else {
        println!("⚠️  无法实时处理，考虑使用更小的模型或优化配置");
    }
    
    Ok(())
}
```

## 常见错误排查

### 1. 模型文件问题

```rust
use stt::{WhisperConfig, SttError};

fn check_model_file(model_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = WhisperConfig::new(model_path);
    
    match config.validate() {
        Ok(_) => println!("✅ 模型文件有效"),
        Err(SttError::ModelLoadError(msg)) => {
            eprintln!("❌ 模型加载错误: {}", msg);
            eprintln!("解决方案:");
            eprintln!("  1. 检查文件路径是否正确");
            eprintln!("  2. 确认文件存在且可读");
            eprintln!("  3. 验证模型文件格式是否正确");
        }
        Err(e) => eprintln!("❌ 其他错误: {}", e),
    }
    
    Ok(())
}
```

### 2. 音频格式问题

```rust
use stt::transcribe_file;

async fn safe_transcribe(model_path: &str, audio_path: &str) {
    match transcribe_file(model_path, audio_path).await {
        Ok(result) => {
            println!("✅ 转录成功: {}", result.text);
        }
        Err(e) => {
            eprintln!("❌ 转录失败: {}", e);
            eprintln!("常见解决方案:");
            eprintln!("  1. 确认音频文件格式支持（WAV, MP3, FLAC, M4A）");
            eprintln!("  2. 检查音频文件是否损坏");
            eprintln!("  3. 尝试转换为 16kHz 单声道 WAV 格式");
            eprintln!("  4. 确认 FFmpeg 已正确安装");
        }
    }
}
```

### 3. 内存不足问题

```rust
// 对于大文件，考虑分段处理
use stt::{WhisperTranscriber, WhisperConfig};
use audio_utils::AudioProcessor;

async fn process_large_file(model_path: &str, audio_path: &str) -> stt::SttResult<String> {
    let config = WhisperConfig::new(model_path)
        .with_language("zh")
        .with_threads(4); // 减少线程数以节省内存
    
    let transcriber = WhisperTranscriber::new(config)?;
    
    // 检查文件大小
    let metadata = std::fs::metadata(audio_path)?;
    if metadata.len() > 100 * 1024 * 1024 { // 100MB
        println!("⚠️  大文件检测，建议分段处理");
        // 这里可以实现分段处理逻辑
    }
    
    let result = transcriber.transcribe_file(audio_path).await?;
    Ok(result.text)
}
```

## 示例代码

完整的示例代码可以在以下位置找到：

- `stt/examples/transcribe_file.rs` - 基础文件转录示例
- `stt/examples/bench_transcribe.rs` - 性能基准测试
- `stt/examples/streaming_transcribe.rs` - 流式转录示例

运行示例：

```bash
# 基础文件转录
cargo run -p stt --example transcribe_file -- models/ggml-base.bin audio/speech.wav

# 性能基准测试
cargo run -p stt --example bench_transcribe -- models/ggml-base.bin audio/speech.wav 5

# 流式转录（如果启用了 streaming 功能）
cargo run -p stt --example streaming_transcribe --features streaming -- models/ggml-base.bin
```

## 最佳实践

1. **模型选择**: 根据应用场景选择合适的模型大小
2. **语言指定**: 明确指定语言可以提高准确性和速度
3. **线程配置**: 根据CPU核心数合理设置线程数
4. **音频预处理**: 使用16kHz单声道格式可以获得最佳性能
5. **错误处理**: 始终处理可能的错误情况
6. **性能监控**: 监控RTF指标确保满足实时性要求
7. **内存管理**: 对于大文件考虑分段处理

通过遵循这些指南，你可以有效地使用 `stt` 模块进行各种语音转文本任务。