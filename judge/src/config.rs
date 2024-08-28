use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;
use thiserror::Error;
use which::which;

use crate::sandbox::{Command, ResourceLimits};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub resource_limits: ResourceLimits,
    pub languages: HashMap<String, Language>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Language {
    pub filename: String,
    pub build: Option<Command>,
    pub run: Command,
}

#[derive(Debug, Error)]
pub enum ConfigLoadError {
    #[error("empty command")]
    EmptyCommand,
    #[error("failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ConfigRepr {
    resource_limits: ResourceLimits,
    #[serde(alias = "language")]
    languages: Vec<LanguageRepr>,
}

#[derive(Deserialize)]
struct LanguageRepr {
    name: String,
    filename: String,
    build: Option<Vec<String>>,
    run: Vec<String>,
}

impl Config {
    pub fn load(s: &str) -> Result<Self, ConfigLoadError> {
        let repr: ConfigRepr = toml::from_str(s)?;
        let mut languages = HashMap::new();
        for language in repr.languages {
            languages.insert(
                language.name,
                Language {
                    filename: language.filename,
                    build: if let Some(build) = language.build {
                        Some(convert_command(build)?)
                    } else {
                        None
                    },
                    run: convert_command(language.run)?,
                },
            );
        }

        Ok(Config {
            resource_limits: repr.resource_limits,
            languages,
        })
    }
}

fn convert_command(repr: Vec<String>) -> Result<Command, ConfigLoadError> {
    let (path, args) = repr.split_first().ok_or(ConfigLoadError::EmptyCommand)?;
    let executable = which(path).unwrap_or_else(|_| PathBuf::from(path));
    Ok(Command::new(executable, args))
}
