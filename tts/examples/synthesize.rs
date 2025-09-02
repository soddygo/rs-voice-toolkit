//! TTS (Text-to-Speech) 语音合成示例
//! 
//! 这个示例展示了如何使用 rs-voice-toolkit-tts 库进行文本转语音操作。
//! 它演示了基本的语音合成功能，包括引擎可用性检查和文件输出。
//! 
//! # 功能特点
//! 
//! - **命令行界面**: 支持命令行参数输入
//! - **引擎检查**: 自动检查 Index-TTS 引擎是否可用
//! - **灵活配置**: 支持自定义引擎路径
//! - **错误处理**: 提供详细的错误信息和退出码
//! - **文件输出**: 将合成的语音保存为 WAV 文件
//! 
//! # 使用方法
//! 
//! ## 基本用法
//! 
//! ```bash
//! # 使用系统 PATH 中的 index-tts 引擎
//! cargo run -p rs-voice-toolkit-tts --example synthesize -- "你好，世界" output.wav
//! 
//! # 指定自定义引擎路径
//! cargo run -p rs-voice-toolkit-tts --example synthesize -- "你好，世界" output.wav /path/to/index-tts
//! ```
//! 
//! ## 参数说明
//! 
//! - `<text>`: 要合成的文本内容
//! - `<output_wav>`: 输出的 WAV 文件路径
//! - `[index-tts-path]`: 可选，Index-TTS 引擎的完整路径
//! 
//! # 退出码
//! 
//! - `0`: 成功完成
//! - `1`: 参数错误
//! - `2`: 引擎不可用
//! - `3`: 语音合成失败
//! 
//! # 依赖要求
//! 
//! - Index-TTS 引擎必须已安装并在系统 PATH 中，或通过参数指定路径
//! - 系统必须有足够的内存和磁盘空间
//! - 输出目录必须存在且有写权限
//! 
//! # 示例输出
//! 
//! 成功运行时，控制台会显示：
//! ```text
//! 已生成: output.wav
//! ```
//! 
//! 同时会在指定路径生成一个 WAV 格式的音频文件。
//! 
//! # 错误处理
//! 
//! 程序会处理以下错误情况：
//! - 参数不足：显示用法说明
//! - 引擎不可用：提示安装或路径问题
//! - 合成失败：显示具体的错误信息
//! 
//! # 性能考虑
//! 
//! - 首次运行可能需要加载引擎，耗时较长
//! - 长文本合成会使用更多内存和处理时间
//! - 建议在合成前检查引擎可用性

use std::path::PathBuf;
use rs_voice_toolkit_tts::{TtsConfig, TtsService};

#[tokio::main]
async fn main() {
    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    
    // 检查参数数量
    if args.len() < 3 {
        eprintln!(
            "用法: cargo run -p rs-voice-toolkit-tts --example synthesize -- <text> <output_wav> [index-tts-path]"
        );
        eprintln!("示例:");
        eprintln!("  cargo run -p rs-voice-toolkit-tts --example synthesize -- \"你好，世界\" output.wav");
        eprintln!("  cargo run -p rs-voice-toolkit-tts --example synthesize -- \"Hello World\" output.wav /usr/local/bin/index-tts");
        std::process::exit(1);
    }

    // 提取参数
    let text = &args[1];
    let output = PathBuf::from(&args[2]);
    let exe = args.get(3).map(PathBuf::from);

    // 创建 TTS 配置
    let cfg = TtsConfig {
        executable_path: exe,
        ..Default::default()
    };
    
    // 创建 TTS 服务
    let svc = TtsService::new(cfg);

    // 检查引擎可用性
    if !svc.is_available().await {
        eprintln!("index-tts 不可用：请安装并确保在 PATH 或提供可执行路径");
        eprintln!("安装方法:");
        eprintln!("  1. 下载 Index-TTS 可执行文件");
        eprintln!("  2. 将其放置在系统 PATH 中或通过参数指定路径");
        eprintln!("  3. 确保文件有执行权限");
        std::process::exit(2);
    }

    // 执行语音合成
    match svc.text_to_file(text, &output).await {
        Ok(()) => {
            println!("✓ 语音合成成功");
            println!("  文本: \"{}\"", text);
            println!("  输出: {}", output.display());
            
            // 显示文件信息
            if let Ok(metadata) = std::fs::metadata(&output) {
                println!("  文件大小: {} 字节", metadata.len());
            }
        }
        Err(e) => {
            eprintln!("✗ 语音合成失败: {e}");
            eprintln!("可能的解决方案:");
            eprintln!("  1. 检查文本内容是否有效");
            eprintln!("  2. 确认输出路径有写权限");
            eprintln!("  3. 验证 Index-TTS 引擎是否正常工作");
            eprintln!("  4. 检查系统是否有足够的内存和磁盘空间");
            std::process::exit(3);
        }
    }
}
