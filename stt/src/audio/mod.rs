//! 音频处理工具模块
//!
//! 提供音频格式转换、重采样、预处理等功能

pub mod converter;
pub mod resampler;
pub mod utils;

pub use converter::AudioConverter;
pub use resampler::AudioResampler;
pub use utils::*;

/// 音频格式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Wav,
    Mp3,
    Flac,
    M4a,
    Ogg,
}

impl AudioFormat {
    /// 从文件扩展名推断音频格式
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "wav" => Some(AudioFormat::Wav),
            "mp3" => Some(AudioFormat::Mp3),
            "flac" => Some(AudioFormat::Flac),
            "m4a" => Some(AudioFormat::M4a),
            "ogg" => Some(AudioFormat::Ogg),
            _ => None,
        }
    }

    /// 获取格式的文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Flac => "flac",
            AudioFormat::M4a => "m4a",
            AudioFormat::Ogg => "ogg",
        }
    }

    /// 检查格式是否被Whisper原生支持
    pub fn is_whisper_native(&self) -> bool {
        matches!(self, AudioFormat::Wav)
    }
}

/// 音频参数配置
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// 采样率 (Hz)
    pub sample_rate: u32,
    /// 声道数
    pub channels: u16,
    /// 位深度
    pub bit_depth: u16,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000, // Whisper 推荐的采样率
            channels: 1,        // 单声道
            bit_depth: 16,      // 16位
        }
    }
}

impl AudioConfig {
    /// 创建新的音频配置
    pub fn new(sample_rate: u32, channels: u16, bit_depth: u16) -> Self {
        Self {
            sample_rate,
            channels,
            bit_depth,
        }
    }

    /// 创建Whisper优化的配置
    pub fn whisper_optimized() -> Self {
        Self::default()
    }

    /// 检查配置是否与Whisper兼容
    pub fn is_whisper_compatible(&self) -> bool {
        self.sample_rate == 16000 && self.channels == 1
    }
}
