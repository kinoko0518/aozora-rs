use serde::Serialize;
use sysinfo::{Disks, System};

#[derive(Serialize)]
pub struct SysInfo {
    os_name: String,
    os_version: String,
    kernel: String,
    architecture: String,

    cpu_name: String,
    memory_size: u64,
    disk_info: Vec<(u64, String, String)>,

    rustc_version: String,
}

pub fn get_sysinfo() -> SysInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    // OS情報
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let architecture = System::cpu_arch();

    // CPU情報
    let cpus = sys.cpus();
    let cpu_name = if let Some(cpu) = cpus.first() {
        cpu.brand()
    } else {
        "None"
    }
    .to_string();

    // メモリ情報
    let memory_size = sys.total_memory();

    // ドライブ情報
    let disks = Disks::new_with_refreshed_list();
    let disk_info: Vec<_> = disks
        .list()
        .iter()
        .map(|disk| {
            let size_gb = disk.total_space();
            let mount_point = disk.mount_point().to_string_lossy().to_string();
            let fs_type = disk.file_system().to_string_lossy().to_string();

            (size_gb, mount_point, fs_type)
        })
        .collect();

    let rustc_version = env!("RUSTC_VERSION").to_string();

    SysInfo {
        os_name,
        os_version,
        kernel,
        architecture,
        cpu_name,
        memory_size,
        disk_info,
        rustc_version,
    }
}
