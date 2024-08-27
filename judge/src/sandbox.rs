use std::{path::Path, process::Stdio};

use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt, Error, ErrorKind, Result};

pub use self::{
    command::{Command, Output},
    resource::{ResourceLimits, ResourceUsage},
};

mod command;
mod landlock;
mod resource;
mod seccomp;

#[derive(Debug)]
pub struct Sandbox {
    dir: TempDir,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SandboxConfig {
    pub resource_limits: Option<ResourceLimits>,
    pub seccomp: bool,
    pub landlock: bool,
}

impl Sandbox {
    pub fn new() -> Result<Self> {
        Ok(Sandbox {
            dir: TempDir::new()?,
        })
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub async fn run(
        &self,
        command: &Command,
        config: SandboxConfig,
        stdin: &[u8],
    ) -> Result<Output> {
        std::env::set_current_dir(&self.dir)?;
        if !stdin.is_empty() {
            let mut rd = tokio::fs::read_dir(".").await?;
            while let Some(e) = rd.next_entry().await? {
                dbg!(e.path());
            }
            tokio::process::Command::new("./submission")
                .current_dir(&self.dir)
                .status()
                .await?;
        }

        println!("got here");

        let mut child = {
            let mut cmd = command.create();
            cmd.current_dir(&self.dir)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            unsafe {
                let dir = self.path().to_owned();

                cmd.pre_exec(move || {
                    if config.landlock {
                        landlock::apply_landlock(&dir).map_err(|e| {
                            Error::new(ErrorKind::Other, format!("landlock failed: {e}"))
                        })?;
                    }

                    if let Some(resource_limits) = config.resource_limits {
                        resource_limits.set()?;
                    }

                    if config.seccomp {
                        seccomp::apply_filters().map_err(|e| {
                            Error::new(ErrorKind::Other, format!("seccomp failed: {e}"))
                        })?;
                    }

                    Ok(())
                });
            }

            cmd.spawn()?
        };

        child
            .stdin
            .take()
            .expect("no stdin")
            .write_all(stdin)
            .await?;

        let (stdout, stderr) = {
            let (mut stdout, mut stderr) = (
                child.stdout.take().expect("no stdout"),
                child.stderr.take().expect("no stderr"),
            );
            let (mut stdout_buf, mut stderr_buf) = (Vec::new(), Vec::new());

            stdout.read_to_end(&mut stdout_buf).await?;
            stderr.read_to_end(&mut stderr_buf).await?;

            (stdout_buf, stderr_buf)
        };

        let (exit_status, resource_usage) = tokio::task::spawn_blocking(move || {
            resource::wait4(child.id().expect("child process has no PID") as i32)
        })
        .await??;

        Ok(Output {
            exit_status,
            stdout,
            stderr,
            resource_usage,
        })
    }
}
