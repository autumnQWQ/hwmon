use crate::types::GpuInfo;

#[cfg(windows)]
mod win {
    use winreg::enums::*;
    use winreg::RegKey;
    use std::process::Command;

    fn cmd_output(program: &str, args: &[&str]) -> String {
        Command::new(program).args(args).output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default()
    }

    fn wmic_val(class: &str, field: &str) -> String {
        let out = cmd_output("wmic", &[class, "get", field, "/format:csv", "/noheading"]);
        for line in out.lines() {
            let t = line.trim();
            if !t.is_empty() && t.contains(',') {
                return t.split(',').nth(1).unwrap_or("").trim().to_string();
            }
        }
        String::new()
    }

    // ─── nvidia-smi (NVIDIA primary) ──────────────────────

    fn try_nvidia_smi() -> Option<(f32, u64, u64, f32, u64)> {
        // usage%, mem_used_mb, mem_total_mb, temp_c, clock_mhz
        let out = cmd_output("nvidia-smi",
            &["--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu,clocks.current.graphics",
              "--format=csv,noheader,nounits"]);
        for line in out.lines() {
            let t = line.trim();
            if !t.is_empty() && t.contains(',') {
                let parts: Vec<f32> = t.split(',').filter_map(|s| s.trim().parse().ok()).collect();
                if parts.len() >= 5 {
                    return Some((parts[0], parts[1] as u64, parts[2] as u64, parts[3], parts[4] as u64));
                }
                if parts.len() >= 4 {
                    return Some((parts[0], parts[1] as u64, parts[2] as u64, parts[3], 0));
                }
                if parts.len() >= 3 {
                    return Some((parts[0], parts[1] as u64, parts[2] as u64, 0.0, 0));
                }
            }
        }
        None
    }

    fn has_nvidia_smi() -> bool {
        Command::new("nvidia-smi").arg("--version").output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    // ─── DXGI enumeration ─────────────────────────────────

    #[derive(Debug, PartialEq)]
    enum GpuType { Integrated, Discrete, Unknown }

    fn enumerate_dxgi_gpus() -> Vec<(String, u64, u64, GpuType, u32)> {
        use windows::Win32::Graphics::Dxgi::*;
        use windows::core::Interface;

        let mut gpus = Vec::new();
        unsafe {
            if let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() {
                for i in 0..8u32 {
                    if let Ok(adapter) = factory.EnumAdapters1(i) {
                        if let Ok(desc) = adapter.GetDesc1() {
                            let name = String::from_utf16_lossy(&desc.Description)
                                .trim_end_matches('\0').to_string();
                            let mut total_vram = desc.DedicatedVideoMemory as u64;
                            if total_vram == 0 { total_vram = desc.SharedSystemMemory as u64; }
                            let vendor = desc.VendorId;
                            let gpu_type = classify_gpu(vendor, total_vram);

                            let mut used_vram: u64 = 0;
                            if let Ok(adapter3) = adapter.cast::<IDXGIAdapter3>() {
                                let mut info = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
                                if adapter3.QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &mut info).is_ok() {
                                    used_vram = info.CurrentUsage as u64;
                                }
                            }
                            gpus.push((name, total_vram, used_vram, gpu_type, vendor));
                        }
                    } else { break; }
                }
            }
        }
        gpus
    }

    fn classify_gpu(vendor: u32, vram: u64) -> GpuType {
        match vendor {
            0x8086 => GpuType::Integrated,
            0x10DE => GpuType::Discrete,
            0x1002 if vram > 0 => GpuType::Discrete,
            0x1002 => GpuType::Integrated,
            _ if vram == 0 => GpuType::Integrated,
            _ => GpuType::Discrete,
        }
    }

    fn select_gpu(gpus: &[(String, u64, u64, GpuType, u32)]) -> Option<usize> {
        for (i, (_, v, _, t, _)) in gpus.iter().enumerate() {
            if *t == GpuType::Discrete && *v > 0 { return Some(i); }
        }
        for (i, (_, _, _, t, _)) in gpus.iter().enumerate() {
            if *t == GpuType::Discrete { return Some(i); }
        }
        if !gpus.is_empty() { Some(0) } else { None }
    }

