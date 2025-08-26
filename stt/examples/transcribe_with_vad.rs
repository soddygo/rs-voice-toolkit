//! VAD 功能测试示例
//!
//! 演示如何在 Whisper 转录中使用 VAD 功能

use std::env;
use std::path::PathBuf;
use rs_voice_toolkit_stt::{
    audio::utils::read_wav_file,
    whisper::{WhisperConfig, WhisperTranscriber},
};
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        log::error!(
            "用法: {} <模型路径> <音频文件> [--enable-vad] [--vad-threshold=0.01]",
            args[0]
        );
        std::process::exit(1);
    }

    let model_path = PathBuf::from(&args[1]);
    let audio_path = PathBuf::from(&args[2]);

    // 解析命令行参数
    let mut enable_vad = false;
    let mut vad_threshold = 0.01;

    for arg in &args[3..] {
        if arg == "--enable-vad" {
            enable_vad = true;
        } else if arg.starts_with("--vad-threshold=") {
            if let Some(threshold_str) = arg.strip_prefix("--vad-threshold=") {
                vad_threshold = threshold_str.parse().unwrap_or(0.01);
            }
        }
    }

    let model_display = model_path.display();
    info!("模型路径: {model_display}");
    let audio_display = audio_path.display();
    info!("音频文件: {audio_display}");
    info!("VAD 启用: {enable_vad}");
    info!("VAD 阈值: {vad_threshold}");
    info!("");

    // 创建配置
    let config = WhisperConfig::new(model_path)
        .with_language("zh".to_string())
        .with_vad(enable_vad)
        .with_vad_threshold(vad_threshold);

    // 验证配置
    config.validate()?;

    // 创建转录器
    let transcriber = WhisperTranscriber::new(config)?;

    // 读取音频数据
    let audio_data = read_wav_file(&audio_path)?;
    info!("音频信息:");
    info!("  时长: {:.2}秒", audio_data.duration());
    info!("  采样率: {}Hz", audio_data.config.sample_rate);
    info!("  声道数: {}", audio_data.config.channels);
    info!("");

    // 执行转录
    info!("开始转录...");
    let start_time = std::time::Instant::now();

    let result = transcriber.transcribe_audio_data(&audio_data).await?;

    let elapsed = start_time.elapsed();

    // 输出结果
    info!("转录完成!");
    info!("处理时间: {:.2}秒", elapsed.as_secs_f64());
    info!("实时因子: {:.2}x", result.real_time_factor());
    info!("");

    if result.text.trim().is_empty() {
        info!("转录结果: [空] (可能被 VAD 过滤)");
    } else {
        info!("转录结果: {}", result.text.trim());
    }

    if !result.segments.is_empty() {
        info!("");
        info!("分段信息:");
        for (i, segment) in result.segments.iter().enumerate() {
            info!(
                "  [{}] {:.2}s-{:.2}s: {} (置信度: {:.2})",
                i + 1,
                segment.start_time as f64 / 1000.0,
                segment.end_time as f64 / 1000.0,
                segment.text.trim(),
                segment.confidence
            );
        }
    }

    Ok(())
}
