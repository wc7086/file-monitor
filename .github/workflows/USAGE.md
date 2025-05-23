# 🚀 GitHub Actions 使用指南

这个项目包含了完整的 CI/CD 流水线配置，提供自动化测试、构建和发布功能。

## 📁 工作流文件说明

### 🔄 持续集成 (`.github/workflows/ci.yml`)

**触发条件：**
- 推送到 `main` 或 `develop` 分支
- 提交 Pull Request 到 `main` 分支

**包含的作业：**

#### 🧪 测试套件 (`test`)
- **平台矩阵**: Ubuntu, Windows, macOS
- **Rust 版本**: stable, beta, nightly
- **检查项目**:
  - 代码格式检查 (`cargo fmt`)
  - Clippy lint 检查 (`cargo clippy`)
  - 单元测试 (`cargo test`)
  - 文档生成测试

#### 🏗️ 集成测试 (`integration-tests`)
- 跨平台集成测试
- CLI 功能测试
- 并行模式压力测试

#### 🚀 性能测试 (`performance-tests`)
- 仅在推送到 `main` 分支时运行
- 大规模文件监控性能基准
- 上传性能结果为 artifact

#### 🛡️ 安全审计 (`security-audit`)
- 依赖漏洞扫描 (`cargo audit`)
- 许可证合规检查 (`cargo deny`)

#### 🌐 跨平台测试 (`cross-platform`)
- 多目标平台编译测试
- 支持的目标:
  - Linux: x86_64-gnu, x86_64-musl, aarch64-gnu
  - Windows: x86_64-msvc, aarch64-msvc
  - macOS: x86_64-darwin, aarch64-darwin

#### 📝 代码覆盖率 (`coverage`)
- 生成代码覆盖率报告
- 自动上传到 codecov.io

### 🚀 发布流程 (`.github/workflows/release.yml`)

**触发条件：**
- 创建 GitHub Release
- 推送标签 (格式: `v*`)

**包含的作业：**

#### 🏗️ 二进制构建 (`build-binaries`)
- **构建目标**: 7个平台的优化二进制文件
- **特性**: 
  - 自动压缩 (Linux/macOS: strip)
  - 跨编译支持
  - ARM 架构支持

#### 📦 发布包创建 (`create-release-assets`)
- 创建完整的发布包
- 包含文档、示例配置、许可证
- 生成 SHA256 校验和

#### 🚀 GitHub 发布 (`github-release`)
- 自动创建 GitHub Release
- 上传所有平台的二进制文件
- 生成详细的发布说明

#### 📦 Crates.io 发布 (`crates-io-release`)
- 自动发布到 Rust 包管理器
- 仅正式版本 (非 alpha/beta/rc)

#### 🐳 Docker 发布 (`docker-release`)
- 构建轻量级 Alpine Docker 镜像
- 推送到 Docker Hub

## 🔧 配置要求

### GitHub Secrets

为了完整使用所有功能，需要在 GitHub 仓库设置中配置以下 secrets：

```bash
# Crates.io 发布 (可选)
CRATES_IO_TOKEN=your_crates_token

# Docker Hub 发布 (可选)
DOCKER_USERNAME=your_docker_username
DOCKER_PASSWORD=your_docker_password_or_token
```

### 分支保护规则

建议为 `main` 分支设置以下保护规则：

1. **要求 PR 审查**: 至少 1 个审查者
2. **要求状态检查**: 
   - `CI Success` (ci.yml 的总体状态)
   - `Security Audit`
   - `Cross Platform Tests`
3. **要求分支是最新的**: 启用
4. **限制推送**: 仅允许通过 PR

## 🎯 使用场景

### 📝 开发工作流

1. **功能开发**:
   ```bash
   git checkout -b feature/new-feature
   # 开发代码...
   git push origin feature/new-feature
   # 创建 PR -> 自动触发 CI 测试
   ```

2. **代码审查**:
   - CI 会自动运行所有测试
   - 查看测试结果和覆盖率报告
   - 确认安全扫描通过

3. **合并到主分支**:
   ```bash
   # PR 通过审查后合并
   # 自动触发主分支的完整 CI，包括性能测试
   ```

### 🚀 发布流程

1. **准备发布**:
   ```bash
   # 更新版本号
   vim Cargo.toml  # 修改 version = "1.0.0"
   
   # 更新变更日志
   vim CHANGELOG.md
   
   # 提交更改
   git add .
   git commit -m "chore: bump version to 1.0.0"
   git push origin main
   ```

2. **创建发布**:
   ```bash
   # 创建标签
   git tag -a v1.0.0 -m "Release version 1.0.0"
   git push origin v1.0.0
   
   # 或在 GitHub 网页创建 Release
   ```

3. **自动发布**:
   - GitHub Actions 自动构建所有平台
   - 创建 GitHub Release
   - 发布到 Crates.io 和 Docker Hub

