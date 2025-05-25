use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local};
use clap::Parser;
use futures::future::join_all;
use log::{debug, error, info, warn};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;
use tokio::task;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "file_monitor")]
#[command(about = "监控目录中的新文件创建")]
struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// 只运行一次，不持续监控
    #[arg(short, long)]
    once: bool,

    /// 非交互式模式，自动使用默认配置
    #[arg(long)]
    non_interactive: bool,

    /// 指定监控目录路径（用于非交互式模式）
    #[arg(long)]
    monitor_path: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Config {
    monitor: MonitorConfig,
    output: OutputConfig,
}

#[derive(Deserialize, Debug)]
struct MonitorConfig {
    root_path: String,
    check_hours: u64,
    scan_interval: u64,
    max_depth: Option<usize>,
    follow_links: Option<bool>,
    time_type: Option<String>,
    parallel_mode: Option<String>,
    max_parallel_tasks: Option<usize>,
    search_latest_subdir_only: Option<bool>,
    // 性能优化选项（不影响精确度）
    use_async_io: Option<bool>,
    batch_size: Option<usize>,
}

#[derive(Deserialize, Debug)]
struct OutputConfig {
    recording_message: String,
    not_recording_message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Args::parse();

    // 加载或创建配置文件
    let mut config = load_or_create_config(&args.config, &args)?;

    // 检查并更新监控路径
    let original_path = config.monitor.root_path.clone();
    config.monitor.root_path = ensure_valid_monitor_path(&config.monitor.root_path, &args)?;

    // 只在路径实际改变时才保存配置文件
    if config.monitor.root_path != original_path {
        info!("监控路径已更新，保存配置文件...");
        save_config_safely(&args.config, &config)?;
    }

    info!("文件监控程序启动");
    info!("监控目录: {}", config.monitor.root_path);
    info!("检查时间范围: {} 小时", config.monitor.check_hours);

    if args.once {
        // 只运行一次
        check_and_report(&config).await?;
    } else {
        // 持续监控
        info!("扫描间隔: {} 秒", config.monitor.scan_interval);
        info!("按 Ctrl+C 停止监控");

        loop {
            clear_screen();
            info!("文件监控中... (按 Ctrl+C 停止)");
            info!("监控目录: {}", config.monitor.root_path);
            info!("检查时间范围: {} 小时", config.monitor.check_hours);
            check_and_report(&config).await?;
            tokio::time::sleep(tokio::time::Duration::from_secs(
                config.monitor.scan_interval,
            ))
            .await;
        }
    }

    Ok(())
}

fn load_or_create_config(config_path: &str, args: &Args) -> Result<Config> {
    // 检查配置文件是否存在
    if !Path::new(config_path).exists() {
        if args.non_interactive {
            // 非交互式模式
            if let Some(monitor_path) = args.monitor_path.as_deref() {
                println!("[配置] 非交互式模式: 创建配置文件 {}", config_path);
                println!("使用监控目录: {}", monitor_path);
                create_default_config_safely(config_path, monitor_path)?;
            } else {
                return Err(anyhow::anyhow!(
                    "非交互式模式下必须使用 --monitor-path 参数指定监控目录路径\n\
                     示例: {} --non-interactive --monitor-path /path/to/monitor",
                    std::env::args()
                        .next()
                        .unwrap_or_else(|| "file_monitor".to_string())
                ));
            }
        } else {
            // 交互式模式
            println!("[配置] 配置文件不存在，开始创建配置文件: {}", config_path);
            println!();

            // 获取用户输入的监控目录
            let monitor_path = get_monitor_path_from_user()?;

            create_default_config_safely(config_path, &monitor_path)?;
        }

        println!("[配置] 配置文件创建完成: {}", config_path);
        if !args.non_interactive {
            println!("您可以随时编辑配置文件来修改设置");
        }
        println!();
    }

    let content = fs::read_to_string(config_path)
        .with_context(|| format!("无法读取配置文件: {}", config_path))?;

    let config: Config = toml::from_str(&content).with_context(|| "配置文件格式错误")?;

    Ok(config)
}

