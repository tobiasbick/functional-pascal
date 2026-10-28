//! Prepared runtime strings follow successful image changes and rollback.

use super::*;

fn images() -> (VerifiedExecutable, VerifiedExecutable) {
    let source = "program P;\nuses Std.Console as Console;\nfunction Text(): string;\nbegin\n  return 'before';\nend function;\nbegin\n  Console.WriteLn(Text());\nend.";
    let compile = |source: &str| {
        let (program, diagnostics) = fpas_parser::parse(source);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        fpas_compiler::compile(&program).expect("string constant image")
    };
    (
        compile(source),
        compile(&source.replace("'before'", "'après!'")),
    )
}

#[test]
fn replaced_and_rolled_back_images_load_their_own_strings() {
    for rollback in [false, true] {
        let (current, candidate) = images();
        let mut session = DebugSession::new(current).expect("session");
        assert!(
            session
                .replace_live_image(&candidate)
                .expect("replace")
                .applied
        );
        if rollback {
            assert!(session.rollback_live_image().expect("rollback").applied);
        }
        assert!(matches!(
            session.continue_execution().expect("run"),
            DebugRunResult::Terminated(_)
        ));
        assert_eq!(
            session.output().lines,
            [if rollback { "before" } else { "après!" }]
        );
    }
}

#[test]
fn rejected_image_keeps_the_prepared_strings() {
    let (current, candidate) = images();
    let mut session = DebugSession::new(current).expect("session");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".into(),
            line: 5,
            column: None,
        })
        .expect("helper breakpoint");
    stopped(session.continue_execution().expect("stop in helper"));
    assert_eq!(
        session
            .replace_live_image(&candidate)
            .expect_err("active body")
            .kind,
        DebugErrorKind::LiveImageIncompatible,
    );
    assert!(matches!(
        session.continue_execution().expect("run original image"),
        DebugRunResult::Terminated(_)
    ));
    assert_eq!(session.output().lines, ["before"]);
}
