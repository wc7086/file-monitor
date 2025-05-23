#!/bin/bash

# 性能测试脚本
set -e

echo "🚀 开始性能基准测试..."

# 创建大型测试目录结构
TEST_DIR="./test_performance"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"

echo "📁 创建大型测试目录结构..."

# 创建1000个子目录，每个包含100个文件
for i in {1..1000}; do
    DIR="$TEST_DIR/dir_$i"
    mkdir -p "$DIR"
    for j in {1..100}; do
        echo "test content $i-$j" > "$DIR/file_$j.txt"
    done
    if [ $((i % 100)) -eq 0 ]; then
        echo "  已创建 $i 个目录..."
    fi
done

echo "✅ 测试目录创建完成：1000个目录，100,000个文件"

# 创建性能测试配置
cat > "$TEST_DIR/perf_config.toml" << EOF
[monitor]
root_path = "$TEST_DIR"
check_hours = 24
scan_interval = 60
parallel_mode = "sync"

[output]
recording_message = "正在录制"
not_recording_message = "未录制"
EOF

# 测试不同模式的性能
modes=("sync" "async" "parallel")

echo "⏱️  开始性能测试..."

for mode in "${modes[@]}"; do
    echo "测试模式: $mode"
    
    # 更新配置文件
    sed -i "s/parallel_mode = \".*\"/parallel_mode = \"$mode\"/" "$TEST_DIR/perf_config.toml"
    
    # 运行性能测试（5次取平均值）
    total_time=0
    for run in {1..5}; do
        start_time=$(date +%s%N)
        
        timeout 60s cargo run --release -- --config "$TEST_DIR/perf_config.toml" --once > /dev/null 2>&1 || true
        
        end_time=$(date +%s%N)
        duration=$(( (end_time - start_time) / 1000000 )) # 转换为毫秒
        total_time=$((total_time + duration))
        
        echo "  运行 $run: ${duration}ms"
    done
    
    avg_time=$((total_time / 5))
    echo "  平均时间: ${avg_time}ms"
    echo "  📊 $mode 模式性能: ${avg_time}ms" >> performance_results.txt
    echo ""
done

echo "🎯 性能测试完成，结果保存到 performance_results.txt"

# 清理测试目录
rm -rf "$TEST_DIR"

echo "✨ 性能测试结束" 