fn ensure_valid_monitor_path(current_path: &str, args: &Args) -> Result<String> {
    let path = Path::new(current_path);

    if path.exists() && path.is_dir() {
        return Ok(current_path.to_string());
    }

    if args.non_interactive {
        println!("[警告] 监控目录不存在: {}", current_path);
        if let Some(new_path) = &args.monitor_path {
            println!("[配置] 使用命令行指定的路径: {}", new_path);
            return Ok(new_path.clone());
        } else {
            println!("[错误] 非交互式模式下无法修复路径问题");
            return Ok(current_path.to_string());
        }
    }

    println!("[错误] 监控目录不存在或无效: {}", current_path);
    println!("请输入一个有效的监控目录路径");
    println!();

    get_monitor_path_from_user()
}

fn get_monitor_path_from_user() -> Result<String> {
    println!("请设置文件监控参数：");
    println!();

    // 显示示例
    println!("监控目录路径示例：");
    if cfg!(target_os = "windows") {
        println!("  Windows绝对路径: D:\\录制文件\\录制输出");
        println!("  Windows相对路径: 录制输出");
        println!("  UNC网络路径: \\\\服务器\\共享\\录制输出");
    } else {
        println!("  绝对路径: /home/user/recordings");
        println!("  相对路径: recordings");
        println!("  网络路径: /mnt/shared/recordings");
    }
    println!();

    loop {
        print!("请输入要监控的目录路径: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            println!("[错误] 路径不能为空，请重新输入");
            continue;
        }

        // 验证路径格式（基本检查）
        if input.contains('\0') {
            println!("[错误] 路径包含无效字符，请重新输入");
            continue;
        }

        // 检查路径是否存在
        let path = Path::new(input);
        if !path.exists() {
            println!("[警告] 目录 '{}' 不存在", input);
            print!("是否仍要使用此路径？程序将在运行时检查目录 (y/N): ");
            io::stdout().flush()?;

            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;
            let confirm = confirm.trim().to_lowercase();

            if confirm != "y" && confirm != "yes" {
                continue;
            }
        } else if !path.is_dir() {
            println!("[错误] '{}' 不是一个目录，请输入目录路径", input);
            continue;
        } else {
            println!("[成功] 目录验证成功: {}", input);
        }

        return Ok(input.to_string());
    }
}

fn save_config_safely(config_path: &str, config: &Config) -> Result<()> {
    let escaped_path = config.monitor.root_path.replace("\\", "\\\\");

    let config_content = format!(
        r#"# 文件监控配置
[monitor]
# 监控的根目录路径
root_path = "{}"
# 检查新文件的时间范围（小时）
check_hours = {}
# 扫描间隔（秒）
scan_interval = {}
# 最大扫描深度（可选）
# 注释掉或删除此行表示无限制深度
# 设置具体数值可限制扫描深度，例如: max_depth = 10
{}
# 是否跟随符号链接（可选，默认false）
# 启用此选项会跟随符号链接到其目标位置
{}
# 时间戳类型（可选，默认modified）
# modified: 使用文件修改时间（跨平台兼容性更好）
# created: 使用文件创建时间（Windows上更准确，但Linux可能不支持）
{}
# 并行模式（可选，默认sync）
# sync: 同步模式，等待所有任务完成
# async: 异步模式，不等待任务完成
# parallel: 并行模式，同时执行多个任务
{}
# 最大并行任务数（可选，默认CPU核心数）
# 设置具体数值可限制并行任务数，例如: max_parallel_tasks = 4
{}
# 是否只搜索最新子目录（可选，默认false）
# 启用此选项可大大节省扫描时间，但只会检查最新修改的子目录
{}
# 性能优化选项（不影响精确度）
use_async_io = {}
batch_size = {}

[output]
# 有新文件时的提示信息
recording_message = "{}"
# 没有新文件时的提示信息
not_recording_message = "{}"
"#,
        escaped_path,
        config.monitor.check_hours,
        config.monitor.scan_interval,
        if let Some(depth) = config.monitor.max_depth {
            format!("max_depth = {}", depth)
        } else {
            "# max_depth = 10".to_string()
        },
        if let Some(follow) = config.monitor.follow_links {
            format!("follow_links = {}", follow)
        } else {
            "# follow_links = true".to_string()
        },
        if let Some(time_type) = &config.monitor.time_type {
            format!("time_type = \"{}\"", time_type)
        } else {
            "# time_type = \"created\"".to_string()
        },
        if let Some(parallel_mode) = &config.monitor.parallel_mode {
            format!("parallel_mode = \"{}\"", parallel_mode)
        } else {
            "# parallel_mode = \"sync\"".to_string()
        },
        if let Some(max_parallel_tasks) = config.monitor.max_parallel_tasks {
            format!("max_parallel_tasks = {}", max_parallel_tasks)
        } else {
            "# max_parallel_tasks = 4".to_string()
        },
        if let Some(search_latest) = config.monitor.search_latest_subdir_only {
            format!("search_latest_subdir_only = {}", search_latest)
        } else {
            "# search_latest_subdir_only = true".to_string()
        },
        if let Some(use_async_io) = config.monitor.use_async_io {
            format!("use_async_io = {}", use_async_io)
        } else {
            "# use_async_io = false".to_string()
        },
        if let Some(batch_size) = config.monitor.batch_size {
            format!("batch_size = {}", batch_size)
        } else {
            "# batch_size = 1000".to_string()
        },
        config.output.recording_message,
        config.output.not_recording_message
    );

    // 原子性写入：先写入临时文件，然后重命名
    let temp_path = format!("{}.tmp", config_path);
    let backup_path = format!("{}.backup", config_path);

    // 如果配置文件已存在，先创建备份
    if Path::new(config_path).exists() {
        fs::copy(config_path, &backup_path)
            .with_context(|| format!("无法创建配置文件备份: {}", backup_path))?;
    }

    // 写入临时文件
    fs::write(&temp_path, &config_content)
        .with_context(|| format!("无法写入临时配置文件: {}", temp_path))?;

    // 验证临时文件内容
    let verification_content = fs::read_to_string(&temp_path)
        .with_context(|| format!("无法验证临时配置文件: {}", temp_path))?;

    if verification_content != config_content {
        // 清理临时文件
        let _ = fs::remove_file(&temp_path);
        return Err(anyhow::anyhow!("配置文件写入验证失败"));
    }

    // 原子性重命名
    fs::rename(&temp_path, config_path)
        .with_context(|| format!("无法完成配置文件更新: {}", config_path))?;

    println!("[配置] 配置文件已安全保存，备份文件: {}", backup_path);
    Ok(())
}

