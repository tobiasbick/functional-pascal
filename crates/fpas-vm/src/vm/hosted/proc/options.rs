//! Borrowed process options and launch preflight independent of the host process.

use std::process::Command;
use std::time::Duration;

use super::failure::Failure;

pub(super) const MAX_IO: usize = 64 * 1024 * 1024;
pub(super) const DEFAULT_OUTPUT: usize = 1024 * 1024;
const MAX_CONFIGURATION: usize = 1024 * 1024;

pub(super) struct Options<'a> {
    pub directory: Option<&'a str>,
    pub environment: Vec<(&'a str, Option<&'a str>)>,
    pub input: &'a str,
    pub timeout: Option<Duration>,
    pub maximum: usize,
    pub token: Option<u64>,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            directory: None,
            environment: Vec::new(),
            input: "",
            timeout: None,
            maximum: DEFAULT_OUTPUT,
            token: None,
        }
    }
}

pub(super) struct Request<'a> {
    pub command: &'a str,
    pub arguments: Vec<&'a str>,
    pub options: Options<'a>,
    pub capture: bool,
    pub inherit_input: bool,
}

impl Request<'_> {
    pub(super) fn command(&self) -> Command {
        let mut command = Command::new(self.command);
        command.args(&self.arguments);
        if let Some(directory) = self.options.directory {
            command.current_dir(directory);
        }
        for (name, value) in &self.options.environment {
            if let Some(value) = value {
                command.env(name, value);
            } else {
                command.env_remove(name);
            }
        }
        command
    }

    pub(super) fn validate(&self) -> Result<(), Failure> {
        if self.arguments.len() > 4096 {
            return Err(Failure::limit("arguments", 4096));
        }
        if self.options.environment.len() > 1024 {
            return Err(Failure::limit("environment", 1024));
        }
        if self.options.input.len() > MAX_IO {
            return Err(Failure::limit("input bytes", MAX_IO));
        }
        if self.options.maximum > MAX_IO {
            return Err(Failure::argument(
                "MaxOutputBytes",
                "Output limit must be in 0..67108864",
            ));
        }
        let mut size = 0;
        configuration(self.command, "Command", &mut size)?;
        for argument in &self.arguments {
            configuration(argument, "Args", &mut size)?;
        }
        if let Some(directory) = self.options.directory {
            configuration(directory, "WorkingDirectory", &mut size)?;
        }
        // Reuse the host's actual environment-key ordering, including Windows
        // ordinal case comparison without Unicode full-case expansions.
        let mut names = Command::new("");
        for (name, value) in &self.options.environment {
            if name.is_empty() || name.contains('=') {
                return Err(Failure::argument(
                    "Environment",
                    "Environment names must be nonempty and contain no '='",
                ));
            }
            configuration(name, "Environment", &mut size)?;
            names.env_remove(name);
            if let Some(value) = value {
                configuration(value, "Environment", &mut size)?;
            }
        }
        if names.get_envs().count() != self.options.environment.len() {
            return Err(Failure::argument(
                "Environment",
                "Environment variable repeated",
            ));
        }
        Ok(())
    }
}

fn configuration(text: &str, name: &'static str, size: &mut usize) -> Result<(), Failure> {
    if text.contains('\0') {
        return Err(Failure::argument(
            name,
            "Process configuration must not contain NUL",
        ));
    }
    *size = size
        .checked_add(text.len())
        .filter(|size| *size <= MAX_CONFIGURATION)
        .ok_or_else(|| Failure::limit("configuration bytes", MAX_CONFIGURATION))?;
    Ok(())
}
