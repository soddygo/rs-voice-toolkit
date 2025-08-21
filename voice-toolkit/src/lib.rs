//! Rust 语音工具库
//!
//! 提供语音转文本(STT)和文本转语音(TTS)功能的统一接口

// 导入错误处理模块
mod error;
pub use error::{Error, Result};

// 重新导出各个模块
#[cfg(feature = "stt")]
pub use rs_voice_toolkit_stt as stt;

#[cfg(feature = "audio")]
pub use rs_voice_toolkit_audio as audio;

#[cfg(feature = "tts")]
pub use rs_voice_toolkit_tts as tts;

// 重新导出常用的类型和函数
#[cfg(feature = "stt")]
pub use rs_voice_toolkit_stt::transcribe_file;

#[cfg(all(feature = "stt", feature = "streaming"))]
pub use rs_voice_toolkit_stt::streaming::StreamingTranscriber;

// 统一错误处理包装函数
#[cfg(feature = "stt")]
mod stt_wrappers {
    use super::*;
    
    /// 转录文件（统一错误处理）
    pub async fn transcribe_file_unified<P1, P2>(
        model_path: P1,
        audio_path: P2,
    ) -> Result<crate::stt::TranscriptionResult>
    where
        P1: Into<std::path::PathBuf>,
        P2: AsRef<std::path::Path>,
    {
        crate::stt::transcribe_file(model_path, audio_path)
            .await
            .map_err(Error::from)
    }
}

// 导出统一错误处理函数
#[cfg(feature = "stt")]
pub use stt_wrappers::transcribe_file_unified;