fn create_default_config_safely(config_path: &str, monitor_path: &str) -> Result<()> {
    // 转义Windows路径中的反斜杠
    let escaped_path = monitor_path.replace("\\", "\\\\");

    let default_config = format!(
        r#"# 文件监控配置
[monitor]
# 监控的根目录路径
root_path = "{}"
# 检查新文件的时间范围（小时）
check_hours = 3
# 扫描间隔（秒）
scan_interval = 60
# 最大扫描深度（可选）
# 注释掉或删除此行表示无限制深度
# 设置具体数值可限制扫描深度，例如: max_depth = 10
# max_depth = 10
# 是否跟随符号链接（可选，默认false）
# 启用此选项会跟随符号链接到其目标位置
# follow_links = true
# 时间戳类型（可选，默认modified）
# modified: 使用文件修改时间（跨平台兼容性更好）
# created: 使用文件创建时间（Windows上更准确，但Linux可能不支持）
# time_type = "created"
# 并行模式（可选，默认sync）
# sync: 同步模式，等待所有任务完成
# async: 异步模式，不等待任务完成
# parallel: 并行模式，同时执行多个任务
# parallel_mode = "sync"
# 最大并行任务数（可选，默认CPU核心数）
# 设置具体数值可限制并行任务数，例如: max_parallel_tasks = 4
# max_parallel_tasks = 4
# 是否只搜索最新子目录（可选，默认false）
# 启用此选项可大大节省扫描时间，但只会检查最新修改的子目录
# search_latest_subdir_only = true
# 性能优化选项（不影响精确度）
use_async_io = false
batch_size = 1000

[output]
# 有新文件时的提示信息
recording_message = "正在录制"
# 没有新文件时的提示信息
not_recording_message = "未录制"
"#,
        escaped_path
    );

    // 检查文件是否已存在，避免意外覆盖
    if Path::new(config_path).exists() {
        return Err(anyhow::anyhow!("配置文件已存在，拒绝覆盖: {}", config_path));
    }

    // 原子性写入
    let temp_path = format!("{}.tmp", config_path);

    fs::write(&temp_path, &default_config)
        .with_context(|| format!("无法创建临时配置文件: {}", temp_path))?;

    // 验证文件内容
    let verification_content = fs::read_to_string(&temp_path)
        .with_context(|| format!("无法验证临时配置文件: {}", temp_path))?;

    if verification_content != default_config {
        // 清理临时文件
        let _ = fs::remove_file(&temp_path);
        return Err(anyhow::anyhow!("配置文件创建验证失败"));
    }

    // 原子性重命名
    fs::rename(&temp_path, config_path)
        .with_context(|| format!("无法完成配置文件创建: {}", config_path))?;

    Ok(())
}

