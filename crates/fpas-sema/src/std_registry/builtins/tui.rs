//! Semantic checking for polymorphic `Std.Tui` builtins.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

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
        s::STD_TUI_APPLICATION_ADD_CHILD => Some(check_add_child(c, args, span)),
        s::STD_TUI_APPLICATION_SET_TEXT => Some(check_set_text(c, args, span)),
        _ => None,
    }
}

fn check_add_child(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 3 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!(
                "`{}` expects 3 arguments, got {}",
                s::STD_TUI_APPLICATION_ADD_CHILD,
                args.len()
            ),
            "Example: Application.AddChild(App, Parent, Child).",
            span,
        );
        return Ty::Unit;
    }

    let app_ty = c.check_expr(&args[0]);
    let parent_ty = c.check_expr(&args[1]);
    let child_ty = c.check_expr(&args[2]);

    let application = lookup_named_type(c, s::STD_TUI_APPLICATION);
    let dialog = lookup_named_type(c, s::STD_TUI_DIALOG);
    let window = lookup_named_type(c, s::STD_TUI_WINDOW);
    let button = lookup_named_type(c, s::STD_TUI_BUTTON);
    let static_text = lookup_named_type(c, s::STD_TUI_STATIC_TEXT);
    let memo = lookup_named_type(c, s::STD_TUI_MEMO);
    let text_viewer = lookup_named_type(c, s::STD_TUI_TEXT_VIEWER);
    let input_line = lookup_named_type(c, s::STD_TUI_INPUT_LINE);
    let list_box = lookup_named_type(c, s::STD_TUI_LIST_BOX);
    let check_box = lookup_named_type(c, s::STD_TUI_CHECK_BOX);
    let radio_button = lookup_named_type(c, s::STD_TUI_RADIO_BUTTON);

    if app_ty != application {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.AddChild` first argument must be an application handle".to_string(),
            "Pass the application handle returned by `Application.Open` or `OpenForTest`.",
            span,
        );
    }

    if parent_ty != dialog && parent_ty != window {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.AddChild` parent must be a dialog or window handle".to_string(),
            "Pass a handle from `Application.CreateDialog` or `Application.CreateWindow`.",
            span,
        );
    }

    if child_ty != button
        && child_ty != static_text
        && child_ty != memo
        && child_ty != text_viewer
        && child_ty != input_line
        && child_ty != list_box
        && child_ty != check_box
        && child_ty != radio_button
    {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.AddChild` child must be a button, static text, memo, text viewer, input line, list box, check box, or radio button handle"
                .to_string(),
            "Pass a handle from `Application.CreateButton`, `Application.CreateStaticText`, `Application.CreateMemo`, `Application.CreateTextViewer`, `Application.CreateInputLine`, `Application.CreateListBox`, `Application.CreateCheckBox`, or `Application.CreateRadioButton`.",
            span,
        );
    }

    Ty::Unit
}

/// Type-checks `Application.SetText(App, Control, Text)`.
///
/// `Control` must be a text-bearing control handle (button, static text, memo,
/// text viewer, input line, check box, or radio button). List boxes carry an
/// item array, not a single text, so they are rejected.
fn check_set_text(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 3 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!(
                "`{}` expects 3 arguments, got {}",
                s::STD_TUI_APPLICATION_SET_TEXT,
                args.len()
            ),
            "Example: Application.SetText(App, Control, 'new text').",
            span,
        );
        return Ty::Unit;
    }

    let app_ty = c.check_expr(&args[0]);
    let control_ty = c.check_expr(&args[1]);
    let text_ty = c.check_expr(&args[2]);

    let application = lookup_named_type(c, s::STD_TUI_APPLICATION);
    let button = lookup_named_type(c, s::STD_TUI_BUTTON);
    let static_text = lookup_named_type(c, s::STD_TUI_STATIC_TEXT);
    let memo = lookup_named_type(c, s::STD_TUI_MEMO);
    let text_viewer = lookup_named_type(c, s::STD_TUI_TEXT_VIEWER);
    let input_line = lookup_named_type(c, s::STD_TUI_INPUT_LINE);
    let check_box = lookup_named_type(c, s::STD_TUI_CHECK_BOX);
    let radio_button = lookup_named_type(c, s::STD_TUI_RADIO_BUTTON);

    if app_ty != application {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.SetText` first argument must be an application handle".to_string(),
            "Pass the application handle returned by `Application.Open` or `OpenForTest`.",
            span,
        );
    }

    if control_ty != button
        && control_ty != static_text
        && control_ty != memo
        && control_ty != text_viewer
        && control_ty != input_line
        && control_ty != check_box
        && control_ty != radio_button
    {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.SetText` control must be a button, static text, memo, text viewer, input line, check box, or radio button handle"
                .to_string(),
            "Pass a handle from `Application.CreateButton`, `Application.CreateStaticText`, `Application.CreateMemo`, `Application.CreateTextViewer`, `Application.CreateInputLine`, `Application.CreateCheckBox`, or `Application.CreateRadioButton`.",
            span,
        );
    }

    if text_ty != Ty::String {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`Application.SetText` text must be a string, got {text_ty}"),
            "Pass a string value as the new control text.",
            span,
        );
    }

    Ty::Unit
}

/// Registers the polymorphic `Application.AddChild` and `Application.SetText` placeholders.
pub(crate) fn register_tui_builtins(checker: &mut Checker) {
    for name in [
        s::STD_TUI_APPLICATION_ADD_CHILD,
        s::STD_TUI_APPLICATION_SET_TEXT,
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
