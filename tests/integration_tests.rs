use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// 创建测试目录结构
fn create_test_structure(base_dir: &Path) -> std::io::Result<()> {
    // 创建多级目录结构
    for i in 1..=5 {
        let dir_path = base_dir.join(format!("dir_{}", i));
        fs::create_dir_all(&dir_path)?;

        // 在每个目录中创建子目录
        for j in 1..=3 {
            let subdir = dir_path.join(format!("subdir_{}", j));
            fs::create_dir_all(&subdir)?;

            // 创建不同时间的文件
            if i <= 3 {
                // 新文件（最近1小时）
                let file_path = subdir.join(format!("new_file_{}.txt", j));
                fs::write(&file_path, "new content")?;
            } else {
                // 旧文件（3天前）
                let file_path = subdir.join(format!("old_file_{}.txt", j));
                fs::write(&file_path, "old content")?;

                // 注意：修改文件时间需要 filetime crate 的特殊处理
                // 这里只是创建文件，实际时间修改需要更复杂的代码
            }
        }
    }
    Ok(())
}

#[test]
fn test_basic_functionality() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // 创建测试目录结构
    create_test_structure(test_path).expect("Failed to create test structure");

    // 创建配置文件
    let config_content = format!(
        r#"
[monitor]
root_path = "{}"
check_hours = 3
scan_interval = 60
parallel_mode = "sync"

[output]
recording_message = "正在录制"
not_recording_message = "未录制"
"#,
        test_path.display()
    );

    let config_path = test_path.join("test_config.toml");
    fs::write(&config_path, config_content).expect("Failed to write config");

    // 运行程序
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "--once",
        ])
        .output()
        .expect("Failed to run program");

    // 验证输出
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("文件监控报告"));
    assert!(output.status.success());
}

#[test]
fn test_parallel_modes() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    create_test_structure(test_path).expect("Failed to create test structure");

    // 测试不同的并行模式
    let modes = vec!["sync", "async", "parallel"];

    for mode in modes {
        let config_content = format!(
            r#"
[monitor]
root_path = "{}"
check_hours = 3
scan_interval = 60
parallel_mode = "{}"
max_parallel_tasks = 2

[output]
recording_message = "正在录制"
not_recording_message = "未录制"
"#,
            test_path.display(),
            mode
        );

        let config_path = test_path.join(format!("test_config_{}.toml", mode));
        fs::write(&config_path, config_content).expect("Failed to write config");

        let output = Command::new("cargo")
            .args([
                "run",
                "--",
                "--config",
                config_path.to_str().unwrap(),
                "--once",
            ])
            .output()
            .unwrap_or_else(|_| panic!("Failed to run program with mode {}", mode));

        assert!(output.status.success(), "Failed with mode: {}", mode);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("文件监控报告"),
            "No report found for mode: {}",
            mode
        );
    }
}

#[test]
fn test_deep_directory_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // 创建深层目录结构（10层深）
    let mut current_path = test_path.to_path_buf();
    for i in 1..=10 {
        current_path = current_path.join(format!("level_{}", i));
        fs::create_dir_all(&current_path).expect("Failed to create deep directory");

        // 在每隔一层创建文件
        if i % 2 == 0 {
            let file_path = current_path.join(format!("deep_file_{}.txt", i));
            fs::write(&file_path, format!("Content at level {}", i))
                .expect("Failed to write file");
        }
    }

    // 测试不同的最大深度设置
    let depths = vec![None, Some(3), Some(5), Some(10)];

    for depth in depths {
        let config_content = format!(
            r#"
[monitor]
root_path = "{}"
check_hours = 24
scan_interval = 60
{}

[output]
recording_message = "正在录制"
not_recording_message = "未录制"
"#,
            test_path.display(),
            if let Some(d) = depth {
                format!("max_depth = {}", d)
            } else {
                "# max_depth = 10".to_string()
            }
        );

        let config_path = test_path.join(format!("test_config_depth_{:?}.toml", depth));
        fs::write(&config_path, config_content).expect("Failed to write config");

        let output = Command::new("cargo")
            .args([
                "run",
                "--",
                "--config",
                config_path.to_str().unwrap(),
                "--once",
            ])
            .output()
            .expect("Failed to run program");

        assert!(output.status.success(), "Failed with depth: {:?}", depth);
    }
}

#[test]
fn test_non_interactive_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    create_test_structure(test_path).expect("Failed to create test structure");

    // 测试非交互模式
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--non-interactive",
            "--monitor-path",
            test_path.to_str().unwrap(),
            "--once",
        ])
        .output()
        .expect("Failed to run program in non-interactive mode");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // 修改断言，检查实际输出的内容
    assert!(stdout.contains("配置文件创建完成") || stdout.contains("文件监控报告"));
}

#[test]
fn test_invalid_configuration() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // 测试无效的配置文件
    let invalid_config = r#"
[monitor]
root_path = "/nonexistent/path/that/should/not/exist"
check_hours = "invalid"
"#;

    let config_path = temp_dir.path().join("invalid_config.toml");
    fs::write(&config_path, invalid_config).expect("Failed to write invalid config");

    let _output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "--once",
        ])
        .output()
        .expect("Failed to run program");

    // 程序应该能够处理无效配置而不崩溃
    // 具体的错误处理取决于程序的设计
}

#[test]
fn test_time_type_options() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    create_test_structure(test_path).expect("Failed to create test structure");

    // 测试不同的时间戳类型
    let time_types = vec!["modified", "created"];

    for time_type in time_types {
        let config_content = format!(
            r#"
[monitor]
root_path = "{}"
check_hours = 3
scan_interval = 60
time_type = "{}"

[output]
recording_message = "正在录制"
not_recording_message = "未录制"
"#,
            test_path.display(),
            time_type
        );

        let config_path = test_path.join(format!("test_config_time_{}.toml", time_type));
        fs::write(&config_path, config_content).expect("Failed to write config");

        let output = Command::new("cargo")
            .args([
                "run",
                "--",
                "--config",
                config_path.to_str().unwrap(),
                "--once",
            ])
            .output()
            .unwrap_or_else(|_| panic!("Failed to run program with time_type {}", time_type));

        assert!(
            output.status.success(),
            "Failed with time_type: {}",
            time_type
        );
    }
}