fn clear_screen() {
    // 跨平台清屏
    if cfg!(target_os = "windows") {
        // Windows 清屏
        let _ = std::process::Command::new("cmd")
            .args(["/c", "cls"])
            .status();
    } else {
        // Unix/Linux/Mac 清屏
        print!("\x1B[2J\x1B[1;1H");
    }
}

async fn check_and_report(config: &Config) -> Result<()> {
    let root_path = Path::new(&config.monitor.root_path);

    if !root_path.exists() {
        error!("监控目录不存在: {}", config.monitor.root_path);
        error!("请检查配置文件中的 root_path 设置");
        // 仍然输出报告，显示空结果
        let empty_status = HashMap::new();
        print_status_report(&empty_status, config);
        return Ok(());
    }

    // 网络文件系统性能验证
    let start_time = Instant::now();
    if let Err(e) = fs::read_dir(root_path) {
        error!("无法读取监控目录: {}", e);
        // 仍然输出报告，显示空结果
        let empty_status = HashMap::new();
        print_status_report(&empty_status, config);
        return Ok(());
    }
    let read_duration = start_time.elapsed();

    // 如果目录读取超过1秒，可能是网络文件系统延迟问题
    if read_duration.as_secs() > 1 {
        warn!(
            "目录读取耗时 {:.2}秒，可能存在网络延迟或挂载问题",
            read_duration.as_secs_f64()
        );
    } else {
        debug!("目录读取耗时 {:.2}毫秒", read_duration.as_millis());
    }

    // 计算时间阈值
    let threshold_time = Local::now() - Duration::hours(config.monitor.check_hours as i64);

    // 获取所有二级目录及其新文件状态
    let subdirs_status = check_subdirectories_async(root_path, threshold_time, config).await?;

    // 总是输出结果
    print_status_report(&subdirs_status, config);

    Ok(())
}

async fn check_subdirectories_async(
    root_path: &Path,
    threshold_time: DateTime<Local>,
    config: &Config,
) -> Result<HashMap<String, bool>> {
    let mut status_map = HashMap::new();

    // 确定并行模式
    let parallel_mode = config.monitor.parallel_mode.as_deref().unwrap_or("sync");

    let max_tasks = config
        .monitor
        .max_parallel_tasks
        .unwrap_or_else(num_cpus::get);

    debug!("使用并行模式: {}, 最大任务数: {}", parallel_mode, max_tasks);

    // 收集所有子目录
    let mut directories = Vec::new();
    if let Ok(entries) = fs::read_dir(root_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    directories.push((dir_name.to_string(), path));
                }
            }
        }
    }

    let scan_start = Instant::now();

    match parallel_mode {
        "async" => {
            // 异步并发模式
            info!("使用异步并发模式扫描 {} 个目录", directories.len());
            let tasks: Vec<_> = directories
                .into_iter()
                .map(|(dir_name, path)| {
                    let max_depth = config.monitor.max_depth;
                    let follow_links = config.monitor.follow_links;
                    let time_type = config.monitor.time_type.clone();
                    let search_latest_subdir_only = config.monitor.search_latest_subdir_only;
                    let use_async_io = config.monitor.use_async_io;

                    task::spawn(async move {
                        let has_recent = task::spawn_blocking(move || {
                            has_recent_files_optimized(
                                &path,
                                threshold_time,
                                max_depth,
                                follow_links,
                                &time_type,
                                search_latest_subdir_only,
                                use_async_io.unwrap_or(false),
                            )
                        })
                        .await
                        .unwrap_or(Ok(false))
                        .unwrap_or(false);
                        (dir_name, has_recent)
                    })
                })
                .collect();

            let results = join_all(tasks).await;
            for (dir_name, has_recent) in results.into_iter().flatten() {
                status_map.insert(dir_name, has_recent);
            }
        }
        "parallel" => {
            // CPU 并行模式
            info!("使用 CPU 并行模式扫描 {} 个目录", directories.len());
            let results: Vec<_> = directories
                .par_iter()
                .map(|(dir_name, path)| {
                    let has_recent = has_recent_files_optimized(
                        path,
                        threshold_time,
                        config.monitor.max_depth,
                        config.monitor.follow_links,
                        &config.monitor.time_type,
                        config.monitor.search_latest_subdir_only,
                        config.monitor.use_async_io.unwrap_or(false),
                    )
                    .unwrap_or(false);
                    (dir_name.clone(), has_recent)
                })
                .collect();

            for (dir_name, has_recent) in results {
                status_map.insert(dir_name, has_recent);
            }
        }
        _ => {
            // 同步模式（默认）
            debug!("使用同步模式扫描 {} 个目录", directories.len());
            for (dir_name, path) in directories {
                let has_recent_files = has_recent_files_optimized(
                    &path,
                    threshold_time,
                    config.monitor.max_depth,
                    config.monitor.follow_links,
                    &config.monitor.time_type,
                    config.monitor.search_latest_subdir_only,
                    config.monitor.use_async_io.unwrap_or(false),
                )?;
                status_map.insert(dir_name, has_recent_files);
            }
        }
    }

    let scan_duration = scan_start.elapsed();
    info!(
        "目录扫描完成，耗时 {:.2}ms（模式: {}）",
        scan_duration.as_millis(),
        parallel_mode
    );

    Ok(status_map)
}