## 🔍 监控和调试

### 查看构建日志

1. 转到 GitHub 仓库的 **Actions** 标签
2. 选择相应的工作流运行
3. 查看各个作业的详细日志

### 常见问题排查

#### ❌ 测试失败
```bash
# 本地运行相同的测试
cargo test --verbose

# 检查特定平台问题
cargo test --target x86_64-pc-windows-msvc
```

#### ❌ 跨编译失败
```bash
# 本地测试跨编译
rustup target add aarch64-unknown-linux-gnu
cargo build --target aarch64-unknown-linux-gnu
```

#### ❌ 发布失败
- 检查 Secrets 配置
- 验证标签格式 (必须以 `v` 开头)
- 确认版本号在 Cargo.toml 中已更新

### 性能监控

- 性能测试结果会作为 artifact 保存
- 可以下载查看具体的性能数据
- 对比不同版本的性能变化

## 📊 Badge 状态

在 README.md 中添加状态徽章：

```markdown
[![CI](https://github.com/USERNAME/file_monitor/workflows/🔄%20Continuous%20Integration/badge.svg)](https://github.com/USERNAME/file_monitor/actions)
[![Release](https://github.com/USERNAME/file_monitor/workflows/🚀%20Release/badge.svg)](https://github.com/USERNAME/file_monitor/actions)
[![codecov](https://codecov.io/gh/USERNAME/file_monitor/branch/main/graph/badge.svg)](https://codecov.io/gh/USERNAME/file_monitor)
```

## 🛠️ 自定义配置

### 修改测试矩阵

编辑 `.github/workflows/ci.yml`:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
    rust: [stable, beta]  # 移除 nightly 减少测试时间
```

### 添加新的构建目标

编辑 `.github/workflows/release.yml`:

```yaml
matrix:
  include:
    # 添加新目标
    - target: riscv64gc-unknown-linux-gnu
      os: ubuntu-latest
      artifact_name: file_monitor
      asset_name: file_monitor-linux-riscv64
```

### 自定义性能测试

修改 `scripts/performance_test.sh` 以适应项目需求。

## 🔄 最佳实践

1. **保持依赖更新**: 定期运行 `cargo update`
2. **安全第一**: 始终关注安全扫描结果
3. **性能监控**: 关注性能测试趋势
4. **文档同步**: 确保文档与代码同步更新
5. **版本管理**: 遵循语义化版本规范

## 📦 缓存优化策略

### ⚡ 多层缓存架构

项目采用细粒度缓存策略，显著提升 CI 构建速度：

#### 🦀 Rust 工具链缓存
```yaml
- name: 📦 Cache Rust toolchain
  uses: actions/cache@v4
  with:
    path: |
      ~/.rustup/toolchains
      ~/.rustup/update-hashes
      ~/.rustup/settings.toml
    key: ${{ runner.os }}-rustup-${{ matrix.rust }}-${{ hashFiles('rust-toolchain.toml') }}
```

#### 📚 Cargo 注册表缓存
```yaml
- name: 📦 Cache Cargo registry and index
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry/index/
      ~/.cargo/registry/cache/
      ~/.cargo/git/db/
    key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
```

#### 🎯 目标目录缓存
```yaml
- name: 📦 Cache Cargo target directory
  uses: actions/cache@v4
  with:
    path: target/
    key: ${{ runner.os }}-cargo-target-${{ matrix.rust }}-${{ hashFiles('**/Cargo.lock') }}-${{ hashFiles('**/*.rs') }}
```

#### 🔧 二进制工具缓存
```yaml
- name: 📦 Cache Cargo binary directory
  uses: actions/cache@v4
  with:
    path: ~/.cargo/bin/
    key: ${{ runner.os }}-cargo-bin-${{ matrix.rust }}
```

### 📊 缓存性能指标

- **首次构建**: ~8-15分钟
- **缓存命中构建**: ~2-5分钟
- **缓存节省**: 60-80% 构建时间
- **存储优化**: 分层缓存避免冗余存储

### 🔄 缓存策略详解

#### 🎯 分层缓存原理
1. **工具链缓存**: 避免重复下载 Rust 编译器
2. **注册表缓存**: 跳过依赖索引下载
3. **目标缓存**: 复用编译产物
4. **工具缓存**: 保存 cargo-audit 等工具

#### 🚀 缓存键设计
- **精确匹配**: `${{ hashFiles('**/Cargo.lock') }}` - 依赖精确匹配
- **源码匹配**: `${{ hashFiles('**/*.rs') }}` - 源码变更检测
- **平台区分**: `${{ runner.os }}-${{ matrix.target }}` - 多平台隔离
- **回退策略**: 多级 `restore-keys` 提供渐进回退

---

**注意**: 请根据实际项目需求调整配置，并确保所有必要的 secrets 和权限配置正确。 