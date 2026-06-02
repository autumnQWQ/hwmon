mod types;
mod cpu;
mod gpu;
mod memory;
mod display;
mod gui;

use types::SystemInfo;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    // Single instance via named mutex — std::process::Command variant for port binding
    // Just check if our port is already in use
    #[cfg(windows)]
    {
        use std::net::TcpStream;
        if TcpStream::connect("127.0.0.1:18789").is_ok() {
            // Another hwmon is already running
            return;
        }
    }

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("hwmon v{}", VERSION);
        return;
    }
    let gui = args.iter().any(|a| a == "--gui" || a == "-g");
    let watch = args.iter().any(|a| a == "--watch" || a == "-w");
    let json = args.iter().any(|a| a == "--json" || a == "-j");
    let interval_ms = parse_interval(&args);

    // Windows: default to GUI overlay when no flags given
    if gui || (cfg!(windows) && !watch && !json) {
        gui::run_overlay();
        return;
    }

    if watch {
        run_watch(json, interval_ms);
    } else {
        run_once(json);
    }
}

fn print_help() {
    println!("hwmon v{} — 极简 Windows 硬件监控工具", VERSION);
    println!();
    println!("用法:");
    println!("  hwmon                  单次采样，终端彩色输出");
    println!("  hwmon --gui, -g        透明悬浮窗 (桌面置顶)");
    println!("  hwmon --watch, -w      终端持续监控 (1s 间隔)");
    println!("  hwmon --json, -j       单次 JSON 输出");
    println!("  hwmon -w -j            持续 JSON 流输出");
    println!("  hwmon -w -j -i 2000    持续监控，每 2s 采样");
    println!();
    println!("选项:");
    println!("  -g, --gui              启动桌面悬浮窗 (macOS: 透明置顶+鼠标穿透)");
    println!("  -w, --watch            终端持续监控模式");
    println!("  -j, --json             JSON 输出格式");
    println!("  -i, --interval <ms>    采样间隔毫秒数 (默认: 1000)");
    println!("  -h, --help             显示此帮助");
    println!("  -V, --version          显示版本");
    println!();
    println!("监控项:");
    println!("  CPU: 型号 | 核心数 | 频率 (MHz) | 利用率 (%) | 温度 (°C)");
    println!("  GPU: 型号 | 厂商 | 频率 (MHz) | 利用率 (%) | 温度 (°C) | 显存");
    println!();
    println!("平台支持:");
    println!("  Windows: WMI + NVAPI/ADL (完整数据)");
    println!("  macOS:   sysctl + ioreg (开发/调试)");
    println!("  Linux:   开发中");
}

fn parse_interval(args: &[String]) -> u64 {
    for i in 0..args.len() {
        if (args[i] == "--interval" || args[i] == "-i") && i + 1 < args.len() {
            return args[i + 1].parse().unwrap_or(1000);
        }
    }
    1000
}

fn collect() -> SystemInfo {
    SystemInfo {
        cpu: cpu::collect(),
        gpu: gpu::collect(),
        memory: memory::collect(),
        fps: fps(),
        timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
    }
}

pub fn fps_display() -> String {
    let f = fps();
    if f > 0 { format!("{}", f) } else { "--".into() }
}

pub(crate) fn fps() -> u32 {
    use std::sync::OnceLock;
    static FPS: OnceLock<u32> = OnceLock::new();
    *FPS.get_or_init(|| {
        #[cfg(windows)] {
            use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC, GetDeviceCaps, VREFRESH};
            unsafe {
                let hdc = GetDC(None);
                let v = GetDeviceCaps(hdc, VREFRESH) as u32;
                ReleaseDC(None, hdc);
                if v > 0 && v < 500 { return v; }
            }
            0
        }
        #[cfg(target_os = "macos")] {
            if let Ok(out) = std::process::Command::new("system_profiler")
                .args(["SPDisplaysDataType","-detailLevel","mini"]).output()
                .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            {
                for line in out.lines() {
                    if line.contains("Resolution:") && line.contains("Hz") {
                        if let Some(hz) = line.rsplit(|c: char| c.is_whitespace()).nth(1) {
                            if let Ok(v) = hz.trim().parse::<u32>() { return v; }
                        }
                    }
                }
            }
            0
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        { 0 }
    })
}

fn run_once(json: bool) {
    let info = collect();
    if json {
        println!("{}", serde_json::to_string_pretty(&info).unwrap());
    } else {
        println!("{}", display::block_color(&info));
    }
    // On Windows, if launched by double-click, pause so user can see output
    #[cfg(windows)]
    wait_for_enter();
}

#[cfg(windows)]
fn wait_for_enter() {
    use std::io::{self, BufRead};
    println!("\nPress Enter to exit...");
    let _ = io::stdin().lock().lines().next();
}

fn run_watch(json: bool, interval_ms: u64) {
    loop {
        let info = collect();
        if json {
            println!("{}", serde_json::to_string(&info).unwrap());
        } else {
            print!("\x1b[2J\x1b[H");
            println!("{}", display::block_color(&info));
            println!("\n  Ctrl+C to exit  |  interval: {}ms", interval_ms);
        }
        std::thread::sleep(std::time::Duration::from_millis(interval_ms));
    }
}
