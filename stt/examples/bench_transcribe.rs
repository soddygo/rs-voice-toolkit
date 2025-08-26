use std::path::PathBuf;
use std::time::Instant;
use log::info;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        log::error!(
            "用法: cargo run -p stt --example bench_transcribe -- <model_path> <audio_path> [iters]"
        );
        std::process::exit(1);
    }

    let model = PathBuf::from(&args[1]);
    let audio = PathBuf::from(&args[2]);
    let iters: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(3);

    let model_path = model.display();
    info!("模型: {model_path}");
    let audio_path = audio.display();
    info!("音频: {audio_path}");
    info!("迭代次数: {iters}");

    // 预热一次，避免首次加载影响
    let _ = rs_voice_toolkit_stt::transcribe_file(&model, &audio).await;

    let mut total_rt_factor = 0.0f64;
    let mut total_ms = 0u128;

    for i in 0..iters {
        let t0 = Instant::now();
        let result = rs_voice_toolkit_stt::transcribe_file(&model, &audio)
            .await
            .expect("转录失败");
        let dt = t0.elapsed().as_millis();
        let rtf = result.real_time_factor();
        total_rt_factor += rtf;
        total_ms += dt;
        info!(
            "迭代 {i}: RTF={rtf:.3}, 用时={dt} ms, 文本='{}'",
            result.text
        );
    }

    let avg_rtf = total_rt_factor / iters as f64;
    let avg_ms = total_ms as f64 / iters as f64;
    info!("平均 RTF={avg_rtf:.3}, 平均用时={avg_ms:.0} ms");
}
