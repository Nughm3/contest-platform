use std::{io, os::unix::process::ExitStatusExt, process::ExitStatus, time::Duration};

use rlimit::{setrlimit, Resource};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ResourceUsage {
    /// User time
    pub user_time: Duration,
    /// System time
    pub sys_time: Duration,
    /// Memory usage (bytes)
    pub memory: u64,
}

impl ResourceUsage {
    pub const TIME_TOLERANCE: f64 = 0.1;

    pub const MEMORY_TOLERANCE: u64 = 1000;

    pub fn total_time(&self) -> Duration {
        self.user_time + self.sys_time
    }

    pub fn exceeded(&self, resource_limits: ResourceLimits) -> bool {
        self.exceeded_time(resource_limits) || self.exceeded_memory(resource_limits)
    }

    pub fn exceeded_time(&self, resource_limits: ResourceLimits) -> bool {
        (self.total_time().as_secs_f64() - resource_limits.cpu as f64).abs() <= Self::TIME_TOLERANCE
    }

    pub fn exceeded_memory(&self, resource_limits: ResourceLimits) -> bool {
        self.memory.abs_diff(resource_limits.memory) <= Self::MEMORY_TOLERANCE
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ResourceLimits {
    /// CPU time (seconds)
    pub cpu: u64,
    /// Memory usage (bytes)
    pub memory: u64,
}

impl ResourceLimits {
    pub fn set(&self) -> io::Result<()> {
        setrlimit(Resource::CPU, self.cpu, self.cpu)?;
        setrlimit(Resource::DATA, self.memory, self.memory)?;
        Ok(())
    }
}

pub fn wait4(pid: i32) -> io::Result<(ExitStatus, ResourceUsage)> {
    let mut status = 0;
    let mut rusage = std::mem::MaybeUninit::zeroed();

    let result = unsafe { libc::wait4(pid, &mut status, 0, rusage.as_mut_ptr()) };

    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        let rusage = unsafe { rusage.assume_init() };
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as i64;

        let convert = |timeval: libc::timeval| {
            let duration = timeval.tv_sec * 1_000_000 + timeval.tv_usec;
            Duration::from_micros(duration as u64)
        };

        Ok((
            ExitStatus::from_raw(status),
            ResourceUsage {
                user_time: convert(rusage.ru_utime),
                sys_time: convert(rusage.ru_stime),
                memory: (rusage.ru_maxrss * page_size) as u64,
            },
        ))
    }
}
