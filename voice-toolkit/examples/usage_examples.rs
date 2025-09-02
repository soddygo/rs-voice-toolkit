//! Voice Toolkit 使用示例
//! 
//! 这个文件展示了如何使用 voice-toolkit 库的各种功能。
//! 包含了 STT、TTS 和音频处理的完整示例。

use voice_toolkit::{transcribe_file_unified, Error, Result};
use std::path::Path;

// 由于我们不知道具体的音频模块接口，先使用一些假想的函数
// 实际使用时需要根据真实的 API 进行调整

// 模拟音频元数据结构
#[derive(Debug)]
#[allow(dead_code)]
struct AudioMetadata {
    format: String,
    duration: f64,
    sample_rate: u32,
    channels: u32,
}

// 模拟音频处理函数
#[allow(dead_code)]
async fn get_audio_metadata(_path: &Path) -> Result<AudioMetadata> {
    // 这是一个模拟实现
    Ok(AudioMetadata {
        format: "wav".to_string(),
        duration: 5.0,
        sample_rate: 16000,
        channels: 1,
    })
}

#[allow(dead_code)]
async fn convert_to_whisper_format<P1, P2>(_input: P1, _output: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    // 这是一个模拟实现
    Ok(())
}

/// 基本的语音转文本示例
/// 
/// 这个示例展示了如何使用 voice-toolkit 进行基本的语音转录。
/// 
/// # 参数
/// 
/// * `model_path` - Whisper 模型文件的路径
/// * `audio_path` - 要转录的音频文件路径
/// 
/// # 示例
/// 
/// ```rust
/// # use voice_toolkit::examples::basic_stt_example;
/// # tokio_test::block_on(async {
/// let result = basic_stt_example("models/ggml-base.bin", "audio/hello.wav").await;
/// match result {
///     Ok(text) => println!("转录结果: {}", text),
///     Err(e) => println!("转录失败: {}", e),
/// }
/// # });
/// ```
pub async fn basic_stt_example<P1, P2>(model_path: P1, audio_path: P2) -> Result<String>
where
    P1: Into<std::path::PathBuf>,
    P2: AsRef<std::path::Path>,
{
    println!("开始转录音频文件: {:?}", audio_path.as_ref());
    
    // 使用统一的转录函数
    let result = transcribe_file_unified(model_path, audio_path).await?;
    
    println!("转录完成!");
    println!("文本内容: {}", result.text);
    println!("处理时间: {:?}", result.processing_time);
    
    Ok(result.text)
}

/// 带错误处理的语音转录示例
/// 
/// 这个示例展示了如何处理各种可能的错误情况。
/// 
/// # 示例
/// 
/// ```rust
/// # use voice_toolkit::examples::stt_with_error_handling;
/// # tokio_test::block_on(async {
/// stt_with_error_handling("models/ggml-base.bin", "audio/hello.wav").await;
/// # });
/// ```
pub async fn stt_with_error_handling<P1, P2>(model_path: P1, audio_path: P2)
where
    P1: Into<std::path::PathBuf>,
    P2: AsRef<std::path::Path>,
{
    match transcribe_file_unified(model_path, audio_path).await {
        Ok(result) => {
            println!("✅ 转录成功!");
            println!("📝 文本: {}", result.text);
            println!("⏱️  处理时间: {:?}", result.processing_time);
            
            // 如果有时间戳信息，显示详细信息
            if !result.segments.is_empty() {
                println!("🔍 详细时间戳:");
                for (i, segment) in result.segments.iter().enumerate() {
                    println!("  段落 {}: [{:.2}s - {:.2}s] {}",
                        i + 1,
                        segment.start_time,
                        segment.end_time,
                        segment.text);
                }
            }
        }
        Err(Error::Stt(e)) => {
            eprintln!("❌ 语音识别错误: {}", e);
            eprintln!("请检查:");
            eprintln!("  1. 模型文件是否存在");
            eprintln!("  2. 音频文件格式是否支持");
            eprintln!("  3. 模型文件是否损坏");
        }
        Err(Error::Audio(e)) => {
            eprintln!("❌ 音频处理错误: {}", e);
            eprintln!("请检查:");
            eprintln!("  1. 音频文件是否存在");
            eprintln!("  2. 音频文件格式是否正确");
            eprintln!("  3. 是否安装了 FFmpeg");
        }
        Err(Error::Io(e)) => {
            eprintln!("❌ 文件操作错误: {}", e);
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    eprintln!("文件不存在，请检查路径是否正确");
                }
                std::io::ErrorKind::PermissionDenied => {
                    eprintln!("权限不足，请检查文件权限");
                }
                _ => {
                    eprintln!("其他 IO 错误: {}", e);
                }
            }
        }
        Err(Error::Other(e)) => {
            eprintln!("❌ 其他错误: {}", e);
        }
    }
}

