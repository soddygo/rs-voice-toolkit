//! STT (Speech-to-Text) 性能基准测试示例
//! 
//! 这个示例展示了如何对 rs-voice-toolkit-stt 库进行性能基准测试。
//! 它通过多次运行转录任务来测量平均性能指标，帮助评估不同模型和硬件的性能表现。
//! 
//! # 功能特点
//! 
//! - **性能基准测试**: 通过多次迭代测量平均性能
//! - **预热机制**: 首次运行作为预热，避免模型加载影响结果
//! - **详细指标**: 提供RTF、处理时间等关键性能指标
//! - **可配置迭代**: 支持自定义测试次数
//! - **结果统计**: 计算平均值和总体性能评估
//! 
//! # 使用方法
//! 
//! ## 基本用法
//! 
//! ```bash
//! # 使用默认3次迭代进行基准测试
//! cargo run -p rs-voice-toolkit-stt --example bench_transcribe -- models/ggml-tiny.bin audio/hello.wav
//! 
//! # 指定迭代次数为5次
//! cargo run -p rs-voice-toolkit-stt --example bench_transcribe -- models/ggml-base.bin audio/speech.wav 5
//! 
//! # 进行10次迭代的详细测试
//! cargo run -p rs-voice-toolkit-stt --example bench_transcribe -- models/ggml-medium.bin audio/long.wav 10
//! ```
//! 
//! ## 参数说明
//! 
//! - `<model_path>`: Whisper 模型文件的路径（.bin 格式）
//! - `<audio_path>`: 要转录的音频文件路径
//! - `[iters]`: 可选，迭代次数，默认为3次
//! 
//! # 性能指标说明
//! 
//! ## RTF (Real-Time Factor)
//! - **定义**: 处理时间 / 音频时长
//! - **意义**: 衡量处理速度相对于音频长度的倍数
//! - **评估**:
//!   - RTF < 0.5: 优秀性能，可以实时处理
//!   - RTF < 1.0: 良好性能，接近实时
//!   - RTF < 2.0: 一般性能，可以接受
//!   - RTF >= 2.0: 较慢性能，需要优化
//! 
//! ## 处理时间
//! - **单位**: 毫秒 (ms)
//! - **意义**: 实际转录所需的绝对时间
//! - **影响因素**: 模型大小、音频长度、硬件性能
//! 
//! # 测试建议
//! 
//! ## 迭代次数选择
//! - **快速测试**: 3次（默认）
//! - **标准测试**: 5-10次
//! - **精确测试**: 10-20次
//! - **正式测试**: 20次以上
//! 
//! ## 测试环境
//! - **硬件**: 在目标部署硬件上测试
//! - **系统**: 关闭不必要的后台程序
//! - **温度**: 确保硬件未过热降频
//! - **电源**: 连接电源，避免电池节能模式
//! 
//! # 输出信息
//! 
//! 程序会输出以下信息：
//! - **预热阶段**: 首次运行作为预热
//! - **每次迭代**: RTF、处理时间、转录文本
//! - **统计结果**: 平均RTF、平均处理时间
//! 
//! # 输出示例
//! 
//! ```text
//! INFO 模型: models/ggml-tiny.bin
//! INFO 音频: audio/hello.wav
//! INFO 迭代次数: 3
//! INFO 迭代 0: RTF=0.342, 用时=845 ms, 文本='Hello, world!'
//! INFO 迭代 1: RTF=0.335, 用时=828 ms, 文本='Hello, world!'
//! INFO 迭代 2: RTF=0.338, 用时=835 ms, 文本='Hello, world!'
//! INFO 平均 RTF=0.338, 平均用时=836 ms
//! ```
//! 
//! # 性能优化建议
//! 
//! ## 硬件优化
//! - **CPU**: 使用多核处理器，支持AVX指令集
//! - **内存**: 确保足够内存，避免交换
//! - **存储**: 使用SSD存储模型文件
//! - **GPU**: 如果支持，启用GPU加速
//! 
//! ## 软件优化
//! - **模型选择**: 根据精度要求选择合适的模型
//! - **音频格式**: 使用WAV格式避免转换开销
//! - **并发处理**: 利用多线程进行批量处理
//! - **缓存机制**: 保持模型实例避免重复加载
//! 
//! # 应用场景
//! 
//! - **模型选择**: 比较不同模型的性能表现
//! - **硬件评估**: 评估不同硬件平台的性能
//! - **优化验证**: 验证优化措施的效果
//! - **容量规划**: 评估系统的处理能力
//! 
//! # 注意事项
//! 
//! - 首次运行包含模型加载时间，不计入统计
//! - 长时间运行可能导致硬件降频，影响结果
//! - 建议在稳定的系统环境下进行测试
//! - 测试结果可能因系统负载而有所波动

use std::path::PathBuf;
use std::time::Instant;
use log::info;

