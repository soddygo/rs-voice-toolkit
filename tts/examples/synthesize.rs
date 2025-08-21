use std::path::PathBuf;
use rs_voice_toolkit_tts::{TtsConfig, TtsService};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "用法: cargo run -p rs-voice-toolkit-tts --example synthesize -- <text> <output_wav> [index-tts-path]"
        );
        std::process::exit(1);
    }

    let text = &args[1];
    let output = PathBuf::from(&args[2]);
    let exe = args.get(3).map(PathBuf::from);

    let cfg = TtsConfig {
        executable_path: exe,
        ..Default::default()
    };
    let svc = TtsService::new(cfg);

    if !svc.is_available().await {
        eprintln!("index-tts 不可用：请安装并确保在 PATH 或提供可执行路径");
        std::process::exit(2);
    }

    match svc.text_to_file(text, &output).await {
        Ok(()) => println!("已生成: {}", output.display()),
        Err(e) => {
            eprintln!("合成失败: {e}");
            std::process::exit(3);
        }
    }
}
