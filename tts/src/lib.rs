//! # TTS (Text-to-Speech) Module
//!
//! 这个模块提供文本转语音功能。
//! 目前作为未来规划的一部分，暂时提供基础框架。

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::process::Command;

/// TTS模块的错误类型
#[derive(Error, Debug)]
pub enum TtsError {
    #[error("TTS功能尚未实现")]
    NotImplemented,
    #[error("配置错误: {0}")]
    ConfigError(String),
    #[error("音频生成错误: {0}")]
    AudioGenerationError(String),
    #[error("引擎执行错误: {0}")]
    EngineExecutionError(String),
}

/// TTS配置
#[derive(Debug, Clone)]
pub struct TtsConfig {
    /// Index-TTS 可执行文件路径（为空则查 PATH）
    pub executable_path: Option<PathBuf>,
    /// 语言
    pub language: Option<String>,
    /// 说话人
    pub speaker: Option<String>,
    /// 采样率（Hz）
    pub sample_rate: u32,
    /// 语音速度 (0.5 - 2.0)
    pub speed: f32,
    /// 音调 (-20.0 - 20.0)
    pub pitch: f32,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            executable_path: None,
            language: Some("auto".to_string()),
            speaker: None,
            sample_rate: 22050,
            speed: 1.0,
            pitch: 0.0,
        }
    }
}

/// TTS引擎类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TtsEngineType {
    /// Index-TTS 引擎
    IndexTts,
    /// Piper 引擎（未来支持）
    #[allow(dead_code)]
    Piper,
    /// Coqui 引擎（未来支持）
    #[allow(dead_code)]
    Coqui,
}

impl Default for TtsEngineType {
    fn default() -> Self {
        Self::IndexTts
    }
}

/// TTS引擎接口
#[async_trait]
pub trait TtsEngine {
    /// 将文本转换为语音
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, TtsError>;

    /// 将文本转换为语音并保存到文件
    async fn synthesize_to_file(&self, text: &str, output_path: &Path) -> Result<(), TtsError>;

    /// 检查引擎是否可用
    async fn is_available(&self) -> bool;

    /// 获取支持的语言列表
    fn supported_languages(&self) -> Vec<String>;

    /// 获取引擎类型
    fn engine_type(&self) -> TtsEngineType;
}

/// Index-TTS 引擎
#[derive(Debug, Clone)]
pub struct IndexTtsEngine {
    /// TTS配置
    cfg: TtsConfig,
}

impl IndexTtsEngine {
    pub fn new(cfg: TtsConfig) -> Self {
        Self { cfg }
    }

    pub async fn is_available(&self) -> bool {
        if let Some(path) = &self.cfg.executable_path {
            return path.exists();
        }
        which::which("index-tts").is_ok()
    }

    async fn resolve_executable(&self) -> Result<PathBuf, TtsError> {
        if let Some(path) = &self.cfg.executable_path {
            return Ok(path.clone());
        }
        which::which("index-tts").map_err(|_| {
            TtsError::ConfigError(
                "找不到 index-tts 可执行文件，请设置 PATH 或配置 executable_path".into(),
            )
        })
    }