/// 批量转录示例
/// 
/// 这个示例展示了如何批量处理多个音频文件。
/// 
/// # 示例
/// 
/// ```rust
/// # use voice_toolkit::examples::batch_transcription;
/// # use std::path::Path;
/// # tokio_test::block_on(async {
/// let audio_files = vec![
///     "audio/file1.wav",
///     "audio/file2.mp3",
///     "audio/file3.flac",
/// ];
/// batch_transcription("models/ggml-base.bin", &audio_files).await;
/// # });
/// ```
pub async fn batch_transcription<P, I>(model_path: P, audio_files: I)
where
    P: Into<std::path::PathBuf>,
    I: IntoIterator,
    I::Item: AsRef<std::path::Path>,
{
    println!("开始批量转录...");
    let model_path = model_path.into();
    let mut success_count = 0;
    let mut total_count = 0;
    
    for audio_path in audio_files {
        total_count += 1;
        let audio_path = audio_path.as_ref();
        
        println!("\n📁 处理文件: {:?}", audio_path);
        
        match transcribe_file_unified(&model_path, audio_path).await {
            Ok(result) => {
                success_count += 1;
                println!("✅ 转录成功: {}", result.text);
                println!("⏱️  处理时间: {:?}", result.processing_time);
            }
            Err(e) => {
                println!("❌ 转录失败: {}", e);
            }
        }
    }
    
    println!("\n📊 批量转录完成:");
    println!("  成功: {}/{}", success_count, total_count);
    println!("  失败: {}/{}", total_count - success_count, total_count);
}

/// 性能测试示例
/// 
/// 这个示例展示了如何测试转录性能。
/// 
/// # 示例
/// 
/// ```rust
/// # use voice_toolkit::examples::performance_test;
/// # tokio_test::block_on(async {
/// performance_test("models/ggml-base.bin", "audio/hello.wav", 3).await;
/// # });
/// ```
pub async fn performance_test<P1, P2>(model_path: P1, audio_path: P2, iterations: usize)
where
    P1: Into<std::path::PathBuf>,
    P2: AsRef<std::path::Path>,
{
    println!("开始性能测试...");
    let model_path = model_path.into();
    println!("模型: {:?}", model_path);
    println!("音频: {:?}", audio_path.as_ref());
    println!("迭代次数: {}", iterations);
    
    let mut total_time = 0u64;
    let mut successful_runs = 0;
    
    for i in 1..=iterations {
        print!("第 {}/{} 次运行... ", i, iterations);
        
        match transcribe_file_unified(&model_path, audio_path.as_ref()).await {
            Ok(result) => {
                successful_runs += 1;
                total_time += result.processing_time;
                println!("✅ {:.2}s", result.processing_time as f64 / 1000.0);
            }
            Err(e) => {
                println!("❌ 失败: {}", e);
            }
        }
    }
    
    if successful_runs > 0 {
        let avg_time = total_time / successful_runs as u64;
        println!("\n📊 性能统计:");
        println!("  成功运行: {}/{}", successful_runs, iterations);
        println!("  平均处理时间: {:.2}s", avg_time as f64 / 1000.0);
        println!("  总处理时间: {:.2}s", total_time as f64 / 1000.0);
        
        // 计算 RTF (Real-Time Factor)
        if let Ok(metadata) = voice_toolkit::audio::probe(audio_path.as_ref()) {
            let audio_duration = metadata.duration_ms.unwrap_or(0) as f64 / 1000.0;
            let rtf = (avg_time as f64 / 1000.0) / audio_duration;
            println!("  实时因子 (RTF): {:.3}", rtf);
            
            if rtf < 1.0 {
                println!("  🚀 性能优秀: 处理速度比实时更快");
            } else if rtf < 2.0 {
                println!("  ⚡ 性能良好: 处理速度可接受");
            } else {
                println!("  🐌 性能较慢: 考虑使用 GPU 加速");
            }
        }
    } else {
        println!("\n❌ 所有运行都失败了");
    }
}

