//! STT (Speech-to-Text) 文件转录示例
//! 
//! 这个示例展示了如何使用 rs-voice-toolkit-stt 库进行音频文件转录。
//! 它演示了完整的语音识别流程，包括文件验证、模型加载、转录处理和结果分析。
//! 
//! # 功能特点
//! 
//! - **命令行界面**: 支持命令行参数输入模型和音频文件路径
//! - **文件验证**: 自动检查模型文件和音频文件是否存在
//! - **详细日志**: 提供完整的处理过程日志和性能指标
//! - **结果分析**: 显示转录文本、语言检测、段落信息和置信度
//! - **性能监控**: 计算并显示处理时间、实时因子等性能指标
//! 
//! # 使用方法
//! 
//! ## 基本用法
//! 
//! ```bash
//! # 使用 tiny 模型转录音频文件
//! cargo run -p rs-voice-toolkit-stt --example transcribe_file -- models/ggml-tiny.bin audio/hello.wav
//! 
//! # 使用 base 模型转录音频文件
//! cargo run -p rs-voice-toolkit-stt --example transcribe_file -- models/ggml-base.bin audio/speech.wav
//! ```
//! 
//! ## 参数说明
//! 
//! - `<model_path>`: Whisper 模型文件的路径（.bin 格式）
//! - `<audio_path>`: 要转录的音频文件路径
//! 
//! # 支持的音频格式
//! 
//! - WAV: 原生支持，推荐格式
//! - MP3: 自动转换为兼容格式
//! - FLAC: 自动转换为兼容格式
//! - M4A: 自动转换为兼容格式
//! - OGG: 自动转换为兼容格式
//! 
//! # 支持的模型
//! 
//! - ggml-tiny.bin: 最小模型，速度快，适合测试
//! - ggml-base.bin: 基础模型，平衡性能和准确度
//! - ggml-small.bin: 小型模型，较好准确度
//! - ggml-medium.bin: 中型模型，高准确度
//! - ggml-large.bin: 大型模型，最高准确度
//! 
//! # 输出信息
//! 
//! 程序会输出以下信息：
//! - **转录文本**: 完整的转录结果
//! - **检测语言**: 自动检测的语言类型
//! - **段落数量**: 识别的语音段落数
//! - **音频时长**: 原始音频的长度
//! - **处理时间**: 转录所需的实际时间
//! - **实时因子**: RTF (Real-Time Factor)，值越小表示性能越好
//! - **平均置信度**: 转录结果的可信度评分
//! - **前3个段落**: 详细的段落信息（时间戳、置信度、文本）
//! 
//! # 性能指标说明
//! 
//! - **RTF (Real-Time Factor)**: 处理时间 / 音频时长
//!   - RTF < 1.0: 能够实时处理
//!   - RTF > 1.0: 处理时间比音频时长更长
//! - **置信度**: 0.0 到 1.0 之间的数值，越高表示越可靠
//! 
//! # 退出码
//! 
//! - `0`: 成功完成
//! - `1`: 参数错误或文件不存在
//! - `2`: 转录过程失败
//! 
//! # 依赖要求
//! 
//! - Whisper 模型文件（.bin 格式）
//! - 足够的系统内存（取决于模型大小）
//! - 支持的音频文件
//! 
//! # 性能考虑
//! 
//! - 模型越大，准确度越高但处理时间越长
//! - 音频文件长度会影响处理时间
//! - 建议根据应用场景选择合适的模型
//! - 首次运行可能需要加载模型，耗时较长
//! 
//! # 错误处理
//! 
//! 程序会处理以下错误情况：
//! - 参数不足：显示用法说明
//! - 模型文件不存在：提示检查文件路径
//! - 音频文件不存在：提示检查文件路径
//! - 转录失败：显示具体的错误信息

use log::{info, warn};
use std::env;
use std::path::PathBuf;
use rs_voice_toolkit_stt::transcribe_file;

