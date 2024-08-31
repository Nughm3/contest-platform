use std::{path::Path, process::Stdio};

use tokio::io::{AsyncReadExt, AsyncWriteExt, Error, ErrorKind, Result};

pub use self::{command::*, resource::*};

mod command;
mod resource;
mod seccomp;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Profile {
    Build,
    Run(ResourceLimits),
}

pub async fn run(
    dir: impl AsRef<Path>,
    command: &Command,
    stdin: &[u8],
    profile: Profile,
) -> Result<Output> {
    let dir = dir.as_ref();

    let mut child = {
        let mut cmd = command.create();
        cmd.current_dir(dir)
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

    Ok(Output::new(exit_status, stdout, stderr, resource_usage))
}
