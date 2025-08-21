use log::{info, warn};
use std::env;
use std::path::PathBuf;
use stt::transcribe_file;

/// 文件转录示例
/// 用法: cargo run -p stt --example transcribe_file -- <model_path> <audio_path>
#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("用法: cargo run -p stt --example transcribe_file -- <model_path> <audio_path>");
        std::process::exit(1);
    }

    let model_path = PathBuf::from(&args[1]);
    let audio_path = PathBuf::from(&args[2]);

    // 检查文件是否存在
    if !model_path.exists() {
        let model_display = model_path.display();
        eprintln!("错误: 模型文件不存在: {model_display}");
        std::process::exit(1);
    }

    if !audio_path.exists() {
        let audio_display = audio_path.display();
        eprintln!("错误: 音频文件不存在: {audio_display}");
        std::process::exit(1);
    }

    let audio_display = audio_path.display();
    info!("开始转录文件: {audio_display}");
    let model_display = model_path.display();
    info!("使用模型: {model_display}");

    let start_time = std::time::Instant::now();

    match transcribe_file(&model_path, &audio_path).await {
        Ok(result) => {
            let elapsed = start_time.elapsed();
            println!("\n转录结果:");
            let text = &result.text;
            println!("文本: {text}");
            if let Some(ref lang) = result.language {
                println!("检测到的语言: {lang}");
            }
            let segment_count = result.segments.len();
            println!("段数: {segment_count}");
            println!("音频时长: {:.2}秒", result.audio_duration as f64 / 1000.0);
            println!("处理时间: {:.2}秒", elapsed.as_secs_f64());
            println!("实时因子(RTF): {:.3}", result.real_time_factor());
            println!("平均置信度: {:.3}", result.average_confidence());

            // 打印前几个段落（如果有）
            if !result.segments.is_empty() {
                println!("\n前3个段落:");
                for (i, segment) in result.segments.iter().take(3).enumerate() {
                    println!(
                        "{}. [{}s - {}s] {:.3}: {}",
                        i + 1,
                        segment.start_time as f64 / 1000.0,
                        segment.end_time as f64 / 1000.0,
                        segment.confidence,
                        segment.text
                    );
                }
            }
        }
        Err(e) => {
            warn!("转录失败: {e}");
            eprintln!("错误: {e}");
            // 打印错误详情
            eprintln!("错误详情: {e}");
            std::process::exit(2);
        }
    }
}
