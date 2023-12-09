//! System monitoring with [sysinfo].

use std::time::Duration;

use anyhow::{anyhow, Result};
use log::*;
use sysinfo::{
    CpuExt, CpuRefreshKind, PidExt, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt,
};

use crate::model::*;

use super::MonitorBackend;

pub fn initialize() -> Result<System> {
    let mut sys = System::new();
    sys.refresh_specifics(RefreshKind::everything());
    Ok(sys)
}

impl MonitorBackend for System {
    fn update(&mut self, _opts: &Options) -> Result<()> {
        debug!("refreshing system");
        let specs = RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory()
            .with_processes(ProcessRefreshKind::everything());
        self.refresh_specifics(specs);
        Ok(())
    }

    fn hostname(&self) -> Result<String> {
        self.host_name().ok_or(anyhow!("no host name"))
    }

    fn system_version(&self) -> Result<String> {
        let os = self.distribution_id();
        let osv = self.os_version().ok_or(anyhow!("no system version"))?;
        let k = self.name().ok_or(anyhow!("no system name"))?;
        let kv = self.kernel_version().ok_or(anyhow!("no kernel version"))?;
        Ok(format!("{} {} with {} {}", os, osv, k, kv))
    }

    fn uptime(&self) -> Result<Duration> {
        Ok(Duration::from_secs(SystemExt::uptime(self)))
    }

    fn cpu_count(&self) -> Result<u32> {
        Ok(self.cpus().len() as u32)
    }

    fn global_cpu(&self) -> Result<CPU> {
        Ok(CPU {
            utilization: self.global_cpu_info().cpu_usage() / 100.0,
        })
    }

    fn memory(&self) -> Result<Memory> {
        let used = self.used_memory();
        let total = self.total_memory();
        let free = self.free_memory();
        let freeable = self.available_memory().saturating_sub(free);
        Ok(Memory {
            used,
            freeable,
            free,
            total,
        })
    }

    fn swap(&self) -> Result<Swap> {
        Ok(Swap {
            used: self.used_swap(),
            free: self.free_swap(),
            total: self.total_swap(),
        })
    }

    fn load_avg(&self) -> Result<LoadAvg> {
        let la = self.load_average();
        Ok(LoadAvg {
            one: la.one as f32,
            five: la.five as f32,
            fifteen: la.fifteen as f32,
        })
    }

    fn processes<'a>(&'a self) -> Result<Vec<Process<'a>>> {
        let procs = SystemExt::processes(self);
        let mut out = Vec::with_capacity(procs.len());
        for proc in procs.values() {
            let disk = proc.disk_usage();
            out.push(Process {
                pid: proc.pid().as_u32(),
                ppid: proc.parent().map(|p| p.as_u32()),
                name: proc.name().into(),
                exe: proc.exe().into(),
                cmd: proc.cmd().into(),
                uid: proc.user_id().map(|u| **u),
                status: match proc.status() {
                    sysinfo::ProcessStatus::Idle => 'I',
                    sysinfo::ProcessStatus::Run => 'R',
                    sysinfo::ProcessStatus::Sleep => 'S',
                    sysinfo::ProcessStatus::Stop => 'T',
                    sysinfo::ProcessStatus::Zombie => 'Z',
                    sysinfo::ProcessStatus::Tracing => 'G',
                    sysinfo::ProcessStatus::Dead => 'D',
                    sysinfo::ProcessStatus::Wakekill => 'K',
                    sysinfo::ProcessStatus::Waking => 'W',
                    sysinfo::ProcessStatus::Parked => 'P',
                    sysinfo::ProcessStatus::LockBlocked => 'L',
                    sysinfo::ProcessStatus::UninterruptibleDiskSleep => 'U',
                    sysinfo::ProcessStatus::Unknown(_) => 'X',
                },
                cpu: proc.cpu_usage(),
                mem_rss: proc.memory(),
                mem_virt: proc.virtual_memory(),
                io: if disk.total_read_bytes > 0 {
                    Some(IOUsage {
                        tot_read: disk.total_read_bytes,
                        tot_write: disk.total_written_bytes,
                        new_read: disk.read_bytes,
                        new_write: disk.written_bytes,
                    })
                } else {
                    None
                },
                wall_time: Duration::from_secs(proc.run_time()),
            })
        }
        Ok(out)
    }
}
