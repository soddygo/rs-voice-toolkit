//! Rust 语音工具库
//!
//! 提供语音转文本(STT)和文本转语音(TTS)功能的统一接口

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
