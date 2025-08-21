# 音频模块合并方案

## 概述
将 `stt::audio` 模块功能合并到独立的 `audio` 库中，确保向后兼容性和功能完整性。

## 当前模块状态

### stt::audio 模块功能
- ✅ AudioFormat 枚举（格式检测）
- ✅ AudioConfig 结构（音频配置）
- ✅ AudioConverter（格式转换，基于 ez-ffmpeg）
- ✅ AudioResampler（重采样，基于 rubato）
- ✅ AudioData 结构（音频数据封装）
- ✅ 高级功能：VAD、静音检测、批量处理
- ✅ 工具函数：文件读写、格式转换、音量处理

### 独立 audio 模块功能
- ✅ AudioError 枚举（错误处理）
- ✅ AudioMeta 结构（元数据）
- ✅ probe() 函数（音频探测）
- ✅ ensure_whisper_compatible() 函数（格式转换）
- ✅ resample() 函数（重采样）
- ✅ StreamingResampler（流式重采样）

## 合并策略

### 1. 保留独立 audio 模块为核心
- 保持现有 API 不变：`probe()`, `ensure_whisper_compatible()`, `resample()`
- 将 `stt::audio` 的高级功能作为可选特性

### 2. 功能整合
- 将 `stt::audio::AudioFormat` 合并到独立模块
- 将 `stt::audio::AudioConfig` 合并到独立模块  
- 将 `stt::audio::AudioData` 作为高级功能
- 将 `stt::audio` 的工具函数作为公共 API

### 3. 特性标志设计
```toml
[features]
default = ["basic"]
basic = []  # 基础功能：探测、格式转换、重采样
advanced = ["audio-processing"]  # 高级功能：VAD、静音检测、批量处理
```

### 4. 向后兼容性
- 保持所有现有公共 API 不变
- `stt` 模块继续重新导出 `audio` 模块功能
- 逐步弃用重复功能

## 实施步骤

### 第一阶段：基础功能合并
1. [ ] 将 `stt::audio::AudioFormat` 移到独立 `audio` 模块
2. [ ] 将 `stt::audio::AudioConfig` 移到独立 `audio` 模块
3. [ ] 统一错误类型定义
4. [ ] 更新 `stt` 模块的引用

### 第二阶段：高级功能整合
1. [ ] 将 `AudioData` 结构移到独立模块（高级特性）
2. [ ] 将工具函数移到独立模块（高级特性）
3. [ ] 配置特性标志依赖关系

### 第三阶段：清理和测试
1. [ ] 移除 `stt::audio` 模块
2. [ ] 更新所有测试用例
3. [ ] 验证向后兼容性

## API 变化

### 合并后的音频模块 API
```rust
// 基础功能（始终可用）
pub fn probe(path: impl AsRef<Path>) -> Result<AudioMeta, AudioError>
pub fn ensure_whisper_compatible(input: impl AsRef<Path>, output: Option<PathBuf>) -> Result<CompatibleWav, AudioError>
pub fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Result<Resampled, AudioError>

// 高级功能（需要 "advanced" 特性）
#[cfg(feature = "advanced")]
pub struct AudioData { /* ... */ }

#[cfg(feature = "advanced")] 
pub fn read_wav_file(path: impl AsRef<Path>) -> Result<AudioData, AudioError>

#[cfg(feature = "advanced")]
pub fn detect_silence(samples: &[f32], threshold: f32, min_duration_ms: u32, sample_rate: u32) -> Vec<(usize, usize)>
```

## 依赖关系调整

### 当前依赖
```
stt → stt::audio → ez-ffmpeg, rubato, hound
audio → ez-ffmpeg, rubato, hound
```

### 合并后依赖  
```
stt → audio → ez-ffmpeg, rubato, hound
```

## 测试计划

1. **单元测试**: 确保所有现有功能正常工作
2. **集成测试**: 验证 `stt` 模块与新的 `audio` 模块集成
3. **特性测试**: 验证特性标志的正确性
4. **性能测试**: 确保性能没有回归

## 风险评估

- **中风险**: 模块合并可能引入编译错误
- **低风险**: API 变化影响现有用户
- **缓解措施**: 逐步实施，充分测试，保持向后兼容

## 时间估算

- 第一阶段：2-3 天
- 第二阶段：1-2 天  
- 第三阶段：1 天
- 总计：4-6 天