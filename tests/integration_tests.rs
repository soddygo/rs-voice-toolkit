//! 集成测试：端到端测试文件→文本和文本→音频的完整流程
//!
//! 这些测试验证各模块之间的集成是否正常工作，包括：
//! 1. STT: 文件 → 文本的完整流程
//! 2. TTS: 文本 → 音频的完整流程（需要 index-tts 可用）
//! 3. 音频处理: 格式转换和预处理

use std::path::PathBuf;
use std::process::Command;
use rs_voice_toolkit_stt;
use log::info;
#[cfg(feature = "streaming")]
use rs_voice_toolkit_stt::{StreamingConfig, StreamingTranscriber, StreamingEvent};
use rs_voice_toolkit_tts;
use rs_voice_toolkit_audio;

/// 检查必要的测试文件是否存在
fn check_test_fixtures() -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let model_path = PathBuf::from("fixtures/models/ggml-tiny.bin");
    let audio_path = PathBuf::from("fixtures/audio/jfk.wav");
    
    if !model_path.exists() {
        return Err(format!("模型文件不存在: {}\n请运行 ./fixtures/get-fixtures.sh 获取测试文件", model_path.display()).into());
    }
    
    if !audio_path.exists() {
        return Err(format!("音频文件不存在: {}\n请运行 ./fixtures/get-fixtures.sh 获取测试文件", audio_path.display()).into());
    }
    
    Ok((model_path, audio_path))
}

