use crate::types::CpuInfo;

#[cfg(windows)]
mod win {
    use winreg::enums::*;
    use winreg::RegKey;
    use std::process::Command;
    use std::time::Duration;
    use std::thread;

    /// Read a registry value, trying DWORD first then QWORD then STRING
    fn reg_u64(k: &RegKey, name: &str) -> Option<u64> {
        k.get_value::<u32, _>(name).ok().map(|v| v as u64)
            .or_else(|| k.get_value::<u64, _>(name).ok())
            .or_else(|| {
                let s: String = k.get_value(name).ok()?;
                s.parse().ok()
            })
    }

    /// Run a command and return stdout as lossy String
    fn cmd_output(program: &str, args: &[&str]) -> String {
        Command::new(program).args(args).output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default()
    }

    /// Extract a CSV field from wmic output
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

    /// Try to get CPU temperature via Performance Counter (most reliable)
    /// Value is in Kelvin (e.g. 290 = 17°C)
    fn cpu_temp_perfcounter() -> Option<f32> {
        let ps = cmd_output("powershell", &["-NoProfile", "-Command",
            "$v=(Get-Counter '\\Thermal Zone Information(*)\\Temperature' -ErrorAction SilentlyContinue).CounterSamples.CookedValue | Where-Object{$_ -gt 0 -and $_ -lt 500} | Select-Object -First 1; if($v){[math]::Round($v-273.15,1)}"]);
        ps.trim().parse::<f32>().ok().filter(|&t| t > -50.0 && t < 150.0)
    }

    /// Try to get CPU temperature via WMI ThermalZone (tenths of Kelvin → °C)
    fn cpu_temp_thermalzone() -> Option<f32> {
        let ps = cmd_output("powershell", &["-NoProfile", "-Command",
            "$v=(Get-CimInstance -Namespace root/wmi -Class MSAcpi_ThermalZoneTemperature -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty CurrentTemperature); if($v-gt0){[math]::Round(($v/10)-273.15,1)}"]);
        ps.trim().parse::<f32>().ok().filter(|&t| t > -50.0 && t < 150.0)
    }

    /// Try to get CPU usage via GetSystemTimes
    fn cpu_usage_system_times() -> Option<f32> {
        use windows::Win32::System::Threading::GetSystemTimes;
        use windows::Win32::Foundation::FILETIME;
        unsafe {
            let mut idle1 = FILETIME::default();
            let mut kernel1 = FILETIME::default();
            let mut user1 = FILETIME::default();
            if GetSystemTimes(Some(&mut idle1), Some(&mut kernel1), Some(&mut user1)).is_ok() {
                thread::sleep(Duration::from_millis(200));
                let mut idle2 = FILETIME::default();
                let mut kernel2 = FILETIME::default();
                let mut user2 = FILETIME::default();
                if GetSystemTimes(Some(&mut idle2), Some(&mut kernel2), Some(&mut user2)).is_ok() {
                    let to_u64 = |f: &FILETIME| f.dwLowDateTime as u64 | ((f.dwHighDateTime as u64) << 32);
                    let idle_d = to_u64(&idle2).wrapping_sub(to_u64(&idle1));
                    let kernel_d = to_u64(&kernel2).wrapping_sub(to_u64(&kernel1));
                    let user_d = to_u64(&user2).wrapping_sub(to_u64(&user1));
                    let total = kernel_d + user_d;
                    if total > 0 { return Some(((total - idle_d) as f32 / total as f32) * 100.0); }
                }
            }
        }
        None
    }

    /// Try to get CPU usage via wmic
    fn cpu_usage_wmic() -> Option<f32> {
        let v = wmic_val("cpu", "LoadPercentage");
        v.parse().ok()
    }

    /// Perf Counter % Processor Performance via PowerShell (fast, accurate)
    fn cpu_freq_perf_counter(max_mhz: u64) -> Option<u64> {
        if max_mhz == 0 { return None; }
        let ps = cmd_output("powershell", &["-NoProfile", "-Command",
            "$v = (Get-Counter '\\Processor Information(_Total)\\% Processor Performance' -ErrorAction SilentlyContinue).CounterSamples.CookedValue; if ($v) { [math]::Round($v) } else { 0 }"]);
        if let Ok(pct) = ps.trim().parse::<f64>() {
            if pct > 0.0 {
                return Some(((max_mhz as f64 * pct / 100.0).round() as u64).max(100));
            }
        }
        None
    }

    /// Try to get current CPU frequency via wmic
    fn cpu_freq_wmic() -> Option<u64> {
        let v = wmic_val("cpu", "CurrentClockSpeed");
        v.parse::<u64>().ok().filter(|&f| f > 0)
    }

