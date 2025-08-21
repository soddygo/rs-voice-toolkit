# 发布指南

本文档说明如何将 rs-voice-toolkit 发布到 crates.io。

## 发布前检查清单

### 1. 代码质量检查

```bash
# 运行所有测试
cargo test --workspace

# 运行带 streaming feature 的测试
cargo test --workspace --features streaming

# 检查代码格式
cargo fmt --check

# 运行 clippy 检查
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 检查文档
cargo doc --workspace --all-features --no-deps
```

### 2. 版本管理

所有包使用统一版本号，在 `Cargo.toml` 的 `[workspace.package]` 中定义：

```toml
[workspace.package]
version = "0.1.0"  # 更新此版本号
```

### 3. 文档更新

- [ ] 更新 `README.md` 中的版本信息和示例
- [ ] 检查所有文档链接是否有效
- [ ] 确保 API 文档完整且准确
- [ ] 更新 `CHANGELOG.md`（如果存在）

### 4. 依赖检查

```bash
# 检查依赖是否为最新稳定版本
cargo update --dry-run

# 检查安全漏洞
cargo audit
```

## 发布顺序

由于包之间的依赖关系，必须按以下顺序发布：

1. `audio` - 基础音频处理库
2. `stt` - 语音转文本库（依赖 audio）
3. `tts` - 文本转语音库
4. `voice-toolkit` - 主包（依赖所有子包）

## 发布步骤

### 1. 准备发布

```bash
# 确保工作目录干净
git status

# 创建发布分支
git checkout -b release/v0.1.0

# 更新版本号（在 workspace Cargo.toml 中）
# 提交版本更新
git add .
git commit -m "chore: bump version to 0.1.0"
```

### 2. 发布 audio 包

```bash
cd audio

# 检查包内容
cargo package --list

# 本地测试构建
cargo package

# 发布到 crates.io
cargo publish

cd ..
```

### 3. 发布 stt 包

```bash
cd stt

# 等待 audio 包在 crates.io 上可用（通常需要几分钟）
# 检查包内容
cargo package --list

# 本地测试构建
cargo package

# 发布到 crates.io
cargo publish

cd ..
```

### 4. 发布 tts 包

```bash
cd tts

# 检查包内容
cargo package --list

# 本地测试构建
cargo package

# 发布到 crates.io
cargo publish

cd ..
```

### 5. 发布主包 voice-toolkit

```bash
cd voice-toolkit

# 等待所有依赖包在 crates.io 上可用
# 检查包内容
cargo package --list

# 本地测试构建
cargo package

# 发布到 crates.io
cargo publish

cd ..
```

### 6. 创建 Git 标签

```bash
# 创建并推送标签
git tag v0.1.0
git push origin v0.1.0

# 合并发布分支
git checkout main
git merge release/v0.1.0
git push origin main
```

## 发布后验证

### 1. 验证包可用性

```bash
# 创建临时项目测试
mkdir test-release
cd test-release
cargo init

# 添加依赖
cargo add voice-toolkit

# 测试构建
cargo build
```

### 2. 检查文档

- 访问 https://docs.rs/voice-toolkit 确认文档正确生成
- 检查 https://crates.io/crates/voice-toolkit 页面信息

## 常见问题

### 1. 发布失败

**错误**: `error: failed to publish to registry`

**解决方案**:
- 检查网络连接
- 确认 crates.io API token 有效
- 检查包名是否已被占用
- 确认版本号未曾发布过

### 2. 依赖解析失败

**错误**: `error: failed to select a version for the requirement`

**解决方案**:
- 等待依赖包在 crates.io 上完全可用
- 检查版本约束是否正确
- 使用 `cargo update` 更新依赖

### 3. 文档生成失败

**错误**: `error: could not document`

**解决方案**:
- 检查文档注释语法
- 确保所有 feature 组合都能编译
- 修复 rustdoc 警告

## 自动化发布

可以考虑使用 GitHub Actions 自动化发布流程：

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y ffmpeg
      
      - name: Run tests
        run: cargo test --workspace --all-features
      
      - name: Publish to crates.io
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          # 按顺序发布各个包
          cd audio && cargo publish
          sleep 30  # 等待包可用
          cd ../stt && cargo publish
          sleep 30
          cd ../tts && cargo publish
          sleep 30
          cd ../voice-toolkit && cargo publish
```

## 版本策略

遵循 [Semantic Versioning](https://semver.org/)：

- **MAJOR** (1.0.0): 不兼容的 API 变更
- **MINOR** (0.1.0): 向后兼容的功能添加
- **PATCH** (0.0.1): 向后兼容的问题修复

在 1.0.0 之前，API 可能会有破坏性变更。

## 许可证说明

本项目使用双许可证 MIT OR Apache-2.0，这是 Rust 生态系统的标准做法。

### 第三方依赖许可证

- **whisper-rs**: MIT
- **ez-ffmpeg**: MIT
- **hound**: Apache-2.0
- **rubato**: MIT
- **tokio**: MIT

所有依赖都与项目许可证兼容。

### Whisper 模型许可证

Whisper 模型由 OpenAI 发布，使用 MIT 许可证。用户需要自行下载模型文件。

## 支持和维护

- **问题报告**: GitHub Issues
- **功能请求**: GitHub Discussions
- **安全问题**: 通过私有渠道报告
- **文档**: docs.rs 和 GitHub Wiki

## 相关资源

- [Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [crates.io 政策](https://crates.io/policies)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Semantic Versioning](https://semver.org/)