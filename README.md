# 🔍 File Monitor

[![CI](https://github.com/USERNAME/file_monitor/workflows/🔄%20Continuous%20Integration/badge.svg)](https://github.com/USERNAME/file_monitor/actions)
[![Release](https://github.com/USERNAME/file_monitor/workflows/🚀%20Release/badge.svg)](https://github.com/USERNAME/file_monitor/actions)
[![codecov](https://codecov.io/gh/USERNAME/file_monitor/branch/main/graph/badge.svg)](https://codecov.io/gh/USERNAME/file_monitor)
[![Crates.io](https://img.shields.io/crates/v/file_monitor.svg)](https://crates.io/crates/file_monitor)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

一个高性能、跨平台的文件监控工具，支持实时监控目录中的新文件创建，提供多种并行扫描模式和灵活的配置选项。

## ✨ 功能特性

- 🔍 **智能监控**: 监控指定目录及其子目录中的新文件
- ⚡ **三种扫描模式**: 同步、异步、并行模式，适应不同性能需求
- 🌐 **跨平台支持**: Windows、Linux、macOS 全平台兼容
- 📊 **性能优化**: 网络文件系统性能监控和优化
- 🔧 **灵活配置**: 支持配置文件和命令行参数
- 📝 **专业日志**: 多级别日志系统，便于调试和监控
- 🛡️ **安全可靠**: 只读操作，不会影响监控目录中的文件

## 🚀 快速开始

### 下载安装

从 [Releases](https://github.com/USERNAME/file_monitor/releases) 页面下载适合您平台的二进制文件：

- **Linux**: `file_monitor-linux-x86_64.tar.gz`
- **Windows**: `file_monitor-windows-x86_64.exe.zip`
- **macOS**: `file_monitor-macos-x86_64.tar.gz`

### 基本用法

```bash
# 交互式配置并启动监控
./file_monitor

# 非交互式模式
./file_monitor --non-interactive --monitor-path /path/to/monitor

# 运行一次检查
./file_monitor --once

# 使用指定配置文件
./file_monitor --config my_config.toml
```

## 📋 配置说明

程序首次运行时会自动创建 `config.toml` 配置文件：

```toml
[monitor]
# 监控的根目录路径
root_path = "/path/to/monitor"
# 检查新文件的时间范围（小时）
check_hours = 3
# 扫描间隔（秒）
scan_interval = 60
# 最大扫描深度（可选）
max_depth = 10
# 是否跟随符号链接（可选）
follow_links = true
# 时间戳类型（modified/created）
time_type = "modified"
# 并行模式（sync/async/parallel）
parallel_mode = "sync"
# 最大并行任务数
max_parallel_tasks = 4

[output]
# 有新文件时的提示信息
recording_message = "正在录制"
# 没有新文件时的提示信息
not_recording_message = "未录制"
```

### 配置参数详解

| 参数 | 说明 | 默认值 | 可选值 |
|------|------|--------|--------|
| `root_path` | 监控目录路径 | 用户输入 | 任意有效路径 |
| `check_hours` | 检查时间范围（小时） | 3 | 任意正整数 |
| `scan_interval` | 扫描间隔（秒） | 60 | 任意正整数 |
| `max_depth` | 最大扫描深度 | 无限制 | 任意正整数 |
| `follow_links` | 跟随符号链接 | false | true/false |
| `time_type` | 时间戳类型 | modified | modified/created |
| `parallel_mode` | 并行模式 | sync | sync/async/parallel |
| `max_parallel_tasks` | 最大并行任务数 | CPU核心数 | 任意正整数 |

## 🏃‍♂️ 并行模式对比

| 模式 | 特点 | 适用场景 | 性能 |
|------|------|----------|------|
| **sync** | 同步顺序扫描 | 小型目录、网络文件系统 | 稳定可靠 |
| **async** | 异步并发扫描 | 中型目录、IO密集型 | 中等性能 |
| **parallel** | CPU并行扫描 | 大型目录、本地存储 | 最高性能 |

## 🔧 命令行选项

```bash
file_monitor [OPTIONS]

OPTIONS:
    -c, --config <CONFIG>              配置文件路径 [默认: config.toml]
    -o, --once                         只运行一次，不持续监控
        --non-interactive              非交互式模式
        --monitor-path <MONITOR_PATH>  指定监控目录路径（非交互模式必需）
    -h, --help                         显示帮助信息
    -V, --version                      显示版本信息
```

## 📊 性能优化

### 网络文件系统优化

程序会自动检测网络文件系统的访问延迟：

```bash
# 设置日志级别以查看性能信息
export RUST_LOG=info
./file_monitor
```

### 大规模目录优化

对于包含大量文件的目录，建议：

1. 使用 `parallel` 模式
2. 设置合理的 `max_depth` 限制
3. 调整 `max_parallel_tasks` 以匹配系统性能

## 🛠️ 开发构建

### 环境要求

- Rust 1.70+
- Cargo

### 构建步骤

```bash
# 克隆项目
git clone https://github.com/USERNAME/file_monitor.git
cd file_monitor

# 构建
cargo build --release

# 运行测试
cargo test

# 运行性能测试
chmod +x scripts/performance_test.sh
./scripts/performance_test.sh
```

### 测试覆盖

```bash
# 安装 cargo-llvm-cov
cargo install cargo-llvm-cov

# 生成覆盖率报告
cargo llvm-cov --html
```

## 📦 Docker 使用

```bash
# 拉取镜像
docker pull USERNAME/file_monitor:latest

# 运行容器
docker run -v /path/to/monitor:/monitor \
  USERNAME/file_monitor:latest \
  --non-interactive --monitor-path /monitor --once
```

## 🔍 日志系统

使用环境变量控制日志级别：

```bash
# 详细日志
export RUST_LOG=debug
./file_monitor

# 只显示错误
export RUST_LOG=error
./file_monitor

# 默认日志级别
export RUST_LOG=info
./file_monitor
```

## 🛡️ 安全性

- **只读操作**: 程序只执行读取操作，不会修改、删除或移动文件
- **安全扫描**: 每次发布都经过安全漏洞扫描
- **依赖审计**: 定期审计第三方依赖的安全性

## 🤝 贡献指南

欢迎贡献代码！请遵循以下步骤：

1. Fork 本项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开 Pull Request

### 代码规范

```bash
# 格式化代码
cargo fmt

# 运行 lint 检查
cargo clippy

# 运行所有测试
cargo test
```

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [Rust](https://www.rust-lang.org/) - 系统编程语言
- [Tokio](https://tokio.rs/) - 异步运行时
- [Rayon](https://github.com/rayon-rs/rayon) - 数据并行处理
- [clap](https://clap.rs/) - 命令行参数解析

## 📞 支持

- 🐛 [报告 Bug](https://github.com/USERNAME/file_monitor/issues/new?template=bug_report.md)
- 💡 [功能请求](https://github.com/USERNAME/file_monitor/issues/new?template=feature_request.md)
- 📖 [文档](https://github.com/USERNAME/file_monitor/wiki)
- 💬 [讨论](https://github.com/USERNAME/file_monitor/discussions)

---

<p align="center">
  <img src="https://img.shields.io/badge/Made%20with-❤️-red.svg"/>
  <img src="https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white"/>
</p> 