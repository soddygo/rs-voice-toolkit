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
}

// 导入流式转录模块
#[cfg(feature = "streaming")]
pub mod streaming;
#[cfg(feature = "streaming")]
pub use streaming::{
    create_custom_streaming_transcriber, create_streaming_transcriber, StreamingConfig,
    StreamingEvent, StreamingTranscriber,
};
