//! Source and function address lookup follows committed images and rollback.

use super::*;

#[test]
fn mixed_breakpoint_addresses_follow_reload_and_rollback() {
    for rollback in [false, true] {
        let current = pair_executable();
        let candidate = patch_helper_with_shifted_sequence_point(&current);
        let mut session = DebugSession::new(current).expect("debug session");
        let function = session
            .replace_function_breakpoints(vec![FunctionBreakpoint {
                name: "helper".into(),
            }])
            .expect("function breakpoint")[0]
            .id;
        let source = session
            .set_breakpoint(SourceBreakpoint {
                source: "test.fpas".into(),
                line: 10,
                column: None,
            })
            .expect("source breakpoint")
            .id;
        assert!(
            session
                .replace_live_image(&candidate)
                .expect("reload")
                .applied
        );
        if rollback {
            assert!(session.rollback_live_image().expect("rollback").applied);
        }

        let stop = stopped(session.continue_execution().expect("rebound stop"));
        assert_eq!(stop.reason, DebugStopReason::Breakpoint);
        assert_eq!(stop.instruction, if rollback { 4 } else { 5 });
        assert_eq!(stop.location.expect("source location").line, 10);
        assert_eq!(stop.breakpoint_ids, [function, source]);
    }
}
