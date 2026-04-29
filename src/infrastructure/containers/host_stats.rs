// SPDX-License-Identifier: GPL-3.0-or-later
use sysinfo::{Disks, System};

use crate::core::domain::network::HostStats;
use crate::infrastructure::containers::error::ContainerError;

/// Read CPU, memory, and disk utilisation from the host OS.
///
/// CPU usage requires two samples to produce a meaningful reading;
/// a single refresh gives an approximate value that is good enough for a
/// dashboard that refreshes every 30 s.
pub fn read_host_stats() -> Result<HostStats, ContainerError> {
    let mut sys = System::new();
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let cpu_percent = sys.global_cpu_usage() as f64;
    let mem_total = sys.total_memory();
    let mem_used = sys.used_memory();
    let mem_percent = if mem_total > 0 {
        mem_used as f64 / mem_total as f64 * 100.0
    } else {
        0.0
    };

    let disks = Disks::new_with_refreshed_list();
    let total_space: u64 = disks.iter().map(|d| d.total_space()).sum();
    let avail_space: u64 = disks.iter().map(|d| d.available_space()).sum();
    let disk_percent = if total_space > 0 {
        (total_space.saturating_sub(avail_space)) as f64 / total_space as f64 * 100.0
    } else {
        0.0
    };

    Ok(HostStats {
        cpu_percent: cpu_percent.clamp(0.0, 100.0),
        mem_percent: mem_percent.clamp(0.0, 100.0),
        disk_percent: disk_percent.clamp(0.0, 100.0),
        disk_total_bytes: total_space,
    })
}
