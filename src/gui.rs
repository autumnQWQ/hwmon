// Electron-based GUI overlay.
// Rust collects hardware data (WMI) → HTTP server → Electron renders UI.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)] use std::os::windows::process::CommandExt;

static LOCKED: AtomicBool = AtomicBool::new(true);

fn start_http_server(latest: Arc<Mutex<Option<String>>>) -> u16 {
    const API_PORT: u16 = 18789;
    #[cfg(windows)] {
        for _ in 0..3 {
            if std::net::TcpListener::bind(("127.0.0.1", API_PORT)).is_ok() { break; }
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command",
                    &format!("$p=(Get-NetTCPConnection -LocalPort {} -ErrorAction SilentlyContinue).OwningProcess; if($p){{Stop-Process -Id $p -Force -ErrorAction SilentlyContinue; Start-Sleep 1}}", API_PORT)])
                .output();
        }
    }
    let listener = std::net::TcpListener::bind(("127.0.0.1", API_PORT))
        .expect("Cannot bind port 18789");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        use std::io::{Read, Write};
        for stream in listener.incoming() {
            if let Ok(mut s) = stream {
                let mut buf = [0u8; 4096]; let _ = s.read(&mut buf);
                let body = latest.lock().ok().and_then(|c| c.clone()).unwrap_or_default();
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(), body
                ); let _ = s.write_all(resp.as_bytes());
            }
        }
    });
    port
}

fn collect_json() -> String {
    let cpu = crate::cpu::collect(); let gpu = crate::gpu::collect(); let mem = crate::memory::collect(); let fps = crate::fps();
    let cpu_temp = cpu.temperature_c.filter(|&t| t > 30.0).or_else(|| gpu.temperature_c.map(|t| (t+5.0).max(40.0).min(100.0)));
    serde_json::json!({
        "locked": LOCKED.load(Ordering::Relaxed),
        "cpu": {"freq": cpu.frequency_mhz, "util": cpu.utilization_pct, "temp": cpu_temp},
        "gpu": {"freq": gpu.frequency_mhz, "util": gpu.utilization_pct, "temp": gpu.temperature_c,
            "mem_used": (gpu.memory_used_mb as f64 / 1024.0 * 10.0).round() / 10.0,
            "mem_total": (gpu.memory_total_mb as f64 / 1024.0 * 10.0).round() / 10.0},
        "memory": {"used": mem.used_gb, "total": mem.total_gb, "pct": mem.used_pct},
        "fps": fps,
    }).to_string()
}

fn find_app_dir() -> Option<std::path::PathBuf> {
    let p = std::path::PathBuf::from("F:\\hwmon\\hwmon-electron");
    if p.join("main.js").exists() { return Some(p); }
    None
}

#[cfg(windows)]
fn launch_electron() -> Option<std::process::Child> {
    let app_dir = find_app_dir()?;
    let exe = app_dir.join("node_modules").join("electron").join("dist").join("electron.exe");
    if !exe.exists() { return None; }

    let dist = app_dir.join("node_modules").join("electron").join("dist");
    std::process::Command::new(exe)
        .current_dir(&dist)
        .creation_flags(0x08000000)
        .arg("--no-sandbox")
        .arg(app_dir.join("main.js"))
        .spawn()
        .ok()
}

#[cfg(not(windows))]
fn launch_electron() -> Option<std::process::Child> { None }

pub fn run_overlay() {
    #[cfg(windows)] {
        extern "system" { fn GetConsoleWindow() -> isize; fn ShowWindow(a: isize, b: i32) -> bool; }
        unsafe { let cw = GetConsoleWindow(); if cw != 0 { let _ = ShowWindow(cw, 0); } }
    }
    let cache = Arc::new(Mutex::new(None::<String>));
    let c2 = cache.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let data = collect_json();
        if let Ok(mut c) = c2.lock() { *c = Some(data); }
    });
    let _port = start_http_server(cache);
    if let Some(mut child) = launch_electron() {
        let _ = child.wait();
    } else {
        eprintln!("hwmon: Electron binary not found");
        loop { std::thread::sleep(std::time::Duration::from_secs(1)); }
    }
}
