use std::{
    path::Path,
    process::{ExitStatus, Stdio},
    str,
};

pub use resource::{ResourceLimits, ResourceUsage};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Error, ErrorKind},
    process::Command,
};

mod resource;
mod seccomp;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Profile {
    Compile,
    Run(ResourceLimits),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    exit_status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    resource_usage: ResourceUsage,
}

impl Output {
    pub fn new(
        exit_status: ExitStatus,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        resource_usage: ResourceUsage,
    ) -> Self {
        Self {
            exit_status,
            stdout,
            stderr,
            resource_usage,
        }
    }

    pub fn exit_status(&self) -> ExitStatus {
        self.exit_status
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stdout_utf8(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(self.stdout())
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }

    pub fn stderr_utf8(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(self.stderr())
    }

    pub fn resource_usage(&self) -> ResourceUsage {
        self.resource_usage
    }
}

pub async fn run(
    dir: impl AsRef<Path>,
    command: &[String],
    stdin: &[u8],
    profile: Profile,
) -> Result<Output, Error> {
    let dir = dir.as_ref();

    let mut child = {
        let (executable, args) = command
            .split_first()
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "empty command"))?;

        let mut cmd = Command::new(executable);
        cmd.args(args)
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Profile::Run(resource_limits) = profile {
            unsafe {
                cmd.pre_exec(move || {
                    resource_limits.set()?;

                    seccomp::apply_filters().map_err(|e| {
                        Error::new(ErrorKind::Other, format!("seccomp failed: {e}"))
                    })?;

                    Ok(())
                });
            }
        }

        cmd.spawn()?
    };

    if let Err(e) = child.stdin.take().expect("no stdin").write_all(stdin).await {
        tracing::error!("failed to write stdin: {e}");
    }

    let (stdout, stderr) = {
        let (mut stdout, mut stderr) = (
            child.stdout.take().expect("no stdout"),
            child.stderr.take().expect("no stderr"),
        );
        let (mut stdout_buf, mut stderr_buf) = (Vec::new(), Vec::new());

        if let Err(e) = stdout.read_to_end(&mut stdout_buf).await {
            tracing::error!("failed to read stdout: {e}");
        }

        if let Err(e) = stderr.read_to_end(&mut stderr_buf).await {
            tracing::error!("failed to read stderr: {e}");
        }

        (stdout_buf, stderr_buf)
    };

    let (exit_status, resource_usage) = tokio::task::spawn_blocking(move || {
        resource::wait4(child.id().expect("child process has no PID") as i32)
    })
    .await??;

    Ok(Output::new(exit_status, stdout, stderr, resource_usage))
}