    fn reg_vram(sub: &RegKey) -> Option<u64> {
        if let Ok(hw) = sub.open_subkey("HardwareInformation") {
            if let Ok(v) = hw.get_value::<u64, _>("qwMemorySize") { return Some(v); }
            if let Ok(v) = hw.get_value::<u32, _>("MemorySize") { return Some(v as u64); }
        }
        None
    }

    // ─── GPU usage via various methods ────────────────────

    fn gpu_usage_perf_counters() -> f32 {
        // PowerShell Get-Counter (most compatible)
        let ps = cmd_output("powershell", &["-NoProfile", "-Command",
            "$c = Get-Counter -Counter @('\\GPU Engine(*)\\Utilization Percentage') -ErrorAction SilentlyContinue -SampleInterval 1 -MaxSamples 2 | Select -ExpandProperty CounterSamples | Where CookedValue -gt 0 | Measure -Maximum CookedValue; if ($c.Count -gt 0) { $c.Maximum } else { 0 }"]);
        if let Ok(v) = ps.trim().parse::<f32>() { if v > 0.0 { return v; } }

        // typeperf
        let tp = cmd_output("typeperf", &["\"\\GPU Engine(*)\\Utilization Percentage\"", "-sc", "1"]);
        for line in tp.lines() {
            if line.contains(',') {
                if let Some(val) = line.rsplit(',').next() {
                    if let Ok(v) = val.trim_matches('"').parse::<f32>() { if v > 0.0 { return v; } }
                }
            }
        }

        // wmic
        let out = cmd_output("wmic",
            &["path", "Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine",
              "get", "Name,PercentUtilization", "/format:csv", "/noheading"]);
        let mut max_u = 0.0f32;
        for line in out.lines() {
            if line.contains("engtype_") {
                if let Some(val) = line.rsplit(',').next() {
                    if let Ok(v) = val.trim().parse::<f32>() { if v > max_u { max_u = v; } }
                }
            }
        }
        max_u
    }

    // ─── Main collector ───────────────────────────────────