    pub async fn synthesize_to_memory(&self, text: &str) -> Result<Vec<u8>, TtsError> {
        let exe = self.resolve_executable().await?;
        let mut args: Vec<String> = Vec::new();
        args.push("--text".into());
        args.push(text.into());
        if let Some(lang) = &self.cfg.language {
            args.push("--language".into());
            args.push(lang.clone());
        }
        if let Some(speaker) = &self.cfg.speaker {
            args.push("--speaker".into());
            args.push(speaker.clone());
        }
        args.push("--sample-rate".into());
        args.push(self.cfg.sample_rate.to_string());
        args.push("--output-format".into());
        args.push("wav".into());
        // 假设 index-tts 支持 stdout 输出；若不支持，需落盘再读出
        let output = Command::new(exe)
            .args(&args)
            .output()
            .await
            .map_err(|e| TtsError::EngineExecutionError(format!("执行失败: {e}")))?;
        if !output.status.success() {
            return Err(TtsError::EngineExecutionError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(output.stdout)
    }

    pub async fn synthesize_to_file<P: AsRef<Path>>(
        &self,
        text: &str,
        output_path: P,
    ) -> Result<(), TtsError> {
        let exe = self.resolve_executable().await?;
        let mut args: Vec<String> = Vec::new();
        args.push("--text".into());
        args.push(text.into());
        if let Some(lang) = &self.cfg.language {
            args.push("--language".into());
            args.push(lang.clone());
        }
        if let Some(speaker) = &self.cfg.speaker {
            args.push("--speaker".into());
            args.push(speaker.clone());
        }
        args.push("--sample-rate".into());
        args.push(self.cfg.sample_rate.to_string());
        args.push("--output".into());
        args.push(output_path.as_ref().to_string_lossy().to_string());
        args.push("--output-format".into());
        args.push("wav".into());
        let status = Command::new(exe)
            .args(&args)
            .status()
            .await
            .map_err(|e| TtsError::EngineExecutionError(format!("执行失败: {e}")))?;
        if !status.success() {
            return Err(TtsError::EngineExecutionError(format!(
                "Index-TTS 退出状态异常: {:?}",
                status.code()
            )));
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl TtsEngine for IndexTtsEngine {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, TtsError> {
        self.synthesize_to_memory(text).await
    }

    async fn synthesize_to_file(&self, text: &str, output_path: &Path) -> Result<(), TtsError> {
        self.synthesize_to_file(text, output_path).await
    }

    async fn is_available(&self) -> bool {
        self.is_available().await
    }

    fn supported_languages(&self) -> Vec<String> {
        vec!["zh".into(), "en".into(), "auto".into()]
    }

    fn engine_type(&self) -> TtsEngineType {
        TtsEngineType::IndexTts
    }
}

/// TTS服务
pub struct TtsService {
    #[allow(dead_code)]
    config: TtsConfig,
    engine: Box<dyn TtsEngine + Send + Sync>,
}

impl TtsService {
    /// 创建新的TTS服务
    pub fn new(config: TtsConfig) -> Self {
        Self::new_with_engine(config, TtsEngineType::default())
    }

    /// 使用指定引擎创建TTS服务
    pub fn new_with_engine(config: TtsConfig, engine_type: TtsEngineType) -> Self {
        let engine = Self::create_engine(config.clone(), engine_type);
        Self { config, engine }
    }

    /// 创建指定类型的引擎
    fn create_engine(config: TtsConfig, engine_type: TtsEngineType) -> Box<dyn TtsEngine + Send + Sync> {
        match engine_type {
            TtsEngineType::IndexTts => Box::new(IndexTtsEngine::new(config)),
            TtsEngineType::Piper => {
                // 未来实现
                panic!("Piper 引擎尚未实现")
            }
            TtsEngineType::Coqui => {
                // 未来实现
                panic!("Coqui 引擎尚未实现")
            }
        }
    }

    /// 文本转语音（内存）
    pub async fn text_to_speech(&self, text: &str) -> Result<Vec<u8>, TtsError> {
        self.engine.synthesize(text).await
    }

    /// 文本转语音并保存到文件
    pub async fn text_to_file<P: AsRef<Path>>(
        &self,
        text: &str,
        output: P,
    ) -> Result<(), TtsError> {
        self.engine.synthesize_to_file(text, output.as_ref()).await
    }

    /// 引擎可用性
    pub async fn is_available(&self) -> bool {
        self.engine.is_available().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tts_service_creation() {
        let config = TtsConfig::default();
        let service = TtsService::new(config);

        // 可用性检测（不保证 index-tts 存在，仅验证 API 不 panic）
        let _ = service.text_to_speech("你好").await.err();
    }
}