    /// Try to get CPU frequency via registry (rated max)
    fn cpu_freq_reg(hklm: &RegKey) -> Option<u64> {
        hklm.open_subkey(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0").ok()
            .and_then(|k| reg_u64(&k, "~MHz"))
    }

    pub fn collect_cpu() -> super::CpuInfo {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let cpu_key_path = r"HARDWARE\DESCRIPTION\System\CentralProcessor\0";

        // Model — registry first, then wmic
        let model: String = hklm.open_subkey(cpu_key_path).ok()
            .and_then(|k| k.get_value::<String, _>("ProcessorNameString").ok())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                let m = wmic_val("cpu", "Name");
                if m.is_empty() { "Unknown".into() } else { m }
            });

        // Cores — registry
        let cores: u32 = hklm.open_subkey(r"HARDWARE\DESCRIPTION\System\CentralProcessor").ok()
            .map(|k| k.enum_keys().filter_map(|n| n.ok()).count() as u32)
            .unwrap_or(0);

        // Frequency — PerfCounter → wmic → registry rated
        let max_mhz = cpu_freq_reg(&hklm).unwrap_or(0);
        let freq_mhz: u64 = cpu_freq_perf_counter(max_mhz)
            .or_else(|| cpu_freq_wmic())
            .unwrap_or(max_mhz);

        // Usage — GetSystemTimes, then wmic
        let usage: f32 = cpu_usage_system_times()
            .or_else(cpu_usage_wmic)
            .unwrap_or(0.0);

        super::CpuInfo {
            model,
            cores_physical: cores,
            cores_logical: cores,
            frequency_mhz: freq_mhz,
            utilization_pct: usage,
            temperature_c: cpu_temp_perfcounter().or(cpu_temp_thermalzone()).filter(|&t| t > 30.0),
        }
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use std::process::Command;

    fn sysctl(key: &str) -> String {
        Command::new("sysctl").args(["-n", key]).output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default().trim().to_string()
    }

    pub fn collect_cpu() -> super::CpuInfo {
        let model = sysctl("machdep.cpu.brand_string");
        let p: u32 = sysctl("hw.perflevel0.physicalcpu").parse().unwrap_or(0);
        let e: u32 = sysctl("hw.perflevel1.physicalcpu").parse().unwrap_or(0);
        let physical = if p+e>0 { p+e } else { sysctl("hw.physicalcpu").parse().unwrap_or(0) };
        let logical: u32 = sysctl("hw.logicalcpu").parse().unwrap_or(0);
        let freq_hz: u64 = sysctl("hw.cpufrequency").parse().unwrap_or(0);

        fn apple_freq(model: &str) -> u64 {
            if model.contains("M3 Ultra")||model.contains("M3 Max")||model.contains("M3 Pro")||model.contains("M3") { return 4050; }
            if model.contains("M2 Ultra")||model.contains("M2 Max")||model.contains("M2 Pro")||model.contains("M2") { return 3490; }
            if model.contains("M1 Ultra")||model.contains("M1 Max")||model.contains("M1 Pro") { return 3220; }
            if model.contains("M1") { return 3200; } 0
        }

        let usage = {
            let out = Command::new("top").args(["-l","2","-n","0","-s","0"]).output()
                .map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default();
            let mut u = 0.0f32;
            for line in out.lines().rev() {
                if line.contains("CPU usage:") {
                    if let Some(idle) = line.split("idle").next().and_then(|s| s.rsplit(',').next()) {
                        u = 100.0 - idle.replace('%',"").trim().parse::<f32>().unwrap_or(100.0);
                    } break;
                }
            }
            u.max(0.0)
        };

        super::CpuInfo {
            model: if model.is_empty(){"Unknown".into()}else{model.clone()},
            cores_physical: physical, cores_logical: logical,
            frequency_mhz: if freq_hz>0 { freq_hz/1_000_000 } else { apple_freq(&model) },
            utilization_pct: usage, temperature_c: None,
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod fallback {
    pub fn collect_cpu() -> super::CpuInfo {
        super::CpuInfo { model:"unsupported OS".into(), cores_physical:0, cores_logical:0, frequency_mhz:0, utilization_pct:0.0, temperature_c:None }
    }
}

#[cfg(windows)] use win as imp;
#[cfg(target_os = "macos")] use mac as imp;
#[cfg(not(any(windows, target_os = "macos")))] use fallback as imp;

pub fn collect() -> CpuInfo { imp::collect_cpu() }