/// 文件转录示例
/// 
/// 这个示例程序演示了如何使用 Whisper 模型进行音频文件转录。
/// 它提供了完整的错误处理、性能监控和结果分析功能。
/// 
/// # 使用示例
/// 
/// ```bash
/// # 基本用法
/// cargo run -p rs-voice-toolkit-stt --example transcribe_file -- models/ggml-tiny.bin audio/hello.wav
/// 
/// # 使用完整路径
/// cargo run -p rs-voice-toolkit-stt --example transcribe_file -- /path/to/model.bin /path/to/audio.wav
/// ```
/// 
/// # 输出示例
/// 
/// 成功运行时，控制台会显示类似以下信息：
/// ```text
/// INFO 开始转录文件: audio/hello.wav
/// INFO 使用模型: models/ggml-tiny.bin
/// 
/// INFO 转录结果:
/// INFO 文本: Hello, welcome to the voice toolkit.
/// INFO 检测到的语言: en
/// INFO 段数: 1
/// INFO 音频时长: 2.45秒
/// INFO 处理时间: 0.82秒
/// INFO 实时因子(RTF): 0.335
/// INFO 平均置信度: 0.856
/// 
/// INFO 前3个段落:
/// INFO 1. [0.5s - 2.3s] 0.856: Hello, welcome to the voice toolkit.
/// ```
/// 
/// # 性能建议
/// 
/// - 对于实时应用，建议使用 tiny 或 base 模型
/// - 对于高精度要求，可以使用 medium 或 large 模型
/// - 长音频文件建议分段处理
/// - 首次运行后，模型会被缓存，后续运行会更快
/// 
/// # 用法: cargo run -p stt --example transcribe_file -- <model_path> <audio_path>
#[tokio::main]
async fn main() {
    // 初始化日志系统
    env_logger::init();

    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        log::error!("用法: cargo run -p rs-voice-toolkit-stt --example transcribe_file -- <model_path> <audio_path>");
        log::error!("示例:");
        log::error!("  cargo run -p rs-voice-toolkit-stt --example transcribe_file -- models/ggml-tiny.bin audio/hello.wav");
        log::error!("  cargo run -p rs-voice-toolkit-stt --example transcribe_file -- models/ggml-base.bin audio/speech.wav");
        std::process::exit(1);
    }

    let model_path = PathBuf::from(&args[1]);
    let audio_path = PathBuf::from(&args[2]);

    // 验证模型文件是否存在
    if !model_path.exists() {
        let model_display = model_path.display();
        log::error!("错误: 模型文件不存在: {model_display}");
        log::error!("请确保:");
        log::error!("  1. 模型文件路径正确");
        log::error!("  2. 模型文件已下载");
        log::error!("  3. 有足够的权限访问文件");
        std::process::exit(1);
    }

    // 验证音频文件是否存在
    if !audio_path.exists() {
        let audio_display = audio_path.display();
        log::error!("错误: 音频文件不存在: {audio_display}");
        log::error!("请确保:");
        log::error!("  1. 音频文件路径正确");
        log::error!("  2. 音频文件存在");
        log::error!("  3. 支持该音频格式");
        std::process::exit(1);
    }

    // 显示处理信息
    let audio_display = audio_path.display();
    info!("🎙️  开始转录文件: {audio_display}");
    let model_display = model_path.display();
    info!("🤖 使用模型: {model_display}");

    // 记录开始时间
    let start_time = std::time::Instant::now();

    // 执行转录
    match transcribe_file(&model_path, &audio_path).await {
        Ok(result) => {
            let elapsed = start_time.elapsed();
            
            info!("\n📝 转录结果:");
            info!("{}", "=".repeat(50));
            
            // 基本信息
            let text = &result.text;
            info!("📄 文本: {text}");
            
            if let Some(ref lang) = result.language {
                info!("🌐 检测到的语言: {lang}");
            }
            
            // 统计信息
            let segment_count = result.segments.len();
            info!("📊 段落数量: {segment_count}");
            info!("⏱️  音频时长: {:.2}秒", result.audio_duration as f64 / 1000.0);
            info!("⚡ 处理时间: {:.2}秒", elapsed.as_secs_f64());
            info!("📈 实时因子(RTF): {:.3}", result.real_time_factor());
            info!("🎯 平均置信度: {:.3}", result.average_confidence());
            
            // 性能评估
            let rtf = result.real_time_factor();
            if rtf < 0.5 {
                info!("⭐ 性能评级: 优秀 (RTF < 0.5)");
            } else if rtf < 1.0 {
                info!("⭐ 性能评级: 良好 (RTF < 1.0)");
            } else if rtf < 2.0 {
                info!("⭐ 性能评级: 一般 (RTF < 2.0)");
            } else {
                info!("⭐ 性能评级: 较慢 (RTF >= 2.0)");
            }
            
            // 详细段落信息
            if !result.segments.is_empty() {
                info!("\n📝 详细段落信息 (前3个):");
                info!("{}", "-".repeat(50));
                for (i, segment) in result.segments.iter().take(3).enumerate() {
                    info!(
                        "{}. [{}s - {}s] {:.3}: {}",
                        i + 1,
                        segment.start_time as f64 / 1000.0,
                        segment.end_time as f64 / 1000.0,
                        segment.confidence,
                        segment.text
                    );
                }
                
                if result.segments.len() > 3 {
                    info!("... 还有 {} 个段落", result.segments.len() - 3);
                }
            }
            
            info!("\n✅ 转录完成！");
        }
        Err(e) => {
            warn!("❌ 转录失败: {e}");
            log::error!("详细错误信息: {e}");
            log::error!("可能的解决方案:");
            log::error!("  1. 检查模型文件是否完整");
            log::error!("  2. 确认音频格式是否受支持");
            log::error!("  3. 验证系统是否有足够的内存");
            log::error!("  4. 检查模型和音频文件的权限");
            std::process::exit(2);
        }
    }
}
