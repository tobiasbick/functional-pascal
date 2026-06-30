//! Headless test event injection for `Std.Tui`.
//!
//! **Documentation:** `docs/pascal/std/tui/app/testing.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{SourceLocation, TuiIntrinsic, Value};
use fpas_diagnostics::codes::{RUNTIME_CONSOLE_STATE_ERROR, RUNTIME_INTRINSIC_STACK_STATE_ERROR};
use fpas_std::ConsoleEvent;
use fpas_std::{event_kind_index, mouse_action_index, mouse_button_index};

impl Worker {
    pub(super) fn exec_tui_test_event_intrinsic(
        &mut self,
        intrinsic: TuiIntrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            TuiIntrinsic::TestSendKey => {
                let key = self.pop_console_key_event(line)?;
                self.pop_tui_application(line)?;
                self.enqueue_test_console_event(ConsoleEvent::key(key));
            }
            TuiIntrinsic::TestSendMouse => {
                let event = self.pop_console_event(line)?;
                Self::validate_test_mouse_event(&event, line)?;
                self.pop_tui_application(line)?;
                self.enqueue_test_console_event(event);
            }
            TuiIntrinsic::TestMoveMouse => {
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                self.enqueue_test_console_event(ConsoleEvent::mouse(
                    mouse_action_index("Move"),
                    mouse_button_index("None"),
                    x,
                    y,
                    false,
                    false,
                    false,
                    false,
                ));
            }
            TuiIntrinsic::TestClickMouse => {
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                self.enqueue_test_console_event(ConsoleEvent::mouse(
                    mouse_action_index("Down"),
                    mouse_button_index("Left"),
                    x,
                    y,
                    false,
                    false,
                    false,
                    false,
                ));
                self.enqueue_test_console_event(ConsoleEvent::mouse(
                    mouse_action_index("Up"),
                    mouse_button_index("Left"),
                    x,
                    y,
                    false,
                    false,
                    false,
                    false,
                ));
            }
            TuiIntrinsic::TestResize => {
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                self.pop_tui_application(line)?;
                self.enqueue_test_console_event(ConsoleEvent::resize(width, height));
            }
            TuiIntrinsic::TestPaste => {
                let text = match self.pop(line)? {
                    Value::Str(text) => text,
                    other => {
                        return Err(runtime_error(
                            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                            format!(
                                "Application.TestPaste(App, Text) expected string, got {}",
                                other.type_name()
                            ),
                            "Pass a string literal or variable, e.g. `Application.TestPaste(App, \"hello\")`.",
                            line,
                        ));
                    }
                };
                self.pop_tui_application(line)?;
                self.enqueue_test_console_event(ConsoleEvent::paste(text));
            }
            TuiIntrinsic::TestFocus => {
                let gained = match self.pop(line)? {
                    Value::Boolean(gained) => gained,
                    other => {
                        return Err(runtime_error(
                            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                            format!(
                                "Application.TestFocus(App, Gained) expected boolean, got {}",
                                other.type_name()
                            ),
                            "Pass `true` for focus gained or `false` for focus lost.",
                            line,
                        ));
                    }
                };
                self.pop_tui_application(line)?;
                self.enqueue_test_console_event(if gained {
                    ConsoleEvent::focus_gained()
                } else {
                    ConsoleEvent::focus_lost()
                });
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    pub(in crate::vm::execute::io::tui) fn enqueue_test_console_event(&self, event: ConsoleEvent) {
        self.with_key_input(|key_input| {
            key_input.push_console_event(event);
        });
    }

    fn validate_test_mouse_event(
        event: &ConsoleEvent,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        if event.kind == event_kind_index("Mouse") {
            Ok(())
        } else {
            Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Application.TestSendMouse(App, Event) requires a mouse event.",
                "Set `Event.kind` to `Std.Console.EventKind.Mouse` and fill the mouse fields.",
                line,
            ))
        }
    }
}
