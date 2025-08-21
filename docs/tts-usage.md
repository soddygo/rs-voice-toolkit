# TTS (文本转语音) 使用指南

本文档介绍如何使用 rs-voice-toolkit 的 TTS 模块进行文本转语音合成。

## 概述

TTS 模块提供文本转语音功能，目前支持 Index-TTS 引擎，未来将扩展支持更多引擎。该模块采用可扩展的架构设计，允许动态选择和切换不同的 TTS 引擎。

## Index-TTS 安装与配置

### 安装 Index-TTS

Index-TTS 是 bilibili 开源的高质量中文 TTS 引擎，提供接近商业级的语音合成效果。

#### 方法一：从源码编译

```bash
# 克隆 Index-TTS 仓库
git clone https://github.com/bilibili/Index-TTS.git
cd Index-TTS

# 安装依赖
pip install -r requirements.txt

# 下载预训练模型
# 按照官方文档下载所需的模型文件

# 运行测试
python inference.py --text "你好，世界" --output test.wav
```

#### 方法二：使用预编译版本

```bash
# 下载预编译的 index-tts 可执行文件
# 将其放置在系统 PATH 中，或记录其完整路径
```

### 验证安装

```bash
# 检查 index-tts 是否在 PATH 中
which index-tts

# 或直接测试
index-tts --help
```

## 基本用法

### 1. 创建 TTS 服务

```rust
use rs_voice_toolkit_tts::{TtsConfig, TtsService, TtsEngineType};
use std::path::PathBuf;

// 使用默认配置（自动查找 index-tts）
let config = TtsConfig::default();
let tts_service = TtsService::new(config);

// 或指定 index-tts 可执行文件路径
let config = TtsConfig {
    executable_path: Some(PathBuf::from("/path/to/index-tts")),
    language: Some("zh".to_string()),
    speaker: Some("default".to_string()),
    sample_rate: 22050,
    speed: 1.0,
    pitch: 0.0,
};
let tts_service = TtsService::new(config);

// 使用指定引擎类型创建服务
let tts_service = TtsService::new_with_engine(config, TtsEngineType::IndexTts);
```

### 2. 检查引擎可用性

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tts_service = TtsService::new(TtsConfig::default());
    
    if !tts_service.is_available().await {
        eprintln!("TTS 引擎不可用，请检查 index-tts 安装");
        return Ok(());
    }
    
    println!("TTS 引擎可用");
    Ok(())
}
```

### 3. 文本转语音

#### 生成音频数据到内存

```rust
use rs_voice_toolkit_tts::{TtsConfig, TtsService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tts_service = TtsService::new(TtsConfig::default());
    
    // 检查可用性
    if !tts_service.is_available().await {
        eprintln!("TTS 引擎不可用");
        return Ok(());
    }
    
    // 合成语音到内存
    let text = "你好，欢迎使用 rs-voice-toolkit TTS 模块";
    match tts_service.text_to_speech(text).await {
        Ok(audio_data) => {
            println!("合成成功，音频数据大小: {} 字节", audio_data.len());
            // 处理音频数据...
        }
        Err(e) => {
            eprintln!("合成失败: {}", e);
        }
    }
    
    Ok(())
}
```

#### 生成音频文件

```rust
use rs_voice_toolkit_tts::{TtsConfig, TtsService};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tts_service = TtsService::new(TtsConfig::default());
    
    let text = "这是一个测试文本，将被转换为语音文件";
    let output_path = Path::new("output.wav");
    
    match tts_service.text_to_file(text, output_path).await {
        Ok(()) => {
            println!("音频文件已保存到: {}", output_path.display());
        }
        Err(e) => {
            eprintln!("保存失败: {}", e);
        }
    }
    
    Ok(())
}
```

## 配置选项

### TtsConfig 结构体

```rust
pub struct TtsConfig {
    /// Index-TTS 可执行文件路径（为空则查找 PATH）
    pub executable_path: Option<PathBuf>,
    
    /// 语言设置
    pub language: Option<String>,
    
    /// 说话人设置
    pub speaker: Option<String>,
    
    /// 采样率（Hz）
    pub sample_rate: u32,
    
    /// 语音速度 (0.5 - 2.0)
    pub speed: f32,
    
    /// 音调调整 (-20.0 - 20.0)
    pub pitch: f32,
}
```

### 默认配置

```rust
impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            executable_path: None,           // 自动查找 PATH
            language: Some("auto".to_string()), // 自动检测语言
            speaker: None,                   // 使用默认说话人
            sample_rate: 22050,              // 22.05kHz 采样率
            speed: 1.0,                      // 正常语速
            pitch: 0.0,                      // 正常音调
        }
    }
}
```

### 配置示例

```rust
// 中文语音配置
let chinese_config = TtsConfig {
    language: Some("zh".to_string()),
    speaker: Some("female".to_string()),
    speed: 1.2,  // 稍快语速
    pitch: 2.0,  // 稍高音调
    ..Default::default()
};