fn has_recent_files_optimized(
    dir_path: &Path,
    threshold_time: DateTime<Local>,
    max_depth: Option<usize>,
    follow_links: Option<bool>,
    time_type: &Option<String>,
    search_latest_subdir_only: Option<bool>,
    use_async_io: bool,
) -> Result<bool> {
    // 确定使用的时间戳类型，默认为modified
    let use_modified = time_type
        .as_ref()
        .map(|t| t.to_lowercase())
        .as_deref()
        .unwrap_or("modified")
        == "modified";

    // 如果启用了只搜索最新子目录的选项
    if search_latest_subdir_only.unwrap_or(false) {
        return search_in_latest_subdir_only_optimized(
            dir_path,
            threshold_time,
            max_depth,
            follow_links,
            use_modified,
        );
    }

    // 激进优化3: 使用异步I/O
    if use_async_io {
        return has_recent_files_async_io(
            dir_path,
            threshold_time,
            max_depth,
            follow_links,
            use_modified,
        );
    }

    // 回退到原有逻辑
    let mut walker = WalkDir::new(dir_path);

    if let Some(depth) = max_depth {
        walker = walker.max_depth(depth);
    }

    if let Some(follow) = follow_links {
        walker = walker.follow_links(follow);
    }

    for entry in walker.into_iter().flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Ok(metadata) = fs::metadata(path) {
                let time_result = if use_modified {
                    metadata.modified()
                } else {
                    metadata.created()
                };

                match time_result {
                    Ok(time) => {
                        let file_time: DateTime<Local> = time.into();
                        if file_time > threshold_time {
                            return Ok(true);
                        }
                    }
                    Err(e) => {
                        let time_name = if use_modified {
                            "修改时间"
                        } else {
                            "创建时间"
                        };
                        warn!("无法获取文件{} '{}': {}", time_name, path.display(), e);
                    }
                }
            } else {
                warn!("无法获取文件元数据 '{}'", path.display());
            }
        }
    }
    Ok(false)
}