/// STT 性能基准测试示例
/// 
/// 这个示例程序用于测量语音识别的性能指标，通过多次运行转录任务
/// 来获得平均性能数据。它可以帮助开发者：
/// 
/// - 评估不同模型的性能差异
/// - 测试硬件平台的处理能力
/// - 验证优化措施的效果
/// - 进行容量规划和性能预测
/// 
/// # 使用示例
/// 
/// ```bash
/// # 基本性能测试
/// cargo run -p rs-voice-toolkit-stt --example bench_transcribe -- models/ggml-tiny.bin audio/hello.wav
/// 
/// # 指定迭代次数
/// cargo run -p rs-voice-toolkit-stt --example bench_transcribe -- models/ggml-base.bin audio/speech.wav 5
/// 
/// # 详细性能分析
/// cargo run -p rs-voice-toolkit-stt --example bench_transcribe -- models/ggml-medium.bin audio/long.wav 10
/// ```
/// 
/// # 性能指标解读
/// 
/// - **RTF (Real-Time Factor)**: 处理时间与音频时长的比率
///   - < 0.5: 优秀，可以实时处理
///   - 0.5-1.0: 良好，接近实时
///   - 1.0-2.0: 一般，可以接受
///   - > 2.0: 较慢，需要优化
/// 
/// - **处理时间**: 每次转录的实际耗时
/// - **一致性**: 多次运行结果的稳定性
/// 
/// # 用法: cargo run -p stt --example bench_transcribe -- <model_path> <audio_path> [iters]
#[tokio::main]
async fn main() {
    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        log::error!(
            "用法: cargo run -p rs-voice-toolkit-stt --example bench_transcribe -- <model_path> <audio_path> [iters]"
        );
        log::error!("示例:");
        log::error!("  cargo run -p rs-voice-toolkit-stt --example bench_transcribe -- models/ggml-tiny.bin audio/hello.wav");
        log::error!("  cargo run -p rs-voice-toolkit-stt --example bench_transcribe -- models/ggml-base.bin audio/speech.wav 5");
        std::process::exit(1);
    }

    let model = PathBuf::from(&args[1]);
    let audio = PathBuf::from(&args[2]);
    let iters: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(3);

    // 显示测试配置
    let model_path = model.display();
    let audio_path = audio.display();
    info!("🚀 性能基准测试");
    info!("{}", "=".repeat(50));
    info!("📊 测试配置:");
    info!("  模型: {model_path}");
    info!("  音频: {audio_path}");
    info!("  迭代次数: {iters}");
    
    // 验证文件存在性
    if !model.exists() {
        log::error!("❌ 模型文件不存在: {model_path}");
        std::process::exit(1);
    }
    
    if !audio.exists() {
        log::error!("❌ 音频文件不存在: {audio_path}");
        std::process::exit(1);
    }

    info!("\n🔥 预热阶段 - 首次加载模型...");
    // 预热一次，避免首次加载影响测试结果
    match rs_voice_toolkit_stt::transcribe_file(&model, &audio).await {
        Ok(_) => info!("✅ 预热完成"),
        Err(e) => {
            log::error!("❌ 预热失败: {e}");
            std::process::exit(1);
        }
    }

    info!("\n📈 开始性能测试...");
    let mut total_rt_factor = 0.0f64;
    let mut total_ms = 0u128;
    let mut results = Vec::new();

    for i in 0..iters {
        let t0 = Instant::now();
        
        match rs_voice_toolkit_stt::transcribe_file(&model, &audio).await {
            Ok(result) => {
                let dt = t0.elapsed().as_millis();
                let rtf = result.real_time_factor();
                total_rt_factor += rtf;
                total_ms += dt;
                results.push((rtf, dt, result.text.clone()));
                
                info!(
                    "🔄 迭代 {i}: RTF={rtf:.3}, 用时={dt} ms, 文本='{}'",
                    result.text
                );
            }
            Err(e) => {
                log::error!("❌ 迭代 {i} 失败: {e}");
                // 继续测试其他迭代，但记录错误
            }
        }
        
        // 在迭代之间稍作停顿，避免过热
        if i < iters - 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    // 计算并显示统计结果
    let avg_rtf = total_rt_factor / results.len() as f64;
    let avg_ms = total_ms as f64 / results.len() as f64;
    
    info!("\n📊 测试结果统计:");
    info!("{}", "=".repeat(50));
    info!("✅ 成功完成迭代: {}/{}", results.len(), iters);
    info!("📈 平均 RTF: {avg_rtf:.3}");
    info!("⏱️  平均用时: {avg_ms:.0} ms");
    
    // 性能评估
    info!("\n🎯 性能评估:");
    if avg_rtf < 0.5 {
        info!("⭐ 性能评级: 优秀 (RTF < 0.5) - 可以实时处理");
    } else if avg_rtf < 1.0 {
        info!("⭐ 性能评级: 良好 (RTF < 1.0) - 接近实时");
    } else if avg_rtf < 2.0 {
        info!("⭐ 性能评级: 一般 (RTF < 2.0) - 可以接受");
    } else {
        info!("⭐ 性能评级: 较慢 (RTF >= 2.0) - 需要优化");
    }
    
    // 稳定性分析
    if results.len() > 1 {
        let rtfs: Vec<f64> = results.iter().map(|(rtf, _, _)| *rtf).collect();
        let min_rtf = rtfs.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_rtf = rtfs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let stability = 1.0 - (max_rtf - min_rtf) / avg_rtf;
        
        info!("📊 稳定性分析:");
        info!("  最小 RTF: {min_rtf:.3}");
        info!("  最大 RTF: {max_rtf:.3}");
        info!("  稳定性评分: {:.1}%", stability * 100.0);
        
        if stability > 0.9 {
            info!("  🟢 性能非常稳定");
        } else if stability > 0.7 {
            info!("  🟡 性能比较稳定");
        } else {
            info!("  🔴 性能波动较大");
        }
    }
    
    info!("\n🎉 性能测试完成！");
}