// 英文语音配置
let english_config = TtsConfig {
    language: Some("en".to_string()),
    speaker: Some("male".to_string()),
    speed: 0.9,  // 稍慢语速
    pitch: -1.0, // 稍低音调
    ..Default::default()
};

// 高质量音频配置
let high_quality_config = TtsConfig {
    sample_rate: 44100,  // 高采样率
    speed: 1.0,
    pitch: 0.0,
    ..Default::default()
};
```

## 引擎类型

### TtsEngineType 枚举

```rust
pub enum TtsEngineType {
    /// Index-TTS 引擎（当前支持）
    IndexTts,
    
    /// Piper 引擎（未来支持）
    Piper,
    
    /// Coqui 引擎（未来支持）
    Coqui,
}
```

### 引擎特性对比

| 引擎 | 状态 | 语言支持 | 音质 | 性能 | 特点 |
|------|------|----------|------|------|------|
| Index-TTS | ✅ 支持 | 中文、英文 | 优秀 | 中等 | 中文自然度高 |
| Piper | 🚧 计划中 | 多语言 | 良好 | 快速 | 轻量级 |
| Coqui | 🚧 计划中 | 多语言 | 优秀 | 慢速 | 功能丰富 |

## 输出格式

### 支持的音频格式

Index-TTS 引擎支持以下输出格式：

- **WAV**: 无损音频格式（推荐）
- **MP3**: 压缩音频格式
- **FLAC**: 无损压缩格式

### 音频参数

- **采样率**: 16kHz, 22.05kHz, 44.1kHz
- **位深度**: 16-bit, 24-bit
- **声道**: 单声道（mono）
- **编码**: PCM, MP3, FLAC

### 输出示例

```rust
// 生成不同格式的音频文件
let tts_service = TtsService::new(TtsConfig::default());
let text = "测试文本";

// WAV 格式（推荐）
tts_service.text_to_file(text, Path::new("output.wav")).await?;

// 高质量 WAV
let high_quality_config = TtsConfig {
    sample_rate: 44100,
    ..Default::default()
};
let hq_service = TtsService::new(high_quality_config);
hq_service.text_to_file(text, Path::new("output_hq.wav")).await?;
```

## 完整示例

### 命令行 TTS 工具

```rust
use rs_voice_toolkit_tts::{TtsConfig, TtsService};
use std::path::PathBuf;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("用法: {} <文本> <输出文件> [index-tts路径]", args[0]);
        std::process::exit(1);
    }
    
    let text = &args[1];
    let output_path = PathBuf::from(&args[2]);
    let executable_path = args.get(3).map(PathBuf::from);
    
    // 创建配置
    let config = TtsConfig {
        executable_path,
        language: Some("auto".to_string()),
        speed: 1.0,
        pitch: 0.0,
        ..Default::default()
    };
    
    // 创建 TTS 服务
    let tts_service = TtsService::new(config);
    
    // 检查可用性
    if !tts_service.is_available().await {
        eprintln!("错误: index-tts 不可用");
        eprintln!("请安装 index-tts 并确保在 PATH 中，或提供可执行文件路径");
        std::process::exit(2);
    }
    
    println!("正在合成语音: {}", text);
    
    // 执行合成
    match tts_service.text_to_file(text, &output_path).await {
        Ok(()) => {
            println!("✅ 合成成功！");
            println!("📁 输出文件: {}", output_path.display());
        }
        Err(e) => {
            eprintln!("❌ 合成失败: {}", e);
            std::process::exit(3);
        }
    }
    
    Ok(())
}
```

### 批量文本合成

```rust
use rs_voice_toolkit_tts::{TtsConfig, TtsService};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tts_service = TtsService::new(TtsConfig::default());
    
    if !tts_service.is_available().await {
        eprintln!("TTS 引擎不可用");
        return Ok(());
    }
    
    let texts = vec![
        "欢迎使用语音合成系统",
        "这是第二段测试文本",
        "感谢您的使用",
    ];
    
    for (i, text) in texts.iter().enumerate() {
        let output_path = format!("output_{}.wav", i + 1);
        
        println!("正在合成第 {} 段: {}", i + 1, text);
        
        match tts_service.text_to_file(text, Path::new(&output_path)).await {
            Ok(()) => println!("✅ 完成: {}", output_path),
            Err(e) => eprintln!("❌ 失败: {}", e),
        }
    }
    
    println!("批量合成完成！");
    Ok(())
}
```

## 故障排除

### 常见问题

#### 1. "index-tts 不可用" 错误

**原因**: 
- index-tts 未安装
- index-tts 不在系统 PATH 中
- 可执行文件路径错误

**解决方案**:
```rust
// 方法1: 检查 PATH
which index-tts

