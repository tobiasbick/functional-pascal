//! CLI subcommand mode detection and shared argv helpers.

use super::types::HelpTopic;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CliMode {
    Init,
    Build,
    Run,
    Debug,
    Env,
    Lsp,
    Check,
    Fmt,
    Test,
}

impl CliMode {
    pub(super) const fn help_topic(self) -> HelpTopic {
        match self {
            Self::Init => HelpTopic::Init,
            Self::Build => HelpTopic::Build,
            Self::Run => HelpTopic::Run,
            Self::Debug => HelpTopic::Debug,
            Self::Env => HelpTopic::Env,
            Self::Lsp => HelpTopic::Lsp,
            Self::Check => HelpTopic::Check,
            Self::Fmt => HelpTopic::Fmt,
            Self::Test => HelpTopic::Test,
        }
    }

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::Build => "build",
            Self::Run => "run",
            Self::Debug => "debug",
            Self::Env => "env",
            Self::Lsp => "lsp",
            Self::Check => "check",
            Self::Fmt => "fmt",
            Self::Test => "test",
        }
    }
}

const SUBCOMMANDS: &str =
    "`init`, `build`, `run`, `debug`, `check`, `test`, `fmt`, `env`, or `lsp`";

pub(super) fn parse_cli_mode(cli_args: &[String]) -> Result<(CliMode, &[String]), String> {
    let Some(first) = cli_args.first() else {
        return Err(missing_subcommand_error());
    };

    match first.as_str() {
        "init" => Ok((CliMode::Init, &cli_args[1..])),
        "build" => Ok((CliMode::Build, &cli_args[1..])),
        "check" => Ok((CliMode::Check, &cli_args[1..])),
        "fmt" => Ok((CliMode::Fmt, &cli_args[1..])),
        "test" => Ok((CliMode::Test, &cli_args[1..])),
        "run" => Ok((CliMode::Run, &cli_args[1..])),
        "debug" => Ok((CliMode::Debug, &cli_args[1..])),
        "env" => Ok((CliMode::Env, &cli_args[1..])),
        "lsp" => Ok((CliMode::Lsp, &cli_args[1..])),
        _ => Err(unexpected_cli_token_error(first)),
    }
}

pub(super) fn usage_error(mode: CliMode) -> String {
    match mode {
        CliMode::Init => {
            "Usage: fpas init <project | library | workspace> <name> [options]\n  help: `fpas init --help` shows scaffold kinds and examples."
                .to_string()
        }
        CliMode::Build => {
            "Usage: fpas build [--std-lib <dir>] [--executable [--name <name>]] [<file.fpasprj | file.fpasworkspace>]\n  help: `fpas build --help` shows options and examples."
                .to_string()
        }
        CliMode::Run => {
            "Usage: fpas run [<file.fpas | file.fpasprj | file.fpasworkspace | file.fpascp>] [-- <args>...]\n  help: `fpas run --help` shows options and examples."
                .to_string()
        }
        CliMode::Debug => {
            "Usage: fpas debug [<file.fpas | file.fpasprj | file.fpasworkspace | file.fpascp>] --protocol <jsonl | dap> [-- <args>...]\n  help: `fpas debug --help` shows options and examples."
                .to_string()
        }
        CliMode::Env => {
            "Usage: fpas env --json\n  help: `fpas env --help` shows the machine-readable toolchain contract."
                .to_string()
        }
        CliMode::Lsp => {
            "Usage: fpas lsp\n  help: `fpas lsp --help` describes the standard-I/O server."
                .to_string()
        }
        CliMode::Check => {
            "Usage: fpas check [<file.fpas | dir | file.fpasprj | file.fpasworkspace>]\n  help: `fpas check --help` shows options and examples."
                .to_string()
        }
        CliMode::Fmt => {
            "Usage: fpas fmt [--check] [--list] [--stdout] [<path>...]\n  help: `fpas fmt --help` shows options and examples."
                .to_string()
        }
        CliMode::Test => {
            "Usage: fpas test [--list] [--fail-fast] [--filter <pattern>] [--report json] [--timeout <secs>] [--jobs <n>] [--script <path>] [<file.fpas | dir | file.fpasprj | file.fpasworkspace>]\n  help: `fpas test --help` shows options and examples."
                .to_string()
        }
    }
}

pub(super) fn missing_subcommand_error() -> String {
    format!("Missing subcommand. Expected {SUBCOMMANDS}.\n  help: `fpas --help` lists commands.")
}

pub(super) fn program_args_require_run_error() -> String {
    "Program arguments after `--` require `fpas run` or `fpas debug`.\n  help: `fpas debug hello.fpas --protocol jsonl -- <args>...`"
        .to_string()
}

fn unexpected_cli_token_error(token: &str) -> String {
    if token.ends_with(".fpas") || token.ends_with(".fpasprj") {
        format!(
            "Expected a subcommand before `{token}`. Use `fpas run {token}` to execute a program.\n  help: `fpas --help` lists commands."
        )
    } else {
        format!(
            "Unknown subcommand `{token}`. Expected {SUBCOMMANDS}.\n  help: `fpas --help` lists commands."
        )
    }
}

pub(super) fn split_program_args(args: &[String]) -> (&[String], Vec<String>) {
    let Some(separator) = args.iter().position(|arg| arg == "--") else {
        return (args, Vec::new());
    };

    (&args[..separator], args[separator + 1..].to_vec())
}