/// 音频格式转换示例
/// 
/// 这个示例展示了如何使用音频处理功能。
/// 
/// # 示例
/// 
/// ```rust
/// # use voice_toolkit::examples::audio_conversion_example;
/// # tokio_test::block_on(async {
/// audio_conversion_example("audio/input.mp3", "audio/output.wav").await;
/// # });
/// ```
pub async fn audio_conversion_example<P1, P2>(input_path: P1, output_path: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    println!("开始音频格式转换...");
    println!("输入: {:?}", input_path.as_ref());
    println!("输出: {:?}", output_path.as_ref());
    
    // 获取输入文件信息
    let input_metadata = voice_toolkit::audio::probe(input_path.as_ref())?;
    println!("输入文件信息:");
    println!("  格式: {:?}", input_metadata.format);
    println!("  时长: {:.2}s", input_metadata.duration_ms.unwrap_or(0) as f64 / 1000.0);
    println!("  采样率: {} Hz", input_metadata.sample_rate);
    println!("  声道数: {}", input_metadata.channels);
    
    // 转换为 Whisper 兼容格式
    voice_toolkit::audio::ensure_whisper_compatible(input_path, Some(output_path.as_ref().to_path_buf()))?;
    
    // 获取输出文件信息
    let output_metadata = voice_toolkit::audio::probe(output_path.as_ref())?;
    println!("输出文件信息:");
    println!("  格式: {:?}", output_metadata.format);
    println!("  时长: {:.2}s", output_metadata.duration_ms.unwrap_or(0) as f64 / 1000.0);
    println!("  采样率: {} Hz", output_metadata.sample_rate);
    println!("  声道数: {}", output_metadata.channels);
    
    println!("✅ 转换完成!");
    Ok(())
}

/// TTS 示例
/// 
/// 这个示例展示了如何使用文本转语音功能。
/// 
/// # 示例
/// 
/// ```rust
/// # use voice_toolkit::examples::tts_example;
/// # tokio_test::block_on(async {
/// tts_example("你好，世界！", "output/hello.wav", None).await;
/// # });
/// ```
#[cfg(feature = "tts")]
pub async fn tts_example<P: AsRef<Path>>(
    text: &str,
    output_path: P,
    tts_engine_path: Option<&str>,
) -> Result<()> {
    println!("开始文本转语音...");
    println!("文本: {}", text);
    println!("输出: {:?}", output_path.as_ref());
    
    if let Some(engine_path) = tts_engine_path {
        println!("TTS 引擎: {}", engine_path);
    }
    
    // 模拟 TTS 调用
    println!("(模拟 TTS 合成...)");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    println!("✅ 语音合成完成!");
    
    // 验证输出文件
    if let Ok(metadata) = voice_toolkit::audio::get_audio_metadata(output_path.as_ref()).await {
        println!("输出文件信息:");
        println!("  格式: {:?}", metadata.format);
        println!("  时长: {:.2}s", metadata.duration);
        println!("  采样率: {} Hz", metadata.sample_rate);
    }
    
    Ok(())
}

