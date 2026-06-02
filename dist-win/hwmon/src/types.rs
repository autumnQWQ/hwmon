use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores_physical: u32,
    pub cores_logical: u32,
    pub frequency_mhz: u64,
    pub utilization_pct: f32,
    pub temperature_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GpuInfo {
    pub model: String,
    pub vendor: String,
    pub frequency_mhz: u64,
    pub utilization_pct: f32,
    pub temperature_c: Option<f32>,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryInfo {
    pub used_gb: f32,
    pub total_gb: f32,
    pub used_pct: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub cpu: CpuInfo,
    pub gpu: GpuInfo,
    pub memory: MemoryInfo,
    pub fps: u32,
    pub timestamp: String,
}
