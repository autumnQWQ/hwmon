use crate::types::MemoryInfo;

#[cfg(windows)]
mod win {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    use std::process::Command;

    fn cmd_output(program: &str, args: &[&str]) -> String {
        Command::new(program).args(args).output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default()
    }

    /// Primary: GlobalMemoryStatusEx Win32 API
    fn mem_api() -> Option<(f32, f32)> {
        unsafe {
            let mut mem = MEMORYSTATUSEX {
                dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
                ..Default::default()
            };
            if GlobalMemoryStatusEx(&mut mem).is_ok() && mem.ullTotalPhys > 0 {
                let total = mem.ullTotalPhys as f32 / 1073741824.0;
                let avail = mem.ullAvailPhys as f32 / 1073741824.0;
                return Some((total - avail, total));
            }
        }
        None
    }

    /// Fallback: wmic OS memory
    fn mem_wmic() -> Option<(f32, f32)> {
        let out = cmd_output("wmic", &["OS", "get", "TotalVisibleMemorySize,FreePhysicalMemory", "/format:csv", "/noheading"]);
        for line in out.lines() {
            let t = line.trim();
            if !t.is_empty() && t.contains(',') && t.matches(',').count() >= 2 {
                let parts: Vec<&str> = t.split(',').collect();
                let total_kb: f64 = parts.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(0.0);
                let free_kb: f64 = parts.get(2).and_then(|s| s.trim().parse().ok()).unwrap_or(0.0);
                if total_kb > 0.0 {
                    let total_gb = total_kb as f32 / 1048576.0;
                    let free_gb = free_kb as f32 / 1048576.0;
                    return Some((total_gb - free_gb, total_gb));
                }
            }
        }
        None
    }

    pub fn collect_memory() -> super::MemoryInfo {
        // Try API first, then wmic fallback
        if let Some((used, total)) = mem_api().or_else(mem_wmic) {
            let pct = if total > 0.0 { (used / total) * 100.0 } else { 0.0 };
            return super::MemoryInfo { used_gb: used, total_gb: total, used_pct: pct };
        }
        super::MemoryInfo { used_gb: 0.0, total_gb: 0.0, used_pct: 0.0 }
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use std::process::Command;

    pub fn collect_memory() -> super::MemoryInfo {
        let total: u64 = Command::new("sysctl").args(["-n","hw.memsize"]).output().ok()
            .and_then(|o| String::from_utf8(o.stdout).ok()).unwrap_or_default().trim().parse().unwrap_or(0);
        let out = Command::new("vm_stat").output().ok()
            .and_then(|o| String::from_utf8(o.stdout).ok()).unwrap_or_default();
        let page_size: u64 = 16384;
        let mut active: u64 = 0; let mut wired: u64 = 0; let mut compressed: u64 = 0;
        for line in out.lines() {
            if line.contains("Pages active:") { active = parse_val(line); }
            else if line.contains("Pages wired down:") { wired = parse_val(line); }
            else if line.contains("Pages occupied by compressor:") { compressed = parse_val(line); }
        }
        let used_bytes = (wired + active + compressed) * page_size;
        let total_gb = total as f32 / 1073741824.0;
        let used_gb = used_bytes as f32 / 1073741824.0;
        super::MemoryInfo { used_gb, total_gb, used_pct: if total>0 { (used_gb/total_gb)*100.0 } else { 0.0 } }
    }
    fn parse_val(line: &str) -> u64 { line.rsplit(':').next().unwrap_or("0").trim().trim_end_matches('.').parse().unwrap_or(0) }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod fallback {
    pub fn collect_memory() -> super::MemoryInfo {
        super::MemoryInfo { used_gb: 0.0, total_gb: 0.0, used_pct: 0.0 }
    }
}

#[cfg(windows)] use win as imp;
#[cfg(target_os = "macos")] use mac as imp;
#[cfg(not(any(windows, target_os = "macos")))] use fallback as imp;

pub fn collect() -> MemoryInfo { imp::collect_memory() }
