//! Voice Toolkit 流式转录示例
//! 
//! 这个文件展示了如何使用 voice-toolkit 进行实时音频流转录。
//! 
//! # 使用说明
//! 
//! 运行这个示例需要：
//! 1. 启用 streaming 特性: `cargo run --example streaming_example --features streaming`
//! 2. 提供 Whisper 模型路径和音频文件路径

use voice_toolkit::Result;

/// 流式转录示例
/// 
/// 这个示例展示了如何使用 StreamingTranscriber 进行实时音频流转录。
/// 
/// # 参数
/// 
/// * `model_path` - Whisper 模型文件的路径
/// * `audio_path` - 要转录的音频文件路径
/// 
/// # 示例
/// 
/// ```rust
/// # use voice_toolkit::streaming_examples::streaming_transcription_example;
/// # tokio_test::block_on(async {
/// streaming_transcription_example("models/ggml-base.bin", "audio/hello.wav").await.unwrap();
/// # });
/// ```
#[cfg(feature = "streaming")]
pub async fn streaming_transcription_example<P1, P2>(model_path: P1, audio_path: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    use voice_toolkit::stt::streaming::StreamingTranscriber;
    use voice_toolkit::audio::get_audio_metadata;
    
    println!("🎙️  开始流式转录示例");
    println!("模型: {:?}", model_path.as_ref());
    println!("音频: {:?}", audio_path.as_ref());
    
    // 获取音频文件信息
    let metadata = get_audio_metadata(audio_path.as_ref()).await?;
    println!("音频信息: {:.2}s, {}Hz, {} 声道", 
        metadata.duration, metadata.sample_rate, metadata.channels);
    
    // 创建流式转录器
    let mut transcriber = StreamingTranscriber::new(model_path).await?;
    
    // 配置转录参数
    transcriber.set_language("auto")?; // 自动检测语言
    transcriber.set_task("transcribe")?; // 转录任务
    
    println!("🎧 开始处理音频流...");
    
    // 读取音频文件并模拟流式处理
    let audio_data = std::fs::read(audio_path.as_ref())?;
    let chunk_size = 1024 * 4; // 4KB chunks
    
    let mut position = 0;
    let mut total_segments = 0;
    let mut start_time = std::time::Instant::now();
    
    while position < audio_data.len() {
        let end_position = (position + chunk_size).min(audio_data.len());
        let chunk = &audio_data[position..end_position];
        
        // 处理音频块
        let segments = transcriber.process_audio(chunk).await?;
        
        // 显示新的转录结果
        for segment in segments {
            total_segments += 1;
            println!("📝 [{}s - {}s] {}", 
                segment.start_time, 
                segment.end_time, 
                segment.text);
        }
        
        position = end_position;
        
        // 模拟实时处理延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    // 获取最终结果
    let final_result = transcriber.finalize().await?;
    
    let processing_time = start_time.elapsed();
    
    println!("\n✅ 流式转录完成!");
    println!("📊 统计信息:");
    println!("  总段落数: {}", total_segments);
    println!("  最终文本: {}", final_result.text);
    println!("  处理时间: {:?}", processing_time);
    println!("  实时因子: {:.3}", 
        processing_time.as_secs_f64() / metadata.duration);
    
    Ok(())
}

/// 带VAD的流式转录示例
/// 
/// 这个示例展示了如何结合语音活动检测(VAD)进行更高效的流式转录。
/// 
/// # 参数
/// 
/// * `model_path` - Whisper 模型文件的路径
/// * `audio_path` - 要转录的音频文件路径
/// 
/// # 示例
/// 
/// ```rust
/// # use voice_toolkit::streaming_examples::streaming_with_vad_example;
/// # tokio_test::block_on(async {
/// streaming_with_vad_example("models/ggml-base.bin", "audio/hello.wav").await.unwrap();
/// # });
/// ```
#[cfg(feature = "streaming")]
pub async fn streaming_with_vad_example<P1, P2>(model_path: P1, audio_path: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    use voice_toolkit::stt::streaming::StreamingTranscriber;
    use voice_toolkit::audio::get_audio_metadata;
    
    println!("🎙️  开始带VAD的流式转录示例");
    println!("模型: {:?}", model_path.as_ref());
    println!("音频: {:?}", audio_path.as_ref());
    
    // 获取音频文件信息
    let metadata = get_audio_metadata(audio_path.as_ref()).await?;
    println!("音频信息: {:.2}s, {}Hz, {} 声道", 
        metadata.duration, metadata.sample_rate, metadata.channels);
    
    // 创建流式转录器并启用VAD
    let mut transcriber = StreamingTranscriber::new(model_path).await?;
    
    // 启用VAD
    transcriber.enable_vad(true)?;
    transcriber.set_vad_threshold(0.5)?; // 设置VAD阈值
    
    println!("🎧 开始处理音频流 (VAD已启用)...");
    
    // 读取音频文件并模拟流式处理
    let audio_data = std::fs::read(audio_path.as_ref())?;
    let chunk_size = 1024 * 2; // 较小的块大小，更适合VAD
    
    let mut position = 0;
    let mut voice_segments = 0;
    let mut silence_segments = 0;
    let mut total_segments = 0;
    let mut start_time = std::time::Instant::now();
    
    while position < audio_data.len() {
        let end_position = (position + chunk_size).min(audio_data.len());
        let chunk = &audio_data[position..end_position];
        
        // 处理音频块
        let segments = transcriber.process_audio(chunk).await?;
        
        // 检查VAD状态
        let is_speech = transcriber.is_speech_detected().unwrap_or(false);
        
        if is_speech {
            voice_segments += 1;
            println!("🗣️  检测到语音 - 处理 {} 个段落", segments.len());
        } else {
            silence_segments += 1;
            if !segments.is_empty() {
                println!("🤫 静音段 - 仍有 {} 个待处理段落", segments.len());
            }
        }
        
        // 显示转录结果
        for segment in segments {
            total_segments += 1;
            println!("📝 [{}s - {}s] {}", 
                segment.start_time, 
                segment.end_time, 
                segment.text);
        }
        
        position = end_position;
        
        // 模拟实时处理延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    // 获取最终结果
    let final_result = transcriber.finalize().await?;
    
    let processing_time = start_time.elapsed();
    
    println!("\n✅ 带VAD的流式转录完成!");
    println!("📊 统计信息:");
    println!("  语音段落数: {}", voice_segments);
    println!("  静音段落数: {}", silence_segments);
    println!("  总转录段落数: {}", total_segments);
    println!("  最终文本: {}", final_result.text);
    println!("  处理时间: {:?}", processing_time);
    println!("  实时因子: {:.3}", 
        processing_time.as_secs_f64() / metadata.duration);
    
    // VAD效率分析
    let total_chunks = voice_segments + silence_segments;
    let voice_ratio = voice_segments as f64 / total_chunks as f64;
    println!("  语音占比: {:.1}%", voice_ratio * 100.0);
    
    if voice_ratio < 0.3 {
        println!("  💡 VAD优化效果显著，跳过了 {:.1}% 的静音部分", 
            (1.0 - voice_ratio) * 100.0);
    }
    
    Ok(())
}

/// 实时麦克风转录示例
/// 
/// 这个示例展示了如何从麦克风进行实时转录。
/// 
/// # 注意事项
/// 
/// 这个示例需要额外的音频输入库，这里只提供框架代码。
/// 
/// # 示例
/// 
/// ```rust
/// # use voice_toolkit::streaming_examples::real_time_microphone_example;
/// # tokio_test::block_on(async {
/// // 注意: 这个示例需要实际的音频输入库支持
/// // real_time_microphone_example("models/ggml-base.bin").await.unwrap();
/// # });
/// ```
#[cfg(feature = "streaming")]
pub async fn real_time_microphone_example<P: AsRef<Path>>(model_path: P) -> Result<()> {
    use voice_toolkit::stt::streaming::StreamingTranscriber;
    
    println!("🎙️  开始实时麦克风转录示例");
    println!("模型: {:?}", model_path.as_ref());
    println!("⚠️  注意: 这个示例需要音频输入库支持");
    
    // 创建流式转录器
    let mut transcriber = StreamingTranscriber::new(model_path).await?;
    
    // 配置参数
    transcriber.set_language("auto")?;
    transcriber.set_task("transcribe")?;
    transcriber.enable_vad(true)?;
    
    println!("🎧 开始监听麦克风...");
    println!("按 Ctrl+C 停止转录");
    
    // 模拟音频输入循环
    // 在实际应用中，这里应该从麦克风读取音频数据
    let mut counter = 0;
    let start_time = std::time::Instant::now();
    
    loop {
        counter += 1;
        
        // 模拟音频数据（实际应用中应该从麦克风读取）
        let fake_audio_data = vec![0u8; 1024];
        
        // 处理音频数据
        match transcriber.process_audio(&fake_audio_data).await {
            Ok(segments) => {
                for segment in segments {
                    println!("📝 [{}s - {}s] {}", 
                        segment.start_time, 
                        segment.end_time, 
                        segment.text);
                }
            }
            Err(e) => {
                eprintln!("处理音频数据时出错: {}", e);
            }
        }
        
        // 模拟实时处理
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // 模拟停止条件
        if counter >= 50 { // 5秒后停止
            break;
        }
    }
    
    // 获取最终结果
    let final_result = transcriber.finalize().await?;
    
    println!("\n✅ 实时转录完成!");
    println!("运行时间: {:?}", start_time.elapsed());
    println!("最终文本: {}", final_result.text);
    
    Ok(())
}

/// 性能对比示例
/// 
/// 这个示例对比了流式转录和批量转录的性能差异。
/// 
/// # 参数
/// 
/// * `model_path` - Whisper 模型文件的路径
/// * `audio_path` - 要转录的音频文件路径
/// 
/// # 示例
/// 
/// ```rust
/// # use voice_toolkit::streaming_examples::performance_comparison_example;
/// # tokio_test::block_on(async {
/// performance_comparison_example("models/ggml-base.bin", "audio/hello.wav").await.unwrap();
/// # });
/// ```
#[cfg(feature = "streaming")]
pub async fn performance_comparison_example<P1, P2>(model_path: P1, audio_path: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    use voice_toolkit::stt::streaming::StreamingTranscriber;
    use voice_toolkit::audio::get_audio_metadata;
    
    println!("🔍 开始性能对比示例");
    println!("模型: {:?}", model_path.as_ref());
    println!("音频: {:?}", audio_path.as_ref());
    
    // 获取音频文件信息
    let metadata = get_audio_metadata(audio_path.as_ref()).await?;
    println!("音频信息: {:.2}s, {}Hz, {} 声道", 
        metadata.duration, metadata.sample_rate, metadata.channels);
    
    // 测试批量转录
    println!("\n📊 批量转录测试:");
    let batch_start = std::time::Instant::now();
    let batch_result = transcribe_file_unified(model_path.as_ref(), audio_path.as_ref()).await?;
    let batch_time = batch_start.elapsed();
    
    println!("  批量转录时间: {:?}", batch_time);
    println!("  批量转录结果: {}", batch_result.text);
    println!("  批量转录RTF: {:.3}", 
        batch_time.as_secs_f64() / metadata.duration);
    
    // 测试流式转录
    println!("\n📊 流式转录测试:");
    let mut transcriber = StreamingTranscriber::new(model_path.as_ref()).await?;
    
    let stream_start = std::time::Instant::now();
    
    // 读取音频文件
    let audio_data = std::fs::read(audio_path.as_ref())?;
    let chunk_size = 1024 * 4;
    
    let mut position = 0;
    let mut stream_segments = Vec::new();
    
    while position < audio_data.len() {
        let end_position = (position + chunk_size).min(audio_data.len());
        let chunk = &audio_data[position..end_position];
        
        let segments = transcriber.process_audio(chunk).await?;
        stream_segments.extend(segments);
        
        position = end_position;
        
        // 模拟实时处理延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    let stream_result = transcriber.finalize().await?;
    let stream_time = stream_start.elapsed();
    
    println!("  流式转录时间: {:?}", stream_time);
    println!("  流式转录结果: {}", stream_result.text);
    println!("  流式转录RTF: {:.3}", 
        stream_time.as_secs_f64() / metadata.duration);
    println!("  流式段落数: {}", stream_segments.len());
    
    // 性能对比分析
    println!("\n📈 性能对比分析:");
    println!("  批量转录时间: {:.2}s", batch_time.as_secs_f64());
    println!("  流式转录时间: {:.2}s", stream_time.as_secs_f64());
    println!("  时间差异: {:.2}s", 
        (stream_time.as_secs_f64() - batch_time.as_secs_f64()).abs());
    
    let time_ratio = stream_time.as_secs_f64() / batch_time.as_secs_f64();
    println!("  时间比率: {:.3}", time_ratio);
    
    if time_ratio < 1.1 {
        println!("  ✅ 流式转录性能接近批量转录");
    } else if time_ratio < 1.5 {
        println!("  ⚠️  流式转录有轻微性能损失");
    } else {
        println!("  ❌ 流式转录性能损失较大");
    }
    
    // 准确性对比
    let batch_text = batch_result.text.trim();
    let stream_text = stream_result.text.trim();
    
    println!("\n🎯 准确性对比:");
    println!("  批量转录: {}", batch_text);
    println!("  流式转录: {}", stream_text);
    
    if batch_text == stream_text {
        println!("  ✅ 转录结果完全一致");
    } else {
        println!("  ⚠️  转录结果存在差异");
        
        // 计算相似度（简单的字符匹配）
        let similarity = calculate_similarity(batch_text, stream_text);
        println!("  相似度: {:.1}%", similarity * 100.0);
    }
    
    Ok(())
}

/// 计算两个字符串的相似度
#[allow(dead_code)]
fn calculate_similarity(s1: &str, s2: &str) -> f64 {
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }
    
    // 使用编辑距离计算相似度
    let distance = levenshtein_distance(s1, s2);
    let max_len = s1.len().max(s2.len());
    
    1.0 - (distance as f64 / max_len as f64)
}

/// 计算编辑距离
#[allow(dead_code)]
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1: Vec<char> = s1.chars().collect();
    let s2: Vec<char> = s2.chars().collect();
    
    let mut matrix = vec![vec![0; s2.len() + 1]; s1.len() + 1];
    
    for i in 0..=s1.len() {
        matrix[i][0] = i;
    }
    
    for j in 0..=s2.len() {
        matrix[0][j] = j;
    }
    
    for i in 1..=s1.len() {
        for j in 1..=s2.len() {
            let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }
    
    matrix[s1.len()][s2.len()]
}

/// 主函数 - 演示流式转录示例的使用
#[tokio::main]
async fn main() -> Result<()> {
    println!("🎙️  Voice Toolkit 流式转录示例");
    println!("{}", "=".repeat(50));
    
    // 模拟文件路径
    let _model_path = "models/ggml-base.bin";
    let _audio_path = "audio/hello.wav";
    
    // 示例 1: 基本流式转录
    println!("\n📋 示例 1: 基本流式转录");
    println!("这个示例展示了基本的流式转录功能");
    println!("注意: 这是一个演示，实际使用需要启用 streaming 特性");
    
    #[cfg(feature = "streaming")]
    {
        match streaming_transcription_example(_model_path, _audio_path).await {
            Ok(_) => println!("流式转录示例完成"),
            Err(e) => println!("流式转录示例失败: {}", e),
        }
    }
    
    #[cfg(not(feature = "streaming"))]
    {
        println!("⚠️  streaming 特性未启用，无法运行流式转录示例");
        println!("请使用以下命令运行:");
        println!("cargo run --example streaming_examples --features streaming");
    }
    
    // 示例 2: 带VAD的流式转录
    println!("\n📋 示例 2: 带VAD的流式转录");
    println!("这个示例展示了如何结合语音活动检测进行流式转录");
    
    #[cfg(feature = "streaming")]
    {
        match streaming_with_vad_example(model_path, audio_path).await {
            Ok(_) => println!("带VAD的流式转录示例完成"),
            Err(e) => println!("带VAD的流式转录示例失败: {}", e),
        }
    }
    
    // 示例 3: 性能对比
    println!("\n📋 示例 3: 性能对比");
    println!("这个示例对比了流式转录和批量转录的性能");
    
    #[cfg(feature = "streaming")]
    {
        match performance_comparison_example(model_path, audio_path).await {
            Ok(_) => println!("性能对比示例完成"),
            Err(e) => println!("性能对比示例失败: {}", e),
        }
    }
    
    // 示例 4: 实时麦克风转录
    println!("\n📋 示例 4: 实时麦克风转录");
    println!("这个示例展示了如何从麦克风进行实时转录");
    println!("注意: 这个示例需要额外的音频输入库支持");
    
    #[cfg(feature = "streaming")]
    {
        println!("(这是一个框架示例，实际使用需要音频输入库)");
    }
    
    println!("\n🎉 所有流式转录示例演示完成!");
    println!("查看代码了解如何在实际应用中使用这些功能");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);
    }
    
    #[test]
    fn test_similarity() {
        assert!(calculate_similarity("hello", "hello") > 0.99);
        assert!(calculate_similarity("hello", "hell") > 0.8);
        assert!(calculate_similarity("hello", "world") < 0.5);
    }
}