//! Explicit host adapters retain the application's separate response right.
use super::super::super::{define_func, p};
use crate::{check::Checker, types::Ty};
fn named(checker: &Checker, name: &str) -> Ty {
    checker
        .scopes
        .imports
        .lookup_metadata(name)
        .unwrap_or_else(|| panic!("ACP prerequisite {name}"))
        .ty
        .clone()
}
fn option(ty: Ty) -> Ty {
    Ty::Option(Box::new(ty))
}
pub(super) fn register(checker: &mut Checker) {
    let error = named(checker, "Std.Acp.Types.AcpError");
    for (name, request, response) in [
        (
            "ReadTextFile",
            "ReadTextFileRequest",
            "ReadTextFileResponse",
        ),
        (
            "WriteTextFile",
            "WriteTextFileRequest",
            "WriteTextFileResponse",
        ),
        (
            "CreateTerminal",
            "CreateTerminalRequest",
            "CreateTerminalResponse",
        ),
        (
            "TerminalOutput",
            "TerminalOutputRequest",
            "TerminalOutputResponse",
        ),
        (
            "WaitForTerminalExit",
            "WaitForTerminalExitRequest",
            "WaitForTerminalExitResponse",
        ),
        (
            "KillTerminal",
            "KillTerminalRequest",
            "KillTerminalResponse",
        ),
        (
            "ReleaseTerminal",
            "ReleaseTerminalRequest",
            "ReleaseTerminalResponse",
        ),
        ("ValidateElicitation", "CreateElicitationResponse", ""),
    ] {
        let mut params = vec![
            p("Responder", named(checker, "Std.Acp.Responder")),
            p(
                "Request",
                named(checker, &format!("Std.Acp.Types.{request}")),
            ),
        ];
        if name == "WaitForTerminalExit" {
            params.push(p("TimeoutMillis", option(Ty::Integer)));
            params.push(p(
                "CancellationToken",
                option(named(checker, "Std.Tasks.CancellationToken")),
            ));
        }
        let response = if response.is_empty() {
            Ty::Boolean
        } else {
            named(checker, &format!("Std.Acp.Types.{response}"))
        };
        define_func(
            checker,
            &format!("Std.Acp.Host.{name}"),
            params,
            Ty::Result(Box::new(response), Box::new(error.clone())),
        );
    }
    define_func(
        checker,
        "Std.Acp.Host.ElicitationDefaults",
        vec![p("Responder", named(checker, "Std.Acp.Responder"))],
        Ty::Result(
            Box::new(named(checker, "Std.Acp.Types.RawJson")),
            Box::new(error.clone()),
        ),
    );
    for (name, family) in [
        ("RunTerminalAuthentication", "Types"),
        ("V2RunTerminalAuthentication", "V2"),
    ] {
        define_func(
            checker,
            &format!("Std.Acp.Host.{name}"),
            vec![
                p("Launch", named(checker, "Std.Acp.Launch")),
                p(
                    "Method",
                    named(checker, &format!("Std.Acp.{family}.AuthMethodTerminal")),
                ),
                p("TimeoutMillis", option(Ty::Integer)),
                p(
                    "CancellationToken",
                    option(named(checker, "Std.Tasks.CancellationToken")),
                ),
            ],
            Ty::Result(Box::new(Ty::Boolean), Box::new(error.clone())),
        );
    }
}
