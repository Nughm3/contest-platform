use std::{collections::HashMap, fs, path::Path};

use color_eyre::eyre::{OptionExt, WrapErr};
use serde::Deserialize;
use which::which;

use super::*;

pub fn load(path: impl AsRef<Path>) -> color_eyre::Result<Config> {
    let path = path.as_ref();

    let config: ConfigFile = {
        let input = fs::read_to_string(path.join("judge.toml"))
            .wrap_err("failed to read judge.toml file")?;
        toml::from_str(&input)?
    };

    Ok(Config {
        contests: {
            let mut contests = HashMap::new();

            for entry in fs::read_dir(path)? {
                let path = entry?.path();
                if path.is_dir() {
                    let name = path
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .ok_or_eyre("could not obtain contest name")?;
                    contests.insert(name.to_owned(), load_contest(&path)?);
                }
            }

            tracing::trace!("{} contests", contests.len());
            contests
        },
        resource_limits_build: config.resource_limits.build,
        resource_limits_run: config.resource_limits.run,
        languages: {
            let mut languages = HashMap::new();

            for language in config.languages {
                let build = if let Some(build) = language.build {
                    let (executable, args) = build.split_first().ok_or_eyre("empty command")?;
                    let executable = which(executable).wrap_err(format!(
                        "failed to find build command {executable} for language {}",
                        language.name
                    ))?;

                    Some(Command::new(executable, args))
                } else {
                    None
                };

                let run = {
                    let (executable, args) =
                        language.run.split_first().ok_or_eyre("empty command")?;

                    let mut executable = PathBuf::from(executable);
                    if build.is_none() {
                        executable = which(&executable).wrap_err(format!(
                            "failed to find run command {} for language {}",
                            executable.display(),
                            language.name
                        ))?;
                    }

                    Command::new(executable, args)
                };

                languages.insert(
                    language.name,
                    Language {
                        filename: language.filename,
                        build,
                        run,
                    },
                );
            }

            languages
        },
    })
}

fn load_contest(path: &Path) -> color_eyre::Result<Contest> {
    let mut tasks = Vec::new();

    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_dir() {
            tasks.push(load_task(&path)?);
        }
    }

    tracing::trace!("contest {}: {} tasks", path.display(), tasks.len());
    Ok(Contest { tasks })
}

fn load_task(path: &Path) -> color_eyre::Result<Task> {
    let mut subtasks = Vec::new();

    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_dir() {
            subtasks.push(load_subtask(&path)?);
        }
    }

    tracing::trace!("task {}: {} subtasks", path.display(), subtasks.len());
    Ok(Task { subtasks })
}

fn load_subtask(path: &Path) -> color_eyre::Result<Subtask> {
    let mut tests = Vec::new();

    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        if let Some("in") = path.extension().and_then(|s| s.to_str()) {
            let (input, output) = (
                fs::read_to_string(&path)?,
                fs::read_to_string(path.with_extension("out"))?,
            );
            tests.push(Test { input, output });
        }
    }

    tracing::trace!("subtask {}: {} tests", path.display(), tests.len());
    Ok(Subtask { tests })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct ConfigFile {
    #[serde(alias = "rlimits")]
    resource_limits: ResourceLimitsConfig,
    #[serde(alias = "language")]
    languages: Vec<LanguageConfig>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceLimitsConfig {
    build: Option<ResourceLimits>,
    run: ResourceLimits,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LanguageConfig {
    name: String,
    filename: String,
    build: Option<Vec<String>>,
    run: Vec<String>,
}
