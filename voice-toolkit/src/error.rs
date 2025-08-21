//! 统一错误处理模块
//!
//! 提供跨所有语音工具模块的统一错误类型

use thiserror::Error;

/// 统一错误类型
#[derive(Error, Debug)]
pub enum Error {
    /// 音频处理错误
    #[cfg(feature = "audio")]
    #[error("音频错误: {0}")]
    Audio(rs_voice_toolkit_audio::AudioError),

    /// 语音转文本错误
    #[cfg(feature = "stt")]
    #[error("语音识别错误: {0}")]
    Stt(rs_voice_toolkit_stt::SttError),

    /// 文本转语音错误
    #[cfg(feature = "tts")]
    #[error("语音合成错误: {0}")]
    Tts(rs_voice_toolkit_tts::TtsError),

    /// IO错误
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    /// 其他错误
    #[error("其他错误: {0}")]
    Other(String),
}

/// 统一结果类型别名
pub type Result<T> = std::result::Result<T, Error>;

/// 错误辅助函数
impl Error {
    /// 创建其他错误
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Error::Other(msg.into())
    }
}

/// 从字符串转换
impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::Other(err)
    }
}

/// 从&str转换
impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::Other(err.to_string())
    }
}

/// 从STT错误转换
#[cfg(feature = "stt")]
impl From<rs_voice_toolkit_stt::SttError> for Error {
    fn from(err: rs_voice_toolkit_stt::SttError) -> Self {
        Error::Stt(err)
    }
}

/// 从音频错误转换
#[cfg(feature = "audio")]
impl From<rs_voice_toolkit_audio::AudioError> for Error {
    fn from(err: rs_voice_toolkit_audio::AudioError) -> Self {
        Error::Audio(err)
    }
}

/// 从TTS错误转换
#[cfg(feature = "tts")]
impl From<rs_voice_toolkit_tts::TtsError> for Error {
    fn from(err: rs_voice_toolkit_tts::TtsError) -> Self {
        Error::Tts(err)
    }
}