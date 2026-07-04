//! CLI subcommand mode detection and shared argv helpers.

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CliMode {
    Run,
    Check,
    Fmt,
    Test,
}

pub(super) fn parse_cli_mode(cli_args: &[String]) -> Result<(CliMode, &[String]), String> {
    if cli_args.first().is_some_and(|arg| arg == "check") {
        return Ok((CliMode::Check, &cli_args[1..]));
    }
    if cli_args.first().is_some_and(|arg| arg == "fmt") {
        return Ok((CliMode::Fmt, &cli_args[1..]));
    }
    if cli_args.first().is_some_and(|arg| arg == "test") {
        return Ok((CliMode::Test, &cli_args[1..]));
    }

    Ok((CliMode::Run, cli_args))
}

pub(super) fn usage_error(mode: CliMode) -> String {
    match mode {
        CliMode::Run => {
            "Usage: fpas [<file.fpas | file.fpasprj>] [-- <args>...]\n  help: `fpas --help` shows options."
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

pub(super) fn split_program_args(args: &[String]) -> (&[String], Vec<String>) {
    let Some(separator) = args.iter().position(|arg| arg == "--") else {
        return (args, Vec::new());
    };

    (&args[..separator], args[separator + 1..].to_vec())
}
