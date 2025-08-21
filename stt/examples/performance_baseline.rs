use std::path::PathBuf;
use std::process;
use std::time::Instant;
use sysinfo::{Pid, System};

/// 性能基线测试工具
/// 记录 RTF、内存使用、处理时间等关键指标
#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "用法: cargo run -p stt --example performance_baseline -- <model_path> <audio_path> [iters]"
        );
        std::process::exit(1);
    }

    let model = PathBuf::from(&args[1]);
    let audio = PathBuf::from(&args[2]);
    let iters: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(5);

    println!("=== STT 性能基线测试 ===");
    let model_path = model.display();
    println!("模型: {model_path}");
    let audio_path = audio.display();
    println!("音频: {audio_path}");
    println!("迭代次数: {iters}");
    println!();

    // 初始化系统监控
    let mut system = System::new_all();
    let pid = process::id();

    // 预热一次，避免首次加载影响
    println!("预热中...");
    let _ = rs_voice_toolkit_stt::transcribe_file(&model, &audio).await;
    println!("预热完成\n");

    let mut metrics = PerformanceMetrics::new();

    for i in 0..iters {
        println!("--- 迭代 {} ---", i + 1);

        // 记录开始状态
        system.refresh_all();
        let start_memory = get_process_memory(&system, pid);
        let start_time = Instant::now();

        // 执行转录
        let result = rs_voice_toolkit_stt::transcribe_file(&model, &audio)
            .await
            .expect("转录失败");

        let elapsed = start_time.elapsed();

        // 记录结束状态
        system.refresh_all();
        let end_memory = get_process_memory(&system, pid);

        // 计算指标
        let rtf = result.real_time_factor();
        let processing_time_ms = elapsed.as_millis();
        let memory_delta_mb = (end_memory as i64 - start_memory as i64) / 1024 / 1024;
        let audio_duration_s = result.audio_duration as f64 / 1000.0;
        let confidence = result.average_confidence();

        // 记录指标
        metrics.add_measurement(Measurement {
            rtf,
            processing_time_ms,
            memory_delta_mb,
            audio_duration_s,
            confidence,
            text_length: result.text.len(),
            segment_count: result.segments.len(),
        });

        println!("  RTF: {rtf:.3}");
        println!("  处理时间: {processing_time_ms} ms");
        println!("  内存变化: {memory_delta_mb} MB");
        println!("  音频时长: {audio_duration_s:.1} s");
        println!("  平均置信度: {confidence:.3}");
        let text_len = result.text.len();
        println!("  文本长度: {text_len} 字符");
        let segment_count = result.segments.len();
        println!("  段落数: {segment_count}");
        let preview_text = result.text.chars().take(50).collect::<String>();
        println!("  文本: '{preview_text}'");
        println!();
    }

    // 输出统计结果
    metrics.print_summary();
}

fn get_process_memory(system: &System, pid: u32) -> u64 {
    system
        .process(Pid::from(pid as usize))
        .map(|p| p.memory())
        .unwrap_or(0)
}

#[derive(Debug)]
struct Measurement {
    rtf: f64,
    processing_time_ms: u128,
    memory_delta_mb: i64,
    audio_duration_s: f64,
    confidence: f32,
    text_length: usize,
    segment_count: usize,
}

struct PerformanceMetrics {
    measurements: Vec<Measurement>,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            measurements: Vec::new(),
        }
    }

    fn add_measurement(&mut self, measurement: Measurement) {
        self.measurements.push(measurement);
    }

    fn print_summary(&self) {
        if self.measurements.is_empty() {
            return;
        }

        println!("=== 性能基线统计 ===");

        // RTF 统计
        let rtfs: Vec<f64> = self.measurements.iter().map(|m| m.rtf).collect();
        let avg_rtf = rtfs.iter().sum::<f64>() / rtfs.len() as f64;
        let min_rtf = rtfs.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_rtf = rtfs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        println!("RTF (实时因子):");
        println!("  平均: {avg_rtf:.3}");
        println!("  最小: {min_rtf:.3}");
        println!("  最大: {max_rtf:.3}");

        // 处理时间统计
        let times: Vec<u128> = self
            .measurements
            .iter()
            .map(|m| m.processing_time_ms)
            .collect();
        let avg_time = times.iter().sum::<u128>() as f64 / times.len() as f64;
        let min_time = *times.iter().min().unwrap();
        let max_time = *times.iter().max().unwrap();

        println!("\n处理时间 (ms):");
        println!("  平均: {avg_time:.3}");
        println!("  最小: {min_time}");
        println!("  最大: {max_time}");

        // 内存使用统计
        let memories: Vec<i64> = self
            .measurements
            .iter()
            .map(|m| m.memory_delta_mb)
            .collect();
        let avg_memory = memories.iter().sum::<i64>() as f64 / memories.len() as f64;
        let min_memory = *memories.iter().min().unwrap();
        let max_memory = *memories.iter().max().unwrap();

        println!("\n内存变化 (MB):");
        println!("  平均: {avg_memory:.1}");
        println!("  最小: {min_memory}");
        println!("  最大: {max_memory}");

        // 置信度统计
        let confidences: Vec<f32> = self.measurements.iter().map(|m| m.confidence).collect();
        let avg_confidence = confidences.iter().sum::<f32>() / confidences.len() as f32;
        let min_confidence = confidences.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_confidence = confidences.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        println!("\n置信度:");
        println!("  平均: {avg_confidence:.3}");
        println!("  最小: {min_confidence:.3}");
        println!("  最大: {max_confidence:.3}");

        // 音频时长
        if let Some(first) = self.measurements.first() {
            println!("\n音频信息:");
            let duration = first.audio_duration_s;
            println!("  时长: {duration:.1} 秒");
            let avg_text_length = self
                .measurements
                .iter()
                .map(|m| m.text_length)
                .sum::<usize>() as f64
                / self.measurements.len() as f64;
            println!("  平均文本长度: {avg_text_length:.0} 字符");
            let avg_segment_count = self
                .measurements
                .iter()
                .map(|m| m.segment_count)
                .sum::<usize>() as f64
                / self.measurements.len() as f64;
            println!("  平均段落数: {avg_segment_count:.1}");
        }

        // 性能评估
        println!("\n=== 性能评估 ===");
        if avg_rtf < 0.3 {
            println!("✅ 性能优秀: RTF < 0.3，实时性能良好");
        } else if avg_rtf < 1.0 {
            println!("⚠️  性能一般: RTF < 1.0，可实时处理但有延迟");
        } else {
            println!("❌ 性能较差: RTF >= 1.0，无法实时处理");
        }

        if avg_confidence > 0.8 {
            println!("✅ 识别质量优秀: 平均置信度 > 0.8");
        } else if avg_confidence > 0.6 {
            println!("⚠️  识别质量一般: 平均置信度 > 0.6");
        } else {
            println!("❌ 识别质量较差: 平均置信度 <= 0.6");
        }

        println!("\n建议:");
        if avg_rtf > 0.5 {
            println!("- 考虑使用更小的模型或优化硬件配置");
        }
        if avg_memory > 100.0 {
            println!("- 内存使用较高，注意内存泄漏");
        }
        if avg_confidence < 0.7 {
            println!("- 考虑使用更大的模型或改善音频质量");
        }
    }
}
