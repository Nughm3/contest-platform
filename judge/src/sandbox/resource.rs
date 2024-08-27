use std::{io, os::unix::process::ExitStatusExt, process::ExitStatus, time::Duration};

use rlimit::{setrlimit, Resource};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ResourceUsage {
    pub user_time: Duration,
    pub sys_time: Duration,
    /// Memory usage (bytes)
    pub memory: u64,
}

impl ResourceUsage {
    pub fn total_time(&self) -> Duration {
        self.user_time + self.sys_time
    }

    pub fn exceeded(&self, resource_limits: ResourceLimits) -> bool {
        self.exceeded_time(resource_limits) || self.exceeded_memory(resource_limits)
    }

    pub fn exceeded_time(&self, resource_limits: ResourceLimits) -> bool {
        self.total_time() > Duration::from_secs(resource_limits.cpu)
    }

    pub fn exceeded_memory(&self, resource_limits: ResourceLimits) -> bool {
        self.memory > resource_limits.memory
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

impl Default for ResourceLimits {
    fn default() -> Self {
        ResourceLimits {
            cpu: 1,
            memory: 512_000_000,
        }
    }
}

impl ResourceLimits {
    #[tracing::instrument]
    pub fn set(&self) -> io::Result<()> {
        setrlimit(Resource::CPU, self.cpu, self.cpu)?;
        setrlimit(Resource::DATA, self.memory, self.memory)?;
        println!("resource limits applied");
        tracing::trace!("resource limits applied");
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

        Ok((
            ExitStatus::from_raw(status),
            ResourceUsage {
                user_time: timeval_to_duration(rusage.ru_utime),
                sys_time: timeval_to_duration(rusage.ru_stime),
                memory: (rusage.ru_maxrss * page_size) as u64,
            },
        ))
    }
}

fn timeval_to_duration(timeval: libc::timeval) -> Duration {
    let v = timeval.tv_sec * 1_000_000 + timeval.tv_usec;
    Duration::from_micros(v as u64)
}
