use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    process::ExitStatus,
    str,
};

use tokio::process::Command as TokioCommand;

use super::resource::ResourceUsage;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Command {
    executable: PathBuf,
    args: Vec<OsString>,
}

impl Command {
    pub fn new(
        executable: impl AsRef<Path>,
        args: impl IntoIterator<Item = impl AsRef<OsStr>>,
    ) -> Self {
        Command {
            executable: executable.as_ref().to_path_buf(),
            args: args.into_iter().map(|s| s.as_ref().to_owned()).collect(),
        }
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn args(&self) -> &[OsString] {
        &self.args
    }

    pub fn create(&self) -> TokioCommand {
        let mut command = TokioCommand::new(&self.executable);
        command.args(&self.args);
        command
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub(super) exit_status: ExitStatus,
    pub(super) stdout: Vec<u8>,
    pub(super) stderr: Vec<u8>,
    pub(super) resource_usage: ResourceUsage,
}

impl Output {
    pub fn exit_status(&self) -> ExitStatus {
        self.exit_status
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stdout_utf8(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(&self.stdout)
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }

    pub fn stderr_utf8(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(&self.stderr)
    }

    pub fn resource_usage(&self) -> ResourceUsage {
        self.resource_usage
    }
}
