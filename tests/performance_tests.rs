//! 性能测试模块
//!
//! 记录和测量语音工具包的性能指标：
//! - RTF (Real-Time Factor): 处理时间 / 音频时长
//! - 内存使用情况
//! - 延迟测量
//! - 历史性能比较

use std::path::PathBuf;
use std::time::{Duration, Instant};
use sysinfo::{System, Process, Pid};
use tokio::time::sleep;
use serde::{Deserialize, Serialize};
use std::fs;

use rs_voice_toolkit_stt::transcribe_file;
use rs_voice_toolkit_tts::{TtsService, TtsConfig};
use rs_voice_toolkit_audio::{probe, ensure_whisper_compatible};

/// 性能指标结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 测试名称
    pub test_name: String,
    /// 测试时间戳
    pub timestamp: String,
    /// 实时因子 (处理时间 / 音频时长)
    pub rtf: f64,
    /// 处理时间 (毫秒)
    pub processing_time_ms: u64,
    /// 音频时长 (毫秒)
    pub audio_duration_ms: u64,
    /// 峰值内存使用 (MB)
    pub peak_memory_mb: f64,
    /// 平均内存使用 (MB)
    pub avg_memory_mb: f64,
    /// 延迟 (毫秒)
    pub latency_ms: Option<u64>,
    /// 模型大小 (MB)
    pub model_size_mb: Option<f64>,
    /// 其他元数据
    pub metadata: std::collections::HashMap<String, String>,
}

