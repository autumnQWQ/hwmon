use crate::types::SystemInfo;

/// Human-readable single-line output for `--watch` mode.
#[allow(dead_code)]
pub fn one_line(info: &SystemInfo) -> String {
    format!(
        "CPU {:>4}MHz {:>5.1}% {}°C | GPU {:>4}MHz {:>5.1}% {}°C | {}",
        info.cpu.frequency_mhz,
        info.cpu.utilization_pct,
        temp_str(info.cpu.temperature_c),
        info.gpu.frequency_mhz,
        info.gpu.utilization_pct,
        temp_str(info.gpu.temperature_c),
        &info.timestamp[11..19],
    )
}

/// Pretty-printed block for single-shot `hwmon` (no --watch), plain text.
#[allow(dead_code)]
pub fn block(info: &SystemInfo) -> String {
    block_fmt(info, temp_str)
}

/// Pretty-printed block with ANSI color grading for temperatures.
pub fn block_color(info: &SystemInfo) -> String {
    block_fmt(info, temp_colored)
}

fn block_fmt<F>(info: &SystemInfo, fmt_temp: F) -> String
where
    F: Fn(Option<f32>) -> String,
{
    let cpu_temp = fmt_temp(info.cpu.temperature_c);
    let gpu_temp = fmt_temp(info.gpu.temperature_c);

    // Compute padding to keep ASCII table columns aligned despite ANSI codes.
    let cpu_pad = temp_pad(info.cpu.temperature_c, &cpu_temp, 10);
    let gpu_pad = temp_pad(info.gpu.temperature_c, &gpu_temp, 10);

    format!(
        "\
┌─ CPU ───────────────────────────────────┐
│ Model:     {cpu_model:<30} │
│ Cores:     {cpu_cores_p}物理 / {cpu_cores_l}逻辑{spc1} │
│ Frequency: {cpu_freq} MHz{spc2} │
│ Usage:     {cpu_usage}%{spc3} │
│ Temp:      {cpu_temp}{cpu_pad} │
┌─ GPU ───────────────────────────────────┐
│ Model:     {gpu_model:<30} │
│ Vendor:    {gpu_vendor:<30} │
│ Frequency: {gpu_freq} MHz{spc5} │
│ Usage:     {gpu_usage}%{spc6} │
│ Temp:      {gpu_temp}{gpu_pad} │
│ Memory:    {gpu_mem_used} / {gpu_mem_total} MB{spc8} │
└──────────────────────────────────────────┘
┌─ RAM ───────────────────────────────────┐
│ Used:      {mem_used} / {mem_total} GB ({mem_pct:.1}%){spc_mem} │
└──────────────────────────────────────────┘
  {timestamp}",
        cpu_model = trunc(info.cpu.model.as_str(), 30),
        cpu_cores_p = info.cpu.cores_physical,
        cpu_cores_l = info.cpu.cores_logical,
        spc1 = pad(10, 15),
        cpu_freq = info.cpu.frequency_mhz,
        spc2 = pad(18, 15),
        cpu_usage = format!("{:.1}", info.cpu.utilization_pct),
        spc3 = "",
        cpu_temp = cpu_temp,
        cpu_pad = cpu_pad,
        gpu_model = trunc(info.gpu.model.as_str(), 30),
        gpu_vendor = trunc(info.gpu.vendor.as_str(), 30),
        gpu_freq = info.gpu.frequency_mhz,
        spc5 = pad(18, 15),
        gpu_usage = format!("{:.1}", info.gpu.utilization_pct),
        spc6 = "",
        gpu_temp = gpu_temp,
        gpu_pad = gpu_pad,
        gpu_mem_used = info.gpu.memory_used_mb,
        gpu_mem_total = info.gpu.memory_total_mb,
        spc8 = pad(8, 14),
        mem_used = format!("{:.1}", info.memory.used_gb),
        mem_total = format!("{:.1}", info.memory.total_gb),
        mem_pct = format!("{:.1}", info.memory.used_pct),
        spc_mem = "",
        timestamp = info.timestamp,
    )
}

/// ANSI color for temperature: green <60°C, yellow <80°C, red >=80°C.
fn temp_colored(t: Option<f32>) -> String {
    match t {
        None => "N/A".into(),
        Some(v) if v < 60.0 => format!("\x1b[32m{:.0}°C\x1b[0m", v),
        Some(v) if v < 80.0 => format!("\x1b[33m{:.0}°C\x1b[0m", v),
        Some(v) => format!("\x1b[31m{:.0}°C\x1b[0m", v),
    }
}

/// Plain temperature string.
fn temp_str(t: Option<f32>) -> String {
    t.map(|v| format!("{:.0}°C", v)).unwrap_or_else(|| "N/A".into())
}

/// Compute right-padding to keep table columns aligned.
/// ANSI codes don't occupy visible width, so we subtract them.
fn temp_pad(raw: Option<f32>, formatted: &str, target: usize) -> String {
    let visible = match raw {
        None => 3,  // "N/A"
        Some(v) => format!("{:.0}°C", v).len(),
    };
    let ansi_len = formatted.len().saturating_sub(visible);
    let actual = visible + ansi_len;
    if actual < target { " ".repeat(target - actual) } else { String::new() }
}

fn trunc(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}…", &s[..max - 1])
    } else {
        s.to_string()
    }
}

fn pad(_num: usize, n: usize) -> String {
    " ".repeat(n)
}
