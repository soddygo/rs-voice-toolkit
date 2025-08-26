use std::env;
use std::path::PathBuf;
use std::time::Duration;

use rs_voice_toolkit_stt::audio::utils::read_wav_file;
use rs_voice_toolkit_stt::{self, AudioConfig};
use log::info;
#[cfg(feature = "streaming")]
use rs_voice_toolkit_stt::{create_custom_streaming_transcriber, StreamingConfig, StreamingEvent};

#[cfg(feature = "streaming")]
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        log::error!(
            "用法: cargo run -p stt --example streaming_transcribe -- <model_path> <audio_wav> [--no-vad] [--n=<agreement_n>] [--chunk-ms=<ms>]"
        );
        std::process::exit(1);
    }

    let model_path = PathBuf::from(&args[1]);
    let audio_path = PathBuf::from(&args[2]);

    // 解析可选参数
    let mut enable_vad = true;
    let mut agreement_n: usize = 3;
    let mut chunk_ms: u64 = 500;

    for arg in args.iter().skip(3) {
        if arg == "--no-vad" {
            enable_vad = false;
        } else if let Some(val) = arg.strip_prefix("--n=") {
            agreement_n = val.parse().unwrap_or(3);
        } else if let Some(val) = arg.strip_prefix("--chunk-ms=") {
            chunk_ms = val.parse().unwrap_or(500);
        }
    }

    // 覆盖默认 streaming 配置
    let custom_cfg = StreamingConfig {
        enable_vad,
        local_agreement_n: agreement_n.max(1), // 允许单次确认
        min_audio_length: std::time::Duration::from_millis(chunk_ms),
        transcription_interval: std::time::Duration::from_millis(chunk_ms),
        ..Default::default()
    };

    // 使用自定义配置构建
    let mut transcriber = create_custom_streaming_transcriber(
        model_path.clone(),
        custom_cfg,
        AudioConfig::whisper_optimized(),
    )
    .expect("创建流式转录器失败");

    let mut rx = transcriber
        .start_streaming()
        .await
        .expect("启动流式转录失败");

    // 启动事件读取任务
    let reader = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                StreamingEvent::Transcription(res) => {
                    if !res.text.trim().is_empty() {
                        let text = &res.text;
                        info!("[转录] {text}");
                    }
                }
                StreamingEvent::SpeechStart => info!("[事件] 语音开始"),
                StreamingEvent::SpeechEnd => info!("[事件] 语音结束"),
                StreamingEvent::Silence => info!("[事件] 静音"),
                StreamingEvent::Error(e) => log::error!("[错误] {e}"),
            }
        }
    });

    // 读取 WAV 并分块推送
    let audio = read_wav_file(&audio_path).expect("读取WAV失败");
    let samples = audio.samples;
    let sr = audio.config.sample_rate as usize;
    let chunk = (sr as u64 * chunk_ms / 1000) as usize; // 自定义块大小

    for chunk_samples in samples.chunks(chunk) {
        transcriber.push_audio(chunk_samples).expect("推送音频失败");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // 等待一会让后台处理完成
    tokio::time::sleep(Duration::from_secs(2)).await;
    transcriber.stop_streaming();

    let _ = reader.await;
}

#[cfg(not(feature = "streaming"))]
fn main() {
    log::error!("此示例需要启用 'streaming' feature");
    log::error!("请使用: cargo run -p stt --example streaming_transcribe --features streaming");
    std::process::exit(1);
}