// 方法2: 指定完整路径
let config = TtsConfig {
    executable_path: Some(PathBuf::from("/full/path/to/index-tts")),
    ..Default::default()
};

// 方法3: 验证可用性
if !tts_service.is_available().await {
    eprintln!("请安装 index-tts 或检查路径配置");
}
```

#### 2. 合成质量问题

**调整语音参数**:
```rust
let config = TtsConfig {
    speed: 0.9,      // 降低语速提高清晰度
    pitch: 1.0,      // 调整音调
    sample_rate: 44100, // 提高采样率
    ..Default::default()
};
```

#### 3. 性能问题

**优化配置**:
```rust
let config = TtsConfig {
    sample_rate: 16000, // 降低采样率提高速度
    speed: 1.2,         // 适当提高语速
    ..Default::default()
};
```

#### 4. 内存使用过高

**建议**:
- 分批处理长文本
- 及时释放音频数据
- 监控系统资源使用

```rust
// 分批处理长文本
let long_text = "很长的文本内容...";
let chunks: Vec<&str> = long_text
    .split('。')
    .filter(|s| !s.is_empty())
    .collect();

for (i, chunk) in chunks.iter().enumerate() {
    let output = format!("chunk_{}.wav", i);
    tts_service.text_to_file(chunk, Path::new(&output)).await?;
}
```

## 性能建议

### 1. 选择合适的配置

- **实时应用**: 使用较低采样率 (16kHz)
- **高质量录制**: 使用较高采样率 (44.1kHz)
- **批量处理**: 考虑并发处理

### 2. 资源管理

```rust
// 避免长时间持有大量音频数据
let audio_data = tts_service.text_to_speech(text).await?;
// 立即处理或保存
save_audio_data(&audio_data)?;
// 数据会在作用域结束时自动释放
```

### 3. 错误处理

```rust
// 实现重试机制
use tokio::time::{sleep, Duration};

async fn synthesize_with_retry(
    service: &TtsService,
    text: &str,
    max_retries: u32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    for attempt in 0..max_retries {
        match service.text_to_speech(text).await {
            Ok(data) => return Ok(data),
            Err(e) if attempt < max_retries - 1 => {
                eprintln!("尝试 {} 失败: {}, 重试中...", attempt + 1, e);
                sleep(Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
    unreachable!()
}
```

## 注意事项

### 1. 依赖要求

- **Index-TTS**: 需要单独安装
- **Python 环境**: Index-TTS 需要 Python 运行时
- **系统资源**: 合成过程需要一定的 CPU 和内存

### 2. 线程安全

`TtsService` 是线程安全的，可以在多线程环境中使用：

```rust
use std::sync::Arc;
use tokio::task;

let service = Arc::new(TtsService::new(TtsConfig::default()));
let mut handles = vec![];

for i in 0..5 {
    let service_clone = Arc::clone(&service);
    let handle = task::spawn(async move {
        let text = format!("这是第 {} 个任务", i);
        let output = format!("output_{}.wav", i);
        service_clone.text_to_file(&text, Path::new(&output)).await
    });
    handles.push(handle);
}

// 等待所有任务完成
for handle in handles {
    handle.await??;
}
```

### 3. 资源清理

TTS 服务会自动管理资源，但建议：

- 及时处理生成的音频数据
- 避免同时进行过多合成任务
- 监控系统资源使用情况

### 4. 许可证考虑

使用 Index-TTS 时请遵守其开源许可证要求，确保合规使用。

## 未来规划

### 即将支持的功能

1. **多引擎支持**: Piper TTS, Coqui TTS
2. **流式合成**: 实时文本到语音转换
3. **语音克隆**: 自定义说话人声音
4. **情感控制**: 调整语音情感色彩
5. **SSML 支持**: 语音合成标记语言

### 性能优化

1. **并行处理**: 多线程合成优化
2. **缓存机制**: 常用文本缓存
3. **模型优化**: 更快的推理速度
4. **内存优化**: 降低内存占用

通过本文档，您应该能够成功集成和使用 rs-voice-toolkit 的 TTS 功能。如有问题，请参考故障排除部分或查看项目文档。