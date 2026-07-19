use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub memory_total: u64,
    pub memory_available: u64,
    pub cpu_count: usize,
}

pub fn get_system_info() -> SystemInfo {
    let sys = sysinfo::System::new_all();

    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hostname: hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        memory_total: sys.total_memory(),
        memory_available: sys.available_memory(),
        cpu_count: sys.cpus().len(),
    }
}
