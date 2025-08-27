//! STT (Speech-to-Text) 语音转文本模块
//!
//! 提供基于 Whisper 的语音识别功能，支持文件转录和实时流处理

// 模块导出集中于下方；避免未使用导入

// 导入错误处理模块
pub mod error;
pub use error::{SttError, SttResult};

// 导入音频处理模块
pub mod audio;
pub use audio::{AudioConfig, AudioData, AudioFormat};

// 导入Whisper转录模块
pub mod whisper;
pub use whisper::{
    transcribe_file, transcribe_file_with_config, transcribe_file_with_language,
    transcribe_file_with_transcriber, TranscriptionResult, TranscriptionSegment, WhisperConfig,
    WhisperTranscriber,
};

// 导入VAD模块
pub mod vad;
pub use vad::SimpleVad;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_transcription_on_fixture() {
        // 定位 fixtures 模型与音频
        let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root_dir = crate_dir.parent().expect("stt crate has parent");
        let model = root_dir.join("fixtures/models/ggml-tiny.bin");
        let audio = root_dir.join("fixtures/audio/jfk.wav");

        if !model.exists() || !audio.exists() {
            eprintln!(
                "跳过: 缺少 fixtures 模型或音频 ({} , {})",
                model.display(),
                audio.display()
            );
            return;
        }

        let result = transcribe_file(&model, &audio)
            .await
            .expect("端到端转录应成功");

        assert!(!result.text.trim().is_empty(), "应产生非空文本");
        assert!(result.audio_duration > 0);
    }

    #[tokio::test]
    async fn test_transcription_bank_audio() {
        // 定位 fixtures 模型与新增的bank_audio.m4a音频文件
        let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root_dir = crate_dir.parent().expect("stt crate has parent");
        // 测试多个模型
        let models = [
            "ggml-tiny.bin",
            "ggml-small.bin",
            "ggml-medium.bin"
        ];
        let audio = root_dir.join("fixtures/audio/bank_audio.m4a");

        if !audio.exists() {
            eprintln!("跳过: 缺少音频文件: {}", audio.display());
            return;
        }

        for model_name in models {
            let model = root_dir.join("fixtures/models/").join(model_name);
            
            if !model.exists() {
                println!("跳过: 缺少模型文件: {}", model.display());
                continue;
            }

            println!("\n开始测试bank_audio.m4a文件的转录，使用模型: {}", model.display());
            
            // 方法1: 使用默认配置
            let default_result = transcribe_file(&model, &audio).await;
            println!("默认配置结果: {}", default_result.as_ref().map(|r| &r.text).unwrap_or(&String::from("失败")));
            
            // 方法2: 明确指定语言为中文
            let with_lang_result = transcribe_file_with_language(&model, &audio, "zh").await;
            println!("指定中文结果: {}", with_lang_result.as_ref().map(|r| &r.text).unwrap_or(&String::from("失败")));
            
            // 方法3: 自定义配置 - 降低置信度要求，适合不太清晰的音频
            let custom_config = WhisperConfig::new(&model)
                .with_language("zh")
                .with_temperature(0.2) // 增加温度可能提高识别率
                .with_vad(false); // 禁用VAD可能有助于捕获所有语音
            
            let custom_result = transcribe_file_with_config(&model, &audio, Some(custom_config)).await;
            println!("自定义配置结果: {}", custom_result.as_ref().map(|r| &r.text).unwrap_or(&String::from("失败")));
        }
    }

    #[tokio::test]
    async fn test_different_models_on_bank_audio() {
        // 测试不同模型在同一音频上的表现
        let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root_dir = crate_dir.parent().expect("stt crate has parent");
        let audio = root_dir.join("fixtures/audio/bank_audio.m4a");

        if !audio.exists() {
            eprintln!("跳过: 缺少音频文件: {}", audio.display());
            return;
        }

        // 尝试的模型列表
        let models_to_test = [
            "ggml-tiny.bin",
            "ggml-small.bin",
            "ggml-medium.bin",
        ];

        for model_name in models_to_test {
            let model = root_dir.join("fixtures/models/").join(model_name);
            
            if !model.exists() {
                println!("跳过: 缺少模型文件: {}", model.display());
                continue;
            }

            println!("\n测试模型: {}", model_name);
            match transcribe_file(&model, &audio).await {
                Ok(result) => {
                    println!("  转录结果: {}", result.text);
                    println!("  音频时长: {}毫秒", result.audio_duration);
                    println!("  处理时长: {}毫秒", result.processing_time);
                    println!("  实时因子: {:.2}x", result.real_time_factor());
                    println!("  检测到的语言: {:?}", result.language);
                    println!("  分段数量: {}", result.segments.len());
                },
                Err(err) => {
                    println!("  转录失败: {}", err);
                }
            }
        }
    }
}

// 导入流式转录模块
#[cfg(feature = "streaming")]
pub mod streaming;
#[cfg(feature = "streaming")]
pub use streaming::{
    create_custom_streaming_transcriber, create_streaming_transcriber, StreamingConfig,
    StreamingEvent, StreamingTranscriber,
};
