# 发布指南

本文档描述了 rs-voice-toolkit 项目的发布流程和注意事项。

## 发布顺序

由于项目包含多个相互依赖的 crate，必须按照以下顺序发布到 crates.io：

1. **rs-voice-toolkit-audio** - 音频处理基础库
2. **rs-voice-toolkit-stt** - 语音转文本库（依赖 audio）
3. **rs-voice-toolkit-tts** - 文本转语音库
4. **voice-toolkit** - 主包（依赖所有子包）

## 发布前检查

在发布前，请确保完成以下检查：

### 1. 代码质量检查
```bash
# 运行所有测试
cargo test --workspace --all-features

# 代码格式检查
cargo fmt --check

# Clippy 检查
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 文档生成检查
cargo doc --workspace --all-features --no-deps

# 安全漏洞检查
cargo audit
```

### 2. 版本号更新

确保所有 `Cargo.toml` 文件中的版本号已正确更新：
- `audio/Cargo.toml`
- `stt/Cargo.toml` 
- `tts/Cargo.toml`
- `voice-toolkit/Cargo.toml`

### 3. 依赖版本同步

确保内部依赖的版本号与实际包版本一致。

## 发布命令

### 单独发布每个包

```bash
# 1. 发布 audio 包
cd audio
cargo publish

# 2. 等待 audio 包在 crates.io 上可用后，发布 stt 包
cd ../stt
cargo publish

# 3. 发布 tts 包
cd ../tts
cargo publish

# 4. 最后发布主包
cd ../voice-toolkit
cargo publish
```

### 验证发布

发布后，可以通过以下方式验证：

```bash
# 检查包是否在 crates.io 上可用
cargo search rs-voice-toolkit-audio
cargo search rs-voice-toolkit-stt
cargo search rs-voice-toolkit-tts
cargo search voice-toolkit

# 在新项目中测试安装
cargo new test-project
cd test-project
cargo add voice-toolkit
cargo check
```

## 注意事项

1. **包名冲突**：由于 `audio`、`stt`、`tts` 等通用名称在 crates.io 上已被占用，我们使用了带前缀的包名。

2. **依赖等待**：发布依赖包后，需要等待几分钟让 crates.io 索引更新，然后才能发布依赖它的包。

3. **版本撤回**：如果发现问题需要撤回版本，使用：
   ```bash
   cargo yank --vers <version> <package-name>
   ```

4. **文档更新**：发布后记得更新 README.md 中的安装说明和版本号。

## 发布检查清单

- [ ] 所有测试通过
- [ ] 代码格式化完成
- [ ] Clippy 检查通过
- [ ] 文档生成正常
- [ ] 安全审计通过
- [ ] 版本号已更新
- [ ] 依赖版本已同步
- [ ] 按顺序发布所有包
- [ ] 验证发布成功
- [ ] 更新文档和 README