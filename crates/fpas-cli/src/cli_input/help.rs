//! Help text for `fpas -h` / `fpas --help`.

/// Text printed for `fpas -h` / `fpas --help` (stdout).
pub(crate) const CLI_HELP: &str = "\
fpas — Functional Pascal compiler

Usage:
    fpas run [--std-lib <dir>] [<file.fpas | file.fpasprj>] [-- <args>...]   Run a source file or project
    fpas run [-- <args>...]                               Discover a workspace program or `.fpasprj` in cwd
    fpas check [--std-lib <dir>] [<file.fpas | dir | file.fpasprj | file.fpasworkspace>]
    fpas test [--std-lib <dir>] [<file.fpas | dir | file.fpasprj | file.fpasworkspace>]
                                                          Type-check without running
    fpas check                                            Discover `.fpasworkspace` or `.fpasprj` in cwd
    fpas test [<file.fpas | dir | file.fpasprj | file.fpasworkspace>]
                                                          Run `*_test.fpas` programs
    fpas test [--list] [--fail-fast] [--strict] [--filter <pattern>] [--report json] [--timeout <secs>] [--jobs <n>] [--script <path>] [<path>]             Discover tests in cwd when path omitted
    fpas fmt [<path>...]                                  Format sources in place (multiple paths ok)
    fpas fmt [--check] [--list] [<path>...]               Check formatting (exit 2 if changes needed)
    fpas fmt --stdout <file.fpas>                         Print formatted text to stdout (one file)
    fpas fmt                                              Discover `.fpasworkspace` or `.fpasprj` in cwd

Options:
  -h, --help      Print this help
  -V, --version   Print version

Program arguments after `--` are visible through `Std.Args` when running programs.
`Std.*` source units are loaded through `lib/stdlib.fpasprj` beside `fpas`; `--std-lib <dir>` replaces that complete library for `run`, `check`, and `test`.

";
