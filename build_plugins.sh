# PowerShell脚本，用于编译和部署插件

# 创建插件目录
$pluginDir = "target/plugins"
if (-not (Test-Path $pluginDir)) {
    New-Item -ItemType Directory -Path $pluginDir -Force
}

# 编译插件
Write-Host "编译插件..." -ForegroundColor Green
cargo build --package plugin_a --release
cargo build --package plugin_b --release
cargo build --package plugin_c --release

# 复制插件到插件目录
Write-Host "部署插件..." -ForegroundColor Green

# 根据操作系统确定文件扩展名
if ($IsWindows -or $env:OS -match "Windows") {
    $ext = "dll"
} elseif ($IsMacOS) {
    $ext = "dylib"
} else {
    $ext = "so"
}

# 复制插件文件
Copy-Item "target/release/plugin_a.$ext" -Destination "$pluginDir/plugin_a.$ext" -Force
Copy-Item "target/release/plugin_b.$ext" -Destination "$pluginDir/plugin_b.$ext" -Force
Copy-Item "target/release/plugin_c.$ext" -Destination "$pluginDir/plugin_c.$ext" -Force

Write-Host "插件已部署到 $pluginDir" -ForegroundColor Green
Write-Host "现在可以运行 'cargo run' 来测试插件系统" -ForegroundColor Cyan