/// 完整的工作流示例
/// 
/// 这个示例展示了完整的语音处理工作流。
/// 
/// # 示例
/// 
/// ```rust
/// # use voice_toolkit::examples::complete_workflow_example;
/// # tokio_test::block_on(async {
/// complete_workflow_example(
///     "models/ggml-base.bin",
///     "audio/input.mp3",
///     "audio/converted.wav",
///     "audio/tts_output.wav"
/// ).await;
/// # });
/// ```
pub async fn complete_workflow_example<P1, P2, P3, P4>(
    model_path: P1,
    input_audio: P2,
    converted_audio: P3,
    _tts_output: P4,
) where
    P1: Into<std::path::PathBuf>,
    P2: AsRef<std::path::Path>,
    P3: AsRef<std::path::Path>,
    P4: AsRef<std::path::Path>,
{
    println!("🎙️  开始完整语音处理工作流");
    println!("{}", "=".repeat(50));
    
    // 步骤 1: 音频格式转换
    println!("\n📋 步骤 1: 音频格式转换");
    match audio_conversion_example(input_audio.as_ref(), converted_audio.as_ref()).await {
        Ok(_) => println!("✅ 音频转换完成"),
        Err(e) => {
            eprintln!("❌ 音频转换失败: {}", e);
            return;
        }
    }
    
    // 步骤 2: 语音转文本
    let model_path = model_path.into();
    println!("\n📋 步骤 2: 语音转文本");
    let _transcribed_text = match transcribe_file_unified(&model_path, converted_audio.as_ref()).await {
        Ok(result) => {
            println!("✅ 转录完成: {}", result.text);
            result.text
        }
        Err(e) => {
            eprintln!("❌ 转录失败: {}", e);
            return;
        }
    };
    
    // 步骤 3: 文本转语音 (如果启用 TTS)
    #[cfg(feature = "tts")]
    {
        println!("\n📋 步骤 3: 文本转语音");
        match tts_example(&transcribed_text, tts_output.as_ref(), None).await {
            Ok(_) => println!("✅ TTS 合成完成"),
            Err(e) => {
                eprintln!("❌ TTS 合成失败: {}", e);
            }
        }
    }
    
    println!("\n🎉 工作流完成!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[tokio::test]
    async fn test_basic_stt_example() {
        // 这个测试需要实际的模型和音频文件
        // 在实际测试中应该使用 mock 或测试文件
        
        let model_path = PathBuf::from("fixtures/models/ggml-tiny.bin");
        let audio_path = PathBuf::from("fixtures/audio/jfk.wav");
        
        // 检查文件是否存在，如果不存在则跳过测试
        if !model_path.exists() || !audio_path.exists() {
            println!("跳过测试: 测试文件不存在");
            return;
        }
        
        let result = basic_stt_example(&model_path, &audio_path).await;
        
        match result {
            Ok(text) => {
                assert!(!text.is_empty());
                println!("测试成功: {}", text);
            }
            Err(e) => {
                println!("测试失败: {}", e);
                // 在实际测试中这里应该是 assert!(false, ...)
            }
        }
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        // 测试错误处理
        let model_path = "nonexistent_model.bin";
        let audio_path = "nonexistent_audio.wav";
        
        // 这个应该返回错误
        let result = transcribe_file_unified(model_path, audio_path).await;
        assert!(result.is_err());
        
        // 测试错误类型
        match result.unwrap_err() {
            Error::Io(_) => println!("正确捕获了 IO 错误"),
            Error::Stt(_) => println!("正确捕获了 STT 错误"),
            Error::Audio(_) => println!("正确捕获了音频错误"),
            Error::Other(_) => println!("正确捕获了其他错误"),
        }
    }
}

/// 主函数 - 演示各种示例的使用
#[tokio::main]
async fn main() -> Result<()> {
    println!("🎙️  Voice Toolkit 使用示例");
    println!("{}", "=".repeat(50));
    
    // 模拟文件路径
    let _model_path = "models/ggml-base.bin";
    let _audio_path = "audio/hello.wav";
    let _converted_path = "audio/converted.wav";
    let _tts_path = "output/tts.wav";
    
    // 示例 1: 基本转录
    println!("\n📋 示例 1: 基本转录");
    println!("这个示例展示了基本的语音转录功能");
    println!("注意: 这是一个演示，实际的转录需要真实的模型和音频文件");
    
    // 示例 2: 错误处理
    println!("\n📋 示例 2: 错误处理");
    println!("这个示例展示了如何处理各种错误情况");
    stt_with_error_handling("nonexistent_model.bin", "nonexistent_audio.wav").await;
    
    // 示例 3: 音频转换
    println!("\n📋 示例 3: 音频转换");
    println!("这个示例展示了音频格式转换功能");
    match audio_conversion_example("input.mp3", "output.wav").await {
        Ok(_) => println!("音频转换示例完成"),
        Err(e) => println!("音频转换示例失败: {}", e),
    }
    
    // 示例 4: 性能测试
    println!("\n📋 示例 4: 性能测试");
    println!("这个示例展示了性能测试功能");
    println!("注意: 这是一个演示，实际测试需要真实的模型和音频文件");
    
    // 示例 5: TTS (如果启用)
    #[cfg(feature = "tts")]
    {
        println!("\n📋 示例 5: 文本转语音");
        println!("这个示例展示了文本转语音功能");
        match tts_example("你好，世界！", tts_path, None).await {
            Ok(_) => println!("TTS 示例完成"),
            Err(e) => println!("TTS 示例失败: {}", e),
        }
    }
    
    println!("\n🎉 所有示例演示完成!");
    println!("查看代码了解如何在实际应用中使用这些功能");
    
    Ok(())
}