impl PerformanceMetrics {
    pub fn new(test_name: String) -> Self {
        Self {
            test_name,
            timestamp: chrono::Utc::now().to_rfc3339(),
            rtf: 0.0,
            processing_time_ms: 0,
            audio_duration_ms: 0,
            peak_memory_mb: 0.0,
            avg_memory_mb: 0.0,
            latency_ms: None,
            model_size_mb: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 计算实时因子
    pub fn calculate_rtf(&mut self) {
        if self.audio_duration_ms > 0 {
            self.rtf = self.processing_time_ms as f64 / self.audio_duration_ms as f64;
        }
    }

    /// 添加元数据
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

/// 内存监控器
pub struct MemoryMonitor {
    system: System,
    pid: u32,
    samples: Vec<f64>,
    monitoring: bool,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_all();
        let pid = std::process::id();
        
        Self {
            system,
            pid,
            samples: Vec::new(),
            monitoring: false,
        }
    }

    /// 开始监控内存
    pub async fn start_monitoring(&mut self) {
        self.monitoring = true;
        self.samples.clear();
        
        while self.monitoring {
            self.system.refresh_process(Pid::from_u32(self.pid));
            if let Some(process) = self.system.process(Pid::from_u32(self.pid)) {
                let memory_mb = process.memory() as f64 / 1024.0 / 1024.0;
                self.samples.push(memory_mb);
            }
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// 停止监控
    pub fn stop_monitoring(&mut self) {
        self.monitoring = false;
    }

    /// 获取峰值内存
    pub fn peak_memory(&self) -> f64 {
        self.samples.iter().fold(0.0, |a, &b| a.max(b))
    }

    /// 获取平均内存
    pub fn average_memory(&self) -> f64 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.samples.iter().sum::<f64>() / self.samples.len() as f64
        }
    }
}

/// 性能测试工具
pub struct PerformanceTester {
    results_dir: PathBuf,
}

impl PerformanceTester {
    pub fn new() -> Self {
        let results_dir = PathBuf::from("performance_results");
        if !results_dir.exists() {
            let _ = fs::create_dir_all(&results_dir);
        }
        
        Self { results_dir }
    }

    /// 保存性能指标
    pub fn save_metrics(&self, metrics: &PerformanceMetrics) -> Result<(), Box<dyn std::error::Error>> {
        let filename = format!(
            "{}_{}.json",
            metrics.test_name.replace(" ", "_"),
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let filepath = self.results_dir.join(filename);
        
        let json = serde_json::to_string_pretty(metrics)?;
        fs::write(filepath, json)?;
        
        Ok(())
    }

    /// 加载历史性能数据
    pub fn load_historical_metrics(&self, test_name: &str) -> Result<Vec<PerformanceMetrics>, Box<dyn std::error::Error>> {
        let mut metrics = Vec::new();
        
        if !self.results_dir.exists() {
            return Ok(metrics);
        }
        
        for entry in fs::read_dir(&self.results_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    if filename.starts_with(&test_name.replace(" ", "_")) {
                        let content = fs::read_to_string(&path)?;
                        if let Ok(metric) = serde_json::from_str::<PerformanceMetrics>(&content) {
                            metrics.push(metric);
                        }
                    }
                }
            }
        }
        
        // 按时间戳排序
        metrics.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        Ok(metrics)
    }

    /// 比较性能趋势
    pub fn analyze_performance_trend(&self, test_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let metrics = self.load_historical_metrics(test_name)?;
        
        if metrics.len() < 2 {
            println!("需要至少 2 个历史数据点来分析趋势");
            return Ok(());
        }
        
        println!("\n=== {} 性能趋势分析 ===", test_name);
        println!("{:<20} {:<10} {:<15} {:<15} {:<15}", "时间戳", "RTF", "处理时间(ms)", "峰值内存(MB)", "平均内存(MB)");
        println!("{}", "-".repeat(80));
        
        for metric in &metrics {
            println!(
                "{:<20} {:<10.3} {:<15} {:<15.2} {:<15.2}",
                &metric.timestamp[..19], // 只显示日期和时间部分
                metric.rtf,
                metric.processing_time_ms,
                metric.peak_memory_mb,
                metric.avg_memory_mb
            );
        }
        
        // 计算趋势
        let latest = &metrics[metrics.len() - 1];
        let previous = &metrics[metrics.len() - 2];
        
        let rtf_change = ((latest.rtf - previous.rtf) / previous.rtf) * 100.0;
        let memory_change = ((latest.peak_memory_mb - previous.peak_memory_mb) / previous.peak_memory_mb) * 100.0;
        
        println!("\n=== 变化趋势 ===");
        println!("RTF 变化: {:.2}%", rtf_change);
        println!("峰值内存变化: {:.2}%", memory_change);
        
        if rtf_change > 10.0 {
            println!("⚠️  警告: RTF 显著增加，性能可能下降");
        } else if rtf_change < -10.0 {
            println!("✅ 性能改善: RTF 显著降低");
        }
        
        if memory_change > 20.0 {
            println!("⚠️  警告: 内存使用显著增加");
        }
        
        Ok(())
    }
}

/// 检查测试文件是否存在
fn check_test_files() -> (PathBuf, PathBuf) {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_dir = crate_dir.parent().expect("tests crate has parent");
    let model_path = root_dir.join("fixtures/models/ggml-tiny.bin");
    let audio_path = root_dir.join("fixtures/audio/jfk.wav");
    
    if !model_path.exists() || !audio_path.exists() {
        panic!(
            "测试文件不存在: 模型 {} 音频 {}",
            model_path.display(),
            audio_path.display()
        );
    }
    
    (model_path, audio_path)
}

#[tokio::test]
async fn test_stt_performance_baseline() {
    let (model_path, audio_path) = check_test_files();
    let tester = PerformanceTester::new();
    let mut metrics = PerformanceMetrics::new("STT_File_Transcription".to_string());
    
    // 获取音频信息
    let audio_meta = probe(&audio_path).expect("获取音频信息失败");
    metrics.audio_duration_ms = audio_meta.duration_ms.unwrap_or(0);
    
    // 获取模型大小
    if let Ok(model_metadata) = fs::metadata(&model_path) {
        metrics.model_size_mb = Some(model_metadata.len() as f64 / 1024.0 / 1024.0);
    }
    
    // 启动内存监控
    let mut memory_monitor = MemoryMonitor::new();
    let monitor_handle = {
        let mut monitor = MemoryMonitor::new();
        tokio::spawn(async move {
            monitor.start_monitoring().await;
            monitor
        })
    };
    
    // 执行转录并测量时间
    let start_time = Instant::now();
    let result = transcribe_file(&model_path, &audio_path).await;
    let processing_time = start_time.elapsed();
    
    // 停止内存监控
    memory_monitor.stop_monitoring();
    let final_monitor = monitor_handle.await.expect("内存监控任务失败");
    
    // 验证转录成功
    assert!(result.is_ok(), "转录应该成功");
    let transcription = result.unwrap();
    
    // 记录性能指标
    metrics.processing_time_ms = processing_time.as_millis() as u64;
    metrics.peak_memory_mb = final_monitor.peak_memory();
    metrics.avg_memory_mb = final_monitor.average_memory();
    metrics.calculate_rtf();
    
    // 添加元数据
    metrics.add_metadata("model_file".to_string(), model_path.file_name().unwrap().to_string_lossy().to_string());
    metrics.add_metadata("audio_file".to_string(), audio_path.file_name().unwrap().to_string_lossy().to_string());
    metrics.add_metadata("transcription_length".to_string(), transcription.text.len().to_string());
    metrics.add_metadata("segments_count".to_string(), transcription.segments.len().to_string());
    
    // 打印结果
    println!("\n=== STT 性能基线测试结果 ===");
    println!("音频时长: {} ms", metrics.audio_duration_ms);
    println!("处理时间: {} ms", metrics.processing_time_ms);
    println!("实时因子 (RTF): {:.3}", metrics.rtf);
    println!("峰值内存: {:.2} MB", metrics.peak_memory_mb);
    println!("平均内存: {:.2} MB", metrics.avg_memory_mb);
    if let Some(model_size) = metrics.model_size_mb {
        println!("模型大小: {:.2} MB", model_size);
    }
    println!("转录文本长度: {} 字符", transcription.text.len());
    
    // 性能断言
    assert!(metrics.rtf < 1.0, "RTF 应该小于 1.0 (实时处理)");
    assert!(metrics.peak_memory_mb < 1000.0, "峰值内存应该小于 1GB");
    assert!(!transcription.text.trim().is_empty(), "转录文本不应为空");
    
    // 保存性能数据
    tester.save_metrics(&metrics).expect("保存性能数据失败");
    
    // 分析性能趋势
    let _ = tester.analyze_performance_trend("STT_File_Transcription");
}

#[tokio::test]
async fn test_tts_performance_baseline() {
    let tester = PerformanceTester::new();
    let mut metrics = PerformanceMetrics::new("TTS_Text_Synthesis".to_string());
    
    let test_text = "Hello, this is a performance test for text-to-speech synthesis.";
    
    // 创建 TTS 服务
    let config = TtsConfig::default();
    let tts_service = TtsService::new(config);
    
    // 检查 TTS 是否可用
    if !tts_service.is_available().await {
        println!("跳过 TTS 性能测试: Index-TTS 不可用");
        return;
    }
    
    // 启动内存监控
    let mut memory_monitor = MemoryMonitor::new();
    let monitor_handle = {
        let mut monitor = MemoryMonitor::new();
        tokio::spawn(async move {
            monitor.start_monitoring().await;
            monitor
        })
    };
    
    // 执行合成并测量时间
    let start_time = Instant::now();
    let result = tts_service.text_to_speech(test_text).await;
    let processing_time = start_time.elapsed();
    
    // 停止内存监控
    memory_monitor.stop_monitoring();
    let final_monitor = monitor_handle.await.expect("内存监控任务失败");
    
    // 验证合成成功
    assert!(result.is_ok(), "TTS 合成应该成功");
    let audio_data = result.unwrap();
    
    // 记录性能指标
    metrics.processing_time_ms = processing_time.as_millis() as u64;
    metrics.peak_memory_mb = final_monitor.peak_memory();
    metrics.avg_memory_mb = final_monitor.average_memory();
    
    // 添加元数据
    metrics.add_metadata("text_length".to_string(), test_text.len().to_string());
    metrics.add_metadata("audio_size_bytes".to_string(), audio_data.len().to_string());
    
    // 打印结果
    println!("\n=== TTS 性能基线测试结果 ===");
    println!("文本长度: {} 字符", test_text.len());
    println!("处理时间: {} ms", metrics.processing_time_ms);
    println!("峰值内存: {:.2} MB", metrics.peak_memory_mb);
    println!("平均内存: {:.2} MB", metrics.avg_memory_mb);
    println!("音频数据大小: {} 字节", audio_data.len());
    
    // 性能断言
    assert!(metrics.processing_time_ms < 10000, "TTS 处理时间应该小于 10 秒");
    assert!(metrics.peak_memory_mb < 500.0, "峰值内存应该小于 500MB");
    assert!(!audio_data.is_empty(), "音频数据不应为空");
    
    // 保存性能数据
    tester.save_metrics(&metrics).expect("保存性能数据失败");
    
    // 分析性能趋势
    let _ = tester.analyze_performance_trend("TTS_Text_Synthesis");
}

#[tokio::test]
async fn test_audio_processing_performance() {
    let (_, audio_path) = check_test_files();
    let tester = PerformanceTester::new();
    let mut metrics = PerformanceMetrics::new("Audio_Processing".to_string());
    
    // 启动内存监控
    let mut memory_monitor = MemoryMonitor::new();
    let monitor_handle = {
        let mut monitor = MemoryMonitor::new();
        tokio::spawn(async move {
            monitor.start_monitoring().await;
            monitor
        })
    };
    
    // 执行音频处理并测量时间
    let start_time = Instant::now();
    
    // 音频探测
    let probe_result = probe(&audio_path);
    assert!(probe_result.is_ok(), "音频探测应该成功");
    
    // 音频格式转换
    let temp_output = std::env::temp_dir().join("perf_test_converted.wav");
    let convert_result = ensure_whisper_compatible(&audio_path, Some(temp_output.clone()));
    assert!(convert_result.is_ok(), "音频转换应该成功");
    
    let processing_time = start_time.elapsed();
    
    // 停止内存监控
    memory_monitor.stop_monitoring();
    let final_monitor = monitor_handle.await.expect("内存监控任务失败");
    
    // 记录性能指标
    metrics.processing_time_ms = processing_time.as_millis() as u64;
    metrics.peak_memory_mb = final_monitor.peak_memory();
    metrics.avg_memory_mb = final_monitor.average_memory();
    
    // 获取音频信息
    let audio_meta = probe_result.unwrap();
    metrics.audio_duration_ms = audio_meta.duration_ms.unwrap_or(0);
    metrics.calculate_rtf();
    
    // 添加元数据
    metrics.add_metadata("audio_file".to_string(), audio_path.file_name().unwrap().to_string_lossy().to_string());
    metrics.add_metadata("sample_rate".to_string(), audio_meta.sample_rate.to_string());
    metrics.add_metadata("channels".to_string(), audio_meta.channels.to_string());
    
    // 打印结果
    println!("\n=== 音频处理性能测试结果 ===");
    println!("音频时长: {} ms", metrics.audio_duration_ms);
    println!("处理时间: {} ms", metrics.processing_time_ms);
    println!("实时因子 (RTF): {:.3}", metrics.rtf);
    println!("峰值内存: {:.2} MB", metrics.peak_memory_mb);
    println!("平均内存: {:.2} MB", metrics.avg_memory_mb);
    
    // 性能断言
    assert!(metrics.rtf < 0.1, "音频处理 RTF 应该很小");
    assert!(metrics.peak_memory_mb < 100.0, "峰值内存应该小于 100MB");
    
    // 清理临时文件
    let _ = fs::remove_file(&temp_output);
    
    // 保存性能数据
    tester.save_metrics(&metrics).expect("保存性能数据失败");
    
    // 分析性能趋势
    let _ = tester.analyze_performance_trend("Audio_Processing");
}