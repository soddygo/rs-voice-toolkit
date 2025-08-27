//! # Video Processing Module
//!
//! 这个模块提供视频处理功能，主要使用 ez-ffmpeg 来调用 FFmpeg 命令。
//! 主要功能包括：
//! - 从视频文件中提取音频
//! - 音频格式转换
//! - 视频基本信息获取

use std::path::{Path, PathBuf};
use thiserror::Error;
use ffmpeg_sidecar::command::FfmpegCommand;

/// 视频处理模块的错误类型
#[derive(Error, Debug)]
pub enum VideoError {
    #[error("FFmpeg 执行错误: {0}")]
    FfmpegError(String),
    #[error("文件不存在: {0}")]
    FileNotFound(String),
    #[error("不支持的格式: {0}")]
    UnsupportedFormat(String),
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    #[error("路径错误: {0}")]
    PathError(String),
}

/// 音频提取配置
#[derive(Debug, Clone)]
pub struct AudioExtractionConfig {
    /// 输出音频格式 (wav, mp3, flac 等)
    pub format: String,
    /// 采样率 (Hz)
    pub sample_rate: Option<u32>,
    /// 声道数 (1=单声道, 2=立体声)
    pub channels: Option<u32>,
    /// 音频比特率 (kbps)
    pub bitrate: Option<String>,
}

impl Default for AudioExtractionConfig {
    fn default() -> Self {
        Self {
            format: "wav".to_string(),
            sample_rate: Some(16000), // 16kHz 适合语音识别
            channels: Some(1),        // 单声道
            bitrate: None,
        }
    }
}

/// 视频信息
#[derive(Debug, Clone)]
pub struct VideoInfo {
    /// 视频时长 (秒)
    pub duration: Option<f64>,
    /// 视频宽度
    pub width: Option<u32>,
    /// 视频高度
    pub height: Option<u32>,
    /// 帧率
    pub fps: Option<f64>,
    /// 音频采样率
    pub audio_sample_rate: Option<u32>,
    /// 音频声道数
    pub audio_channels: Option<u32>,
}

/// 视频处理服务
pub struct VideoProcessor {
    /// FFmpeg可执行文件的路径（可选）
    ffmpeg_path: Option<PathBuf>,
}

impl VideoProcessor {
    /// 创建新的视频处理器实例
    pub fn new() -> Self {
        Self {
            ffmpeg_path: None,
        }
    }
    
    /// 设置自定义的 FFmpeg 路径
    pub fn with_ffmpeg_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.ffmpeg_path = Some(path.as_ref().to_path_buf());
        self
    }
    
    /// 从视频文件中提取音频
    pub async fn extract_audio<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
        config: Option<AudioExtractionConfig>,
    ) -> Result<(), VideoError> {
        let input = input_path.as_ref();
        let output = output_path.as_ref();
        let config = config.unwrap_or_default();
        
        // 检查输入文件是否存在
        if !input.exists() {
            return Err(VideoError::FileNotFound(input.display().to_string()));
        }
        
        // 使用 ffmpeg-sidecar 进行音频提取
        let mut command = FfmpegCommand::new()
            .input(input)
            .overwrite();
        
        // 构建过滤器描述
        let mut filter_parts = Vec::new();
        
        // 设置采样率
        if let Some(sample_rate) = config.sample_rate {
            filter_parts.push(format!("aresample={}", sample_rate));
        }
        
        // 设置声道数
        if let Some(channels) = config.channels {
            filter_parts.push(format!("pan={}c", channels));
        }
        
        // 如果有过滤器，添加到命令
        if !filter_parts.is_empty() {
            command = command.args(["-filter:a", &filter_parts.join(",")]);
        }
        
        // 执行命令
        let status = command
            .output(output.to_string_lossy())
            .spawn()?
            .wait()?;

        if !status.success() {
            return Err(VideoError::FfmpegError("FFmpeg转换失败".to_string()));
        }
        
        // 如果执行到这里说明成功完成
        log::info!("音频提取成功: {} -> {}", input.display(), output.display());
        Ok(())
    }
    
    /// 获取视频文件信息
    pub async fn get_video_info<P: AsRef<Path>>(
        &self,
        input_path: P,
    ) -> Result<VideoInfo, VideoError> {
        let input = input_path.as_ref();
        
        // 检查输入文件是否存在
        if !input.exists() {
            return Err(VideoError::FileNotFound(input.display().to_string()));
        }
        
        // 注意：ez-ffmpeg 主要用于媒体处理，不直接支持 ffprobe 功能
        // 这里提供一个占位符实现，实际项目中可能需要：
        // 1. 使用其他库如 ffprobe-rs
        // 2. 直接调用 ffprobe 命令行工具
        // 3. 使用 rust-ffmpeg 等更底层的绑定
        log::warn!("视频信息获取功能需要进一步实现，当前ez-ffmpeg主要用于媒体转换");
        
        // 返回默认的视频信息
        Ok(VideoInfo {
            duration: None,
            width: None,
            height: None,
            fps: None,
            audio_sample_rate: None,
            audio_channels: None,
        })
    }
    
    /// 转换音频格式
    pub async fn convert_audio<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
        config: AudioExtractionConfig,
    ) -> Result<(), VideoError> {
        // 复用音频提取功能
        self.extract_audio(input_path, output_path, Some(config)).await
    }
}

impl Default for VideoProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_audio_extraction_config_default() {
        let config = AudioExtractionConfig::default();
        assert_eq!(config.format, "wav");
        assert_eq!(config.sample_rate, Some(16000));
        assert_eq!(config.channels, Some(1));
        assert!(config.bitrate.is_none());
    }
    
    #[test]
    fn test_video_processor_creation() {
        let processor = VideoProcessor::new();
        assert!(processor.ffmpeg_path.is_none());
        
        let processor_with_path = VideoProcessor::new()
            .with_ffmpeg_path("/usr/local/bin/ffmpeg");
        assert!(processor_with_path.ffmpeg_path.is_some());
    }
    
    #[tokio::test]
    async fn test_extract_audio_file_not_found() {
        let processor = VideoProcessor::new();
        let result = processor.extract_audio(
            "nonexistent.mp4",
            "output.wav",
            None,
        ).await;
        
        assert!(matches!(result, Err(VideoError::FileNotFound(_))));
    }
}