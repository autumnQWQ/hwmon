# hwmon — 极简 Windows 硬件监控悬浮窗

> 桌面悬浮窗实时显示 CPU、GPU、内存、FPS 数据。Rust 采集 + Electron 渲染。

![screenshot](hwmon.ico)

## 功能

- **桌面悬浮窗** — 透明置顶，鼠标穿透/可拖动
- **CPU** — 频率、利用率、温度
- **GPU** — 频率、利用率、温度、显存占用
- **内存** — 已用/总量、百分比
- **FPS** — 显示器刷新率检测
- **低资源** — Rust 原生采集，~500KB 二进制，无额外后台进程

## 快速开始

### 下载预编译包（推荐）

从 [Releases](https://github.com/autumnQWQ/hwmon/releases) 下载最新 `hwmon-v0.1.0-win64.zip`，解压后：

```bash
# 双击 hwmon.exe 启动悬浮窗（GUI 模式）
# 或命令行运行：
hwmon.exe                # 终端模式，单次采样
hwmon.exe --gui          # 启动桌面悬浮窗
hwmon.exe --watch        # 终端持续监控（1s 间隔）
hwmon.exe --json         # JSON 输出
```

### 从源码构建

需要 Rust 工具链和 Node.js：

```bash
git clone https://github.com/autumnQWQ/hwmon.git
cd hwmon

# 1. 编译 Rust 后端
cargo build --release

# 2. 安装 Electron 依赖
cd hwmon-electron
npm install
cd ..

# 3. 运行
./target/release/hwmon --gui
```

Windows 下也可直接运行 `build.bat` 一键编译。

## 使用

| 命令 | 说明 |
|------|------|
| `hwmon.exe` | 单次采样，终端彩色输出 |
| `hwmon.exe --gui` | 启动桌面悬浮窗 |
| `hwmon.exe --watch` | 终端持续监控 |
| `hwmon.exe --json` | JSON 单次输出 |
| `hwmon.exe -w -j` | JSON 持续流 |
| `hwmon.exe -w -j -i 2000` | 持续监控，2s 间隔 |

### 悬浮窗操作

| 操作 | 说明 |
|------|------|
| 🔒 按钮 | 切换锁定/拖动模式 |
| ✕ 按钮 | 关闭悬浮窗（同时退出终端） |
| 锁定状态 | 鼠标穿透，不干扰前台操作 |
| 解锁状态 | 可拖动悬浮窗到任意位置 |

## 项目结构

```
hwmon/
├── src/               # Rust 源码
│   ├── main.rs        # 入口 + CLI 解析
│   ├── gui.rs         # Electron 启动 + HTTP 服务器
│   ├── cpu.rs         # CPU 采集（WMI/GetSystemTimes）
│   ├── gpu.rs         # GPU 采集（nvidia-smi/DXGI）
│   ├── memory.rs      # 内存采集（GlobalMemoryStatusEx）
│   ├── display.rs     # 终端彩色输出
│   └── types.rs       # 数据结构
├── hwmon-electron/    # Electron 前端
│   ├── main.js        # Electron 主进程
│   ├── index.html     # 悬浮窗 UI
│   └── package.json
├── dist-win/          # 打包输出目录
├── build.bat          # Windows 一键构建
└── install.bat        # 安装脚本
```

## 技术栈

- **后端**: Rust (windows-rs, winreg, serde)
- **前端**: Electron (Node.js)
- **通信**: HTTP localhost (127.0.0.1:18789)
- **平台**: Windows 10/11

## License

MIT