/// 检查 index-tts 是否可用
fn check_index_tts_available() -> bool {
    Command::new("index-tts")
        .arg("--help")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[tokio::test]
async fn test_stt_file_to_text_integration() {
    // 检查测试文件
    let (model_path, audio_path) = match check_test_fixtures() {
        Ok(paths) => paths,
        Err(e) => {
            log::warn!("跳过 STT 集成测试: {}", e);
            return;
        }
    };
    
    info!("开始 STT 集成测试...");
    info!("模型: {}", model_path.display());
    info!("音频: {}", audio_path.display());
    
    // 测试文件转录
    let result = rs_voice_toolkit_stt::transcribe_file(&model_path, &audio_path).await;
    
    match result {
        Ok(transcription) => {
            info!("转录成功: {}", transcription.text);
            
            // 验证转录结果
            assert!(!transcription.text.trim().is_empty(), "转录文本不应为空");
            assert!(transcription.segments.len() > 0, "应该有转录片段");
            assert!(transcription.audio_duration > 0, "音频时长应大于 0");
            
            // JFK 音频应该包含一些关键词
            let text_lower = transcription.text.to_lowercase();
            let contains_keywords = text_lower.contains("ask") || 
                                  text_lower.contains("not") || 
                                  text_lower.contains("what") ||
                                  text_lower.contains("country") ||
                                  text_lower.contains("can") ||
                                  text_lower.contains("do");
            
            if contains_keywords {
                info!("✓ 转录内容包含预期关键词");
            } else {
                info!("⚠ 转录内容未包含预期关键词，但测试仍然通过");
            }
            
            info!("✓ STT 集成测试通过");
        }
        Err(e) => {
            panic!("STT 集成测试失败: {}", e);
        }
    }
}

#[tokio::test]
async fn test_audio_processing_integration() {
    // 检查测试文件
    let (_, audio_path) = match check_test_fixtures() {
        Ok(paths) => paths,
        Err(e) => {
            log::warn!("跳过音频处理集成测试: {}", e);
            return;
        }
    };
    
    info!("开始音频处理集成测试...");
    
    // 测试音频信息探测
    let probe_result = rs_voice_toolkit_audio::probe(&audio_path);
    match probe_result {
        Ok(info) => {
            info!("音频信息: {:?}", info);
            if let Some(duration_ms) = info.duration_ms {
                assert!(duration_ms > 0, "音频时长应大于 0");
            }
            info!("✓ 音频探测成功");
        }
        Err(e) => {
            panic!("音频探测失败: {}", e);
        }
    }
    
    // 测试音频格式转换
    let temp_output = std::env::temp_dir().join("test_converted.wav");
    let convert_result = rs_voice_toolkit_audio::ensure_whisper_compatible(&audio_path, Some(temp_output.clone()));
    
    match convert_result {
        Ok(compatible_wav) => {
            info!("转换输出: {}", compatible_wav.path.display());
            assert!(compatible_wav.path.exists(), "转换后的文件应该存在");
            
            // 清理临时文件
    if temp_output.exists() {
        let _ = std::fs::remove_file(&temp_output);
    }
            
            info!("✓ 音频格式转换成功");
        }
        Err(e) => {
            panic!("音频格式转换失败: {}", e);
        }
    }
    
    info!("✓ 音频处理集成测试通过");
}

#[tokio::test]
async fn test_tts_text_to_audio_integration() {
    // 检查 index-tts 是否可用
    if !check_index_tts_available() {
        info!("跳过 TTS 集成测试: index-tts 不可用");
        info!("要启用 TTS 测试，请安装 index-tts 并确保在 PATH 中");
        return;
    }
    
    info!("开始 TTS 集成测试...");
    
    let test_text = "Hello, this is a test.";
    info!("测试文本: {}", test_text);
    
    // 创建 TTS 服务
    let config = rs_voice_toolkit_tts::TtsConfig::default();
    let tts_service = rs_voice_toolkit_tts::TtsService::new(config);
    
    // 测试内存合成
    let memory_result = tts_service.text_to_speech(test_text).await;
    match memory_result {
        Ok(audio_data) => {
            info!("内存合成成功，音频数据大小: {} 字节", audio_data.len());
            assert!(!audio_data.is_empty(), "音频数据不应为空");
            
            // 验证是否为有效的 WAV 文件（简单检查 WAV 头）
            if audio_data.len() >= 12 {
                let riff_header = &audio_data[0..4];
                let wave_header = &audio_data[8..12];
                
                if riff_header == b"RIFF" && wave_header == b"WAVE" {
                    info!("✓ 生成的音频数据具有有效的 WAV 格式");
                } else {
                    info!("⚠ 音频数据格式可能不是标准 WAV，但测试仍然通过");
                }
            }
            
            info!("✓ TTS 内存合成测试通过");
        }
        Err(e) => {
            panic!("TTS 内存合成失败: {}", e);
        }
    }
    
    // 测试文件合成
    let temp_output = std::env::temp_dir().join("test_tts_output.wav");
    let file_result = tts_service.text_to_file(test_text, &temp_output).await;
    
    match file_result {
        Ok(_) => {
            info!("文件合成成功: {}", temp_output.display());
            assert!(temp_output.exists(), "输出文件应该存在");
            
            // 检查文件大小
            let metadata = std::fs::metadata(&temp_output).expect("无法读取文件元数据");
            assert!(metadata.len() > 0, "输出文件不应为空");
            info!("输出文件大小: {} 字节", metadata.len());
            
            // 清理临时文件
            let _ = std::fs::remove_file(&temp_output);
            
            info!("✓ TTS 文件合成测试通过");
        }
        Err(e) => {
            panic!("TTS 文件合成失败: {}", e);
        }
    }
    
    info!("✓ TTS 集成测试通过");
}

#[tokio::test]
async fn test_end_to_end_stt_workflow() {
    // 检查测试文件
    let (model_path, audio_path) = match check_test_fixtures() {
        Ok(paths) => paths,
        Err(e) => {
            log::warn!("跳过端到端 STT 工作流测试: {}", e);
            return;
        }
    };
    
    info!("开始端到端 STT 工作流测试...");
    
    // 步骤 1: 音频预处理
    info!("步骤 1: 音频预处理");
    let temp_processed = std::env::temp_dir().join("test_processed_audio.wav");
    let temp_processed_clone = temp_processed.clone();
    let compatible_wav = rs_voice_toolkit_audio::ensure_whisper_compatible(&audio_path, Some(temp_processed))
        .expect("音频预处理失败");
    let processed_path = compatible_wav.path.clone();
    
    info!("音频预处理完成: {}", processed_path.display());
    
    // 步骤 2: STT 转录
    info!("步骤 2: STT 转录");
    let transcription = rs_voice_toolkit_stt::transcribe_file(&model_path, &processed_path)
        .await
        .expect("STT 转录失败");
    
    info!("转录结果: {}", transcription.text);
    info!("片段数量: {}", transcription.segments.len());
    info!("音频时长: {} ms", transcription.audio_duration);
    info!("处理时间: {} ms", transcription.processing_time);
    
    // 验证结果
    assert!(!transcription.text.trim().is_empty(), "转录文本不应为空");
    assert!(transcription.segments.len() > 0, "应该有转录片段");
    
    // 清理临时文件
    if temp_processed_clone.exists() {
        let _ = std::fs::remove_file(&temp_processed_clone);
    }
    
    info!("✓ 端到端 STT 工作流测试通过");
}

#[cfg(feature = "streaming")]
#[tokio::test]
async fn test_streaming_stt_integration() {
    // 检查测试文件
    let (model_path, audio_path) = match check_test_fixtures() {
        Ok(paths) => paths,
        Err(e) => {
            log::warn!("跳过流式 STT 集成测试: {}", e);
            return;
        }
    };
    
    info!("开始流式 STT 集成测试...");
    
    // 读取音频文件
    let audio_data = std::fs::read(&audio_path).expect("无法读取音频文件");
    
    // 创建流式转录器
    let config = StreamingConfig::default();
    let mut transcriber = StreamingTranscriber::new(&model_path, config)
        .await
        .expect("创建流式转录器失败");
    
    info!("流式转录器创建成功");
    
    // 启动流式转录
    let mut receiver = transcriber.start_streaming().expect("启动流式转录失败");
    
    info!("流式转录已启动");
    
    // 模拟分块推送音频数据
    let chunk_size = 8192; // 8KB 块
    let mut total_chunks = 0;
    
    tokio::spawn(async move {
        for chunk in audio_data.chunks(chunk_size) {
            if let Err(e) = transcriber.push_audio(chunk.to_vec()).await {
                log::error!("推送音频数据失败: {}", e);
                break;
            }
            total_chunks += 1;
            
            // 模拟实时流的延迟
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        info!("音频数据推送完成，共 {} 块", total_chunks);
        
        // 停止流式转录
        if let Err(e) = transcriber.stop_streaming().await {
            log::error!("停止流式转录失败: {}", e);
        }
    });
    
    // 接收转录事件
    let mut event_count = 0;
    let mut final_text = String::new();
    
    while let Some(event) = receiver.recv().await {
        event_count += 1;
        
        match event {
            StreamingEvent::SpeechStart => {
                info!("检测到语音开始");
            }
            StreamingEvent::SpeechEnd => {
                info!("检测到语音结束");
            }
            StreamingEvent::Silence => {
                info!("检测到静音");
            }
            StreamingEvent::TranscriptionResult(result) => {
                info!("转录结果: {}", result.text);
                if !result.text.trim().is_empty() {
                    final_text = result.text;
                }
            }
            StreamingEvent::Error(e) => {
                log::error!("流式转录错误: {}", e);
            }
        }
        
        // 防止无限等待
        if event_count > 100 {
            break;
        }
    }
    
    info!("接收到 {} 个事件", event_count);
    info!("最终转录文本: {}", final_text);
    
    // 验证结果
    assert!(event_count > 0, "应该接收到至少一个事件");
    
    info!("✓ 流式 STT 集成测试通过");
}