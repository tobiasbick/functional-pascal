//! Help text for the `fpas` command and its subcommands.

use super::types::HelpTopic;

const GENERAL_HELP: &str = "\
fpas — Functional Pascal compiler

Usage:
    fpas build [<path>]                   Build project artifacts
    fpas run [<path>] [-- <args>...]     Run a source, project, workspace, or image
    fpas debug [<path>] --protocol <p>   Debug through JSONL or DAP
    fpas check [<path>]                  Type-check without running
    fpas test [<path>]                   Run `*_test.fpas` programs
    fpas fmt [<path>...]                 Format sources in place

Options:
  -h, --help      Print this help
  -V, --version   Print version

Run `fpas <command> --help` for command-specific options and examples.

Examples:
  fpas build my-app.fpasprj
  fpas run hello.fpas
  fpas debug hello.fpas --protocol jsonl
  fpas check my-app.fpasprj
  fpas test --report json tests/
  fpas fmt --check --list

";

const BUILD_HELP: &str = "\
Build Functional Pascal project artifacts.

Usage:
  fpas build [--std-lib <dir>] [--executable [--name <name>]] [<file.fpasprj | file.fpasworkspace>]

With no path, discovers a `.fpasworkspace` or `.fpasprj` in the current directory.
Program projects produce or reuse `<project.name>.fpascp`. Library projects build
their source-adjacent `.fpascu` files. Workspaces process every member.
`--executable` requires exactly one program and produces a native application
for the current host. `--name` overrides its output base name.

Options:
  --std-lib <dir>  Replace the complete source standard library
  --executable     Bundle one program with the native FPAS runner
  --name <name>    Application/output base name (requires --executable)
  -h, --help       Print this help
  -V, --version    Print version

Examples:
  fpas build my-app.fpasprj
  fpas build --executable my-app.fpasprj
  fpas build --executable --name hello suite.fpasworkspace
  fpas build suite.fpasworkspace
  fpas build

";

const RUN_HELP: &str = "\
Run a Functional Pascal source, project, workspace, or compiled program.

Usage:
  fpas run [--std-lib <dir>] [<file.fpas | file.fpasprj | file.fpasworkspace | file.fpascp>] [-- <args>...]

With no path, discovers the workspace program or `.fpasprj` in the current directory.
Direct `.fpascp` execution does not load sources or the source standard library.

Options:
  --std-lib <dir>  Replace the complete source standard library
  -h, --help       Print this help
  -V, --version    Print version

Examples:
  fpas run hello.fpas
  fpas run my-app.fpasprj -- input.txt verbose
  fpas run suite.fpasworkspace
  fpas run my-app.fpascp
  fpas run --std-lib ./lib hello.fpas

";

const CHECK_HELP: &str = "\
Type-check Functional Pascal sources and projects without running them.

Usage:
  fpas check [--std-lib <dir>] [<file.fpas | dir | file.fpasprj | file.fpasworkspace>]

With no path, discovers a `.fpasworkspace` or `.fpasprj` in the current directory.

Options:
  --std-lib <dir>  Replace the complete source standard library
  -h, --help       Print this help
  -V, --version    Print version

Examples:
  fpas check hello.fpas
  fpas check my-app.fpasprj
  fpas check src/

";

const DEBUG_HELP: &str = "\
Debug a Functional Pascal source, project, workspace, or compiled program.

Usage:
  fpas debug [<target>] --protocol <jsonl | dap> [options] [-- <args>...]

JSONL live mode reads requests from stdin. `--commands` reads the same protocol
from a file. Direct `.fpascp` input requires `--source-root` so embedded source
identities can be verified before launch.

Options:
  --protocol <jsonl | dap>       Select the external debugger protocol
  --commands <path>              Read deterministic JSONL commands from a file
  --source-root <dir>            Root containing sources for a compiled image
  --timeout <secs>               Resume timeout (default: 300)
  --instruction-limit <count>    Session instruction limit (default: 100000000)
  --output-limit <bytes>         Captured output limit (default: 1048576)
  --std-lib <dir>                Replace the complete source standard library
  -h, --help                     Print this help

Examples:
  fpas debug hello.fpas --protocol jsonl
  fpas debug app.fpasprj --protocol jsonl --commands session.jsonl --report jsonl
  fpas debug app.fpascp --source-root . --protocol jsonl

";

const FMT_HELP: &str = "\
Format Functional Pascal sources.

Usage:
  fpas fmt [<path>...]
  fpas fmt --check [--list] [<path>...]
  fpas fmt --stdout <file.fpas>

With no path, discovers a `.fpasworkspace` or `.fpasprj` in the current directory.

Options:
  --check          Exit 2 when formatting would change a file
  --list           With --check, print only paths that would change
  --stdout          Print one formatted file to stdout without modifying it
  -h, --help       Print this help
  -V, --version    Print version

Examples:
  fpas fmt src/main.fpas
  fpas fmt --check --list
  fpas fmt --stdout src/main.fpas > formatted.fpas

";

const TEST_HELP: &str = "\
Run Functional Pascal test programs.

Usage:
  fpas test [--std-lib <dir>] [--list] [--fail-fast] [--strict] [--filter <pattern>] [--report json] [--timeout <secs>] [--jobs <n>] [--script <path>] [<file.fpas | dir | file.fpasprj | file.fpasworkspace>]

With no path, discovers a `.fpasworkspace` or `.fpasprj` in the current directory.

Options:
  --std-lib <dir>     Replace the complete source standard library
  --list              Print discovered tests without running them
  --fail-fast         Stop after the first failing test
  --strict            Treat skipped tests as a failure
  --filter <pattern>  Run matching test paths only
  --report json        Write a machine-readable report to stdout
  --timeout <secs>    Fail a test after a positive number of seconds
  --jobs <n>          Run up to n tests in parallel; 0 uses available CPUs
  --script <path>     Apply test-script overrides
  -h, --help          Print this help
  -V, --version       Print version

Examples:
  fpas test tests/
  fpas test --filter tui --jobs 4 tests/
  fpas test --report json tests/ > test-report.json

";

/// Returns stdout help text for a command or subcommand.
pub(crate) const fn help_text(topic: HelpTopic) -> &'static str {
    match topic {
        HelpTopic::General => GENERAL_HELP,
        HelpTopic::Build => BUILD_HELP,
        HelpTopic::Run => RUN_HELP,
        HelpTopic::Debug => DEBUG_HELP,
        HelpTopic::Check => CHECK_HELP,
        HelpTopic::Fmt => FMT_HELP,
        HelpTopic::Test => TEST_HELP,
    }
}