// 激进优化2: 优化版的最新子目录搜索
fn search_in_latest_subdir_only_optimized(
    dir_path: &Path,
    threshold_time: DateTime<Local>,
    max_depth: Option<usize>,
    follow_links: Option<bool>,
    use_modified: bool,
) -> Result<bool> {
    // 首先快速检查当前目录时间
    if let Ok(metadata) = fs::metadata(dir_path) {
        let time_result = if use_modified {
            metadata.modified()
        } else {
            metadata.created()
        };

        if let Ok(time) = time_result {
            let dir_time: DateTime<Local> = time.into();
            if dir_time > threshold_time {
                debug!("目录本身就是新的: {}", dir_path.display());
                return Ok(true);
            }
        }
    }

    // 找到最新的子目录
    let latest_subdir = find_latest_subdir(dir_path, use_modified)?;
    
    if let Some(latest_dir) = latest_subdir {
        debug!("搜索最新子目录: {}", latest_dir.display());
        
        // 否则进行更详细的搜索
        let mut walker = WalkDir::new(&latest_dir);
        
        // 如果设置了最大深度，需要减1（因为我们已经进入了一层子目录）
        if let Some(depth) = max_depth {
            if depth > 1 {
                walker = walker.max_depth(depth - 1);
            } else {
                walker = walker.max_depth(1);
            }
        }

        // 如果设置了跟随符号链接，则应用设置
        if let Some(follow) = follow_links {
            walker = walker.follow_links(follow);
        }

        for entry in walker.into_iter().flatten() {
            let path = entry.path();
            if path.is_file() {
                // 使用DirEntry的metadata而不是fs::metadata，更快
                if let Ok(metadata) = entry.metadata() {
                    let time_result = if use_modified {
                        metadata.modified()
                    } else {
                        metadata.created()
                    };

                    if let Ok(time) = time_result {
                        let file_time: DateTime<Local> = time.into();
                        if file_time > threshold_time {
                            debug!("在最新子目录中找到新文件: {}", path.display());
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }

    Ok(false)
}

// 优化的异步I/O版本（不影响精确度）
fn has_recent_files_async_io(
    dir_path: &Path,
    threshold_time: DateTime<Local>,
    max_depth: Option<usize>,
    follow_links: Option<bool>,
    use_modified: bool,
) -> Result<bool> {
    // 批量处理文件，减少系统调用
    let mut files_to_check = Vec::new();
    let mut walker = WalkDir::new(dir_path);

    if let Some(depth) = max_depth {
        walker = walker.max_depth(depth);
    }

    if let Some(follow) = follow_links {
        walker = walker.follow_links(follow);
    }

    // 批量收集文件路径
    for entry in walker.into_iter().flatten() {
        if entry.path().is_file() {
            files_to_check.push(entry);
        }
    }

    // 检查所有文件（不影响精确度）
    check_files_batch(&files_to_check, threshold_time, use_modified)
}

fn check_files_batch(
    files: &[walkdir::DirEntry],
    threshold_time: DateTime<Local>,
    use_modified: bool,
) -> Result<bool> {
    for entry in files {
        // 使用DirEntry的metadata方法，避免额外的系统调用
        if let Ok(metadata) = entry.metadata() {
            let time_result = if use_modified {
                metadata.modified()
            } else {
                metadata.created()
            };

            if let Ok(time) = time_result {
                let file_time: DateTime<Local> = time.into();
                if file_time > threshold_time {
                    debug!("批量检查找到新文件: {}", entry.path().display());
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn find_latest_subdir(dir_path: &Path, use_modified: bool) -> Result<Option<std::path::PathBuf>> {
    let mut latest_dir: Option<std::path::PathBuf> = None;
    let mut latest_time: Option<DateTime<Local>> = None;

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(metadata) = fs::metadata(&path) {
                    let time_result = if use_modified {
                        metadata.modified()
                    } else {
                        metadata.created()
                    };

                    if let Ok(time) = time_result {
                        let dir_time: DateTime<Local> = time.into();
                        
                        if latest_time.is_none() || dir_time > latest_time.unwrap() {
                            latest_time = Some(dir_time);
                            latest_dir = Some(path);
                        }
                    } else {
                        let time_name = if use_modified {
                            "修改时间"
                        } else {
                            "创建时间"
                        };
                        warn!("无法获取目录{} '{}': {}", time_name, path.display(), time_result.unwrap_err());
                    }
                } else {
                    warn!("无法获取目录元数据 '{}'", path.display());
                }
            }
        }
    }

    Ok(latest_dir)
}

fn print_status_report(status_map: &HashMap<String, bool>, config: &Config) {
    let current_time = Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("\n=== [报告] 文件监控报告 [{}] ===", current_time);

    if status_map.is_empty() {
        println!("[警告] 未找到任何二级目录");
        return;
    }

    // 按目录名排序输出
    let mut sorted_dirs: Vec<_> = status_map.iter().collect();
    sorted_dirs.sort_by_key(|&(name, _)| name);

    for (dir_name, &has_recent_files) in sorted_dirs {
        let (status, icon) = if has_recent_files {
            (&config.output.recording_message, "[REC]")
        } else {
            (&config.output.not_recording_message, "[---]")
        };

        println!("{} 目录 '{}': {}", icon, dir_name, status);
    }
    println!("=======================================\n");
}
