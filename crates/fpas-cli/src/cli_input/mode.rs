//! CLI subcommand mode detection and shared argv helpers.

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CliMode {
    Run,
    Check,
    Fmt,
    Test,
}

const SUBCOMMANDS: &str = "`run`, `check`, `test`, or `fmt`";

pub(super) fn parse_cli_mode(cli_args: &[String]) -> Result<(CliMode, &[String]), String> {
    let Some(first) = cli_args.first() else {
        return Err(missing_subcommand_error());
    };

    match first.as_str() {
        "check" => Ok((CliMode::Check, &cli_args[1..])),
        "fmt" => Ok((CliMode::Fmt, &cli_args[1..])),
        "test" => Ok((CliMode::Test, &cli_args[1..])),
        "run" => Ok((CliMode::Run, &cli_args[1..])),
        _ => Err(unexpected_cli_token_error(first)),
    }
}

pub(super) fn usage_error(mode: CliMode) -> String {
    match mode {
        CliMode::Run => {
            "Usage: fpas run [<file.fpas | file.fpasprj>] [-- <args>...]\n  help: `fpas --help` shows options."
                .to_string()
        }
        CliMode::Check => {
            "Usage: fpas check [<file.fpas | dir | file.fpasprj | file.fpasworkspace>]\n  help: `fpas --help` shows options."
                .to_string()
        }
        CliMode::Fmt => {
            "Usage: fpas fmt [--check] [--list] [--stdout] [<path>...]\n  help: `fpas --help` shows options."
                .to_string()
        }
        CliMode::Test => {
            "Usage: fpas test [--list] [--fail-fast] [--filter <pattern>] [--report json] [--timeout <secs>] [--jobs <n>] [--script <path>] [<file.fpas | dir | file.fpasprj | file.fpasworkspace>]\n  help: `fpas --help` shows options."
                .to_string()
        }
    }
}

pub(super) fn missing_subcommand_error() -> String {
    format!("Missing subcommand. Expected {SUBCOMMANDS}.\n  help: `fpas --help` lists commands.")
}

pub(super) fn program_args_require_run_error() -> String {
    "Program arguments after `--` require `fpas run`.\n  help: `fpas run [<file.fpas | file.fpasprj>] -- <args>...`"
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