    pub fn collect_gpu() -> super::GpuInfo {
        // ── Primary: nvidia-smi ──
        if has_nvidia_smi() {
            if let Some((usage, mem_used, mem_total, temp, clock)) = try_nvidia_smi() {
                let gpus = enumerate_dxgi_gpus();
                let model = select_gpu(&gpus)
                    .map(|i| gpus[i].0.clone())
                    .unwrap_or_else(|| "NVIDIA GPU".into());
                return super::GpuInfo {
                    model,
                    vendor: "NVIDIA (Discrete)".into(),
                    frequency_mhz: clock,
                    utilization_pct: usage,
                    temperature_c: if temp > 0.0 { Some(temp) } else { None },
                    memory_used_mb: mem_used,
                    memory_total_mb: mem_total,
                };
            }
        }

        // ── Fallback: DXGI + registry + WMI ──
        let gpus = enumerate_dxgi_gpus();
        let selected = select_gpu(&gpus);
        let mut model = String::new();
        let mut vram_total: u64 = 0;
        let mut vram_used: u64 = 0;
        let mut vendor_id: u32 = 0;
        let mut gpu_type_str = "Unknown".to_string();

        if let Some(i) = selected {
            model = gpus[i].0.clone();
            vram_total = gpus[i].1;
            vram_used = gpus[i].2;
            vendor_id = gpus[i].4;
            gpu_type_str = match gpus[i].3 {
                GpuType::Integrated => "Integrated",
                GpuType::Discrete => "Discrete",
                GpuType::Unknown => "Unknown",
            }.into();
        }

        let mut vendor = String::new();
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let gpu_guid = r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";

        // Registry for model + vendor + VRAM
        if model.is_empty() || vram_total == 0 {
            if let Ok(parent) = hklm.open_subkey(gpu_guid) {
                for sub_name in parent.enum_keys().filter_map(|k| k.ok()) {
                    if let Ok(sub) = parent.open_subkey(&sub_name) {
                        let desc: String = sub.get_value("DriverDesc").unwrap_or_default();
                        if !desc.is_empty() {
                            if model.is_empty() { model = desc; }
                            vendor = sub.get_value("ProviderName").unwrap_or_default();
                            if vram_total == 0 { vram_total = reg_vram(&sub).unwrap_or(0); }
                            break;
                        }
                    }
                }
            }
        }

        if model.is_empty() { model = wmic_val("path win32_VideoController", "Name"); }
        if model.is_empty() { model = "Unknown".into(); }

        if vendor.is_empty() {
            vendor = match vendor_id {
                0x8086 => "Intel".into(), 0x10DE => "NVIDIA".into(), 0x1002 => "AMD".into(), _ => "Unknown".into()
            };
        }
        let vendor_display = format!("{} ({})", vendor, gpu_type_str);

        if vram_total == 0 {
            vram_total = wmic_val("path win32_VideoController", "AdapterRAM").parse().unwrap_or(0);
        }

        // GPU usage — perf counters (may be 0 if counters not available)
        let usage = gpu_usage_perf_counters();

        super::GpuInfo {
            model,
            vendor: vendor_display,
            frequency_mhz: 0,
            utilization_pct: usage,
            temperature_c: None,
            memory_used_mb: if vram_used > 0 { (vram_used / (1024*1024)) as u64 } else { 0 },
            memory_total_mb: if vram_total > 0 { (vram_total / (1024*1024)) as u64 } else { 0 },
        }
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use std::process::Command;
    pub fn collect_gpu() -> super::GpuInfo {
        let out = Command::new("system_profiler").args(["SPDisplaysDataType","-detailLevel","mini"]).output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default();
        fn extract(text: &str, key: &str) -> String {
            text.lines().find(|l| l.contains(key)).and_then(|l| l.split(':').nth(1)).map(|s| s.trim().to_string()).unwrap_or_default()
        }
        let vram: f32 = extract(&out, "VRAM (Total):").split_whitespace().next().unwrap_or("0").parse().unwrap_or(0.0);
        let vram_mb = if extract(&out, "VRAM (Total):").contains("GB") { (vram*1024.0) as u64 } else { vram as u64 };
        let usage: f32 = {
            let io = Command::new("ioreg").args(["-r","-d","1","-c","IOAccelerator","-w","0"]).output()
                .map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default();
            if let Some(pos) = io.find("\"Device Utilization %\"") {
                io[pos+24..].split(|c: char| c==','||c=='}'||c==';').next()
                    .and_then(|s| s.split('=').nth(1))
                    .and_then(|s| s.trim().replace('%',"").replace('"',"").parse().ok()).unwrap_or(0.0)
            } else { 0.0 }
        };
        super::GpuInfo {
            model: if extract(&out,"Chipset Model:").is_empty(){"Unknown".into()}else{extract(&out,"Chipset Model:")},
            vendor: if extract(&out,"Vendor:").is_empty(){"Unknown".into()}else{extract(&out,"Vendor:")},
            frequency_mhz: 0, utilization_pct: usage, temperature_c: None,
            memory_used_mb: 0, memory_total_mb: vram_mb,
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod fallback {
    pub fn collect_gpu() -> super::GpuInfo {
        super::GpuInfo { model:"unsupported OS".into(), vendor:"n/a".into(), frequency_mhz:0, utilization_pct:0.0, temperature_c:None, memory_used_mb:0, memory_total_mb:0 }
    }
}

#[cfg(windows)] use win as imp;
#[cfg(target_os = "macos")] use mac as imp;
#[cfg(not(any(windows, target_os = "macos")))] use fallback as imp;

pub fn collect() -> GpuInfo { imp::collect_gpu() }
