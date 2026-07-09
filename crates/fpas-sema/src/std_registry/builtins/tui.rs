//! Semantic checking for polymorphic `Std.Tui` builtins.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use super::super::p;
use crate::check::Checker;
use crate::types::{ProcedureTy, Ty};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT};
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

/// Type-checks a `Std.Tui` [`SymbolKind::BuiltinStd`] call when `name` matches.
pub(super) fn check_tui_builtin_std_call(
    c: &mut Checker,
    name: &str,
    args: &[Expr],
    span: Span,
) -> Option<Ty> {
    match name {
        s::STD_TUI_APPLICATION_RUN => Some(check_application_run(c, args, span)),
        s::STD_TUI_DIALOG_ADD => Some(check_try2_dialog_add(c, args, span)),
        s::STD_TUI_WINDOW_ADD => Some(check_try2_window_add(c, args, span)),
        _ => None,
    }
}

fn check_application_run(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    let application = lookup_named_type(c, s::STD_TUI_APPLICATION);
    let on_command = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application.clone(), false),
            p("CommandId", Ty::Integer, false),
        ],
        variadic: false,
    });

    match args.len() {
        1 => {
            let app_ty = c.check_expr(&args[0]);
            if app_ty != application {
                c.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    "`Application.Run` first argument must be an application handle".to_string(),
                    "Pass the application handle returned by `Application.Open` or `OpenForTest`.",
                    span,
                );
            }
        }
        2 => {
            let app_ty = c.check_expr(&args[0]);
            let handler_ty = c.check_expr(&args[1]);
            if app_ty != application {
                c.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    "`Application.Run` first argument must be an application handle".to_string(),
                    "Pass the application handle returned by `Application.Open` or `OpenForTest`.",
                    span,
                );
            }
            if !on_command.compatible_with(&handler_ty) {
                c.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    "`Application.Run` OnCommand handler must be `procedure (Application, integer)`"
                        .to_string(),
                    "Pass a command handler such as `procedure OnCommand(App: Application; Cmd: integer)`.",
                    span,
                );
            }
        }
        count => {
            c.error_with_code(
                SEMA_WRONG_ARGUMENT_COUNT,
                format!("`Application.Run` expects 1 or 2 arguments, got {count}"),
                "Use `Application.Run(App)` or `Application.Run(App, OnCommand)`.",
                span,
            );
            c.check_args_only(args);
        }
    }

    Ty::Unit
}

fn check_try2_dialog_add(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 2 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!("`Dialog.Add` expects 2 arguments, got {}", args.len()),
            "Example: Dialog.Add(Dlg, Btn) or Dialog.Add(Dlg, Label).",
            span,
        );
        return Ty::Unit;
    }

    let dialog = lookup_named_type(c, s::STD_TUI_DIALOG);
    let button = lookup_named_type(c, s::STD_TUI_BUTTON);
    let static_text = lookup_named_type(c, s::STD_TUI_STATIC_TEXT);
    let check_box = lookup_named_type(c, s::STD_TUI_CHECK_BOX);
    let input_line = lookup_named_type(c, s::STD_TUI_INPUT_LINE);
    let list_box = lookup_named_type(c, s::STD_TUI_LIST_BOX);
    let outline = lookup_named_type(c, s::STD_TUI_OUTLINE);
    let radio_button = lookup_named_type(c, s::STD_TUI_RADIO_BUTTON);
    let memo = lookup_named_type(c, s::STD_TUI_MEMO);
    let text_viewer = lookup_named_type(c, s::STD_TUI_TEXT_VIEWER);

    let dlg_ty = c.check_expr(&args[0]);
    let child_ty = c.check_expr(&args[1]);

    if dlg_ty != dialog {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Dialog.Add` first argument must be a dialog handle".to_string(),
            "Pass a handle from `Dialog.NewModal`.",
            span,
        );
    }

    if child_ty != button
        && child_ty != static_text
        && child_ty != check_box
        && child_ty != input_line
        && child_ty != list_box
        && child_ty != outline
        && child_ty != radio_button
        && child_ty != memo
        && child_ty != text_viewer
    {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Dialog.Add` child must be a button, static text, check box, input line, list box, outline, radio button, memo, or text viewer handle"
                .to_string(),
            "Pass a handle from `Button.New`, `StaticText.New`, `CheckBox.New`, `InputLine.New`, `ListBox.New`, `Outline.New`, `RadioButton.New`, `Memo.New`, or `TextViewer.New`.",
            span,
        );
    }

    Ty::Unit
}

fn check_try2_window_add(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 2 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!("`Window.Add` expects 2 arguments, got {}", args.len()),
            "Example: Window.Add(Win, Btn) or Window.Add(Win, Label).",
            span,
        );
        return Ty::Unit;
    }

    let window = lookup_named_type(c, s::STD_TUI_WINDOW);
    let button = lookup_named_type(c, s::STD_TUI_BUTTON);
    let static_text = lookup_named_type(c, s::STD_TUI_STATIC_TEXT);
    let check_box = lookup_named_type(c, s::STD_TUI_CHECK_BOX);
    let input_line = lookup_named_type(c, s::STD_TUI_INPUT_LINE);
    let list_box = lookup_named_type(c, s::STD_TUI_LIST_BOX);
    let outline = lookup_named_type(c, s::STD_TUI_OUTLINE);
    let radio_button = lookup_named_type(c, s::STD_TUI_RADIO_BUTTON);
    let memo = lookup_named_type(c, s::STD_TUI_MEMO);
    let text_viewer = lookup_named_type(c, s::STD_TUI_TEXT_VIEWER);

    let win_ty = c.check_expr(&args[0]);
    let child_ty = c.check_expr(&args[1]);

    if win_ty != window {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Window.Add` first argument must be a window handle".to_string(),
            "Pass a handle from `Window.New`.",
            span,
        );
    }

    if child_ty != button
        && child_ty != static_text
        && child_ty != check_box
        && child_ty != input_line
        && child_ty != list_box
        && child_ty != outline
        && child_ty != radio_button
        && child_ty != memo
        && child_ty != text_viewer
    {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Window.Add` child must be a button, static text, check box, input line, list box, outline, radio button, memo, or text viewer handle"
                .to_string(),
            "Pass a handle from `Button.New`, `StaticText.New`, `CheckBox.New`, `InputLine.New`, `ListBox.New`, `Outline.New`, `RadioButton.New`, `Memo.New`, or `TextViewer.New`.",
            span,
        );
    }

    Ty::Unit
}

/// Registers polymorphic `Application.Run`, `Dialog.Add`, and `Window.Add` builtins.
pub(crate) fn register_tui_builtins(checker: &mut Checker) {
    for name in [
        s::STD_TUI_APPLICATION_RUN,
        s::STD_TUI_DIALOG_ADD,
        s::STD_TUI_WINDOW_ADD,
    ] {
        super::super::define_builtin_std(
            checker,
            name,
            Ty::Procedure(ProcedureTy {
                type_params: Vec::new(),
                params: Vec::new(),
                variadic: false,
            }),
        );
    }
}

fn lookup_named_type(c: &Checker, name: &str) -> Ty {
    c.scopes
        .lookup(name)
        .map(|symbol| symbol.ty.clone())
        .unwrap_or(Ty::Error)
}
