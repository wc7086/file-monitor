@echo off
echo 文件监控程序 - Windows版本
echo.

REM 检查是否存在编译好的程序
if exist "target\release\file_monitor.exe" (
    echo 使用Release版本运行...
    target\release\file_monitor.exe %*
) else if exist "target\debug\file_monitor.exe" (
    echo 使用Debug版本运行...
    target\debug\file_monitor.exe %*
) else (
    echo 未找到编译好的程序，正在编译...
    cargo build --release
    if exist "target\release\file_monitor.exe" (
        echo 编译完成，正在运行...
        target\release\file_monitor.exe %*
    ) else (
        echo 编译失败！
        pause
        exit /b 1
    )
)

pause 