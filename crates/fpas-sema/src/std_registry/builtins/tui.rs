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
    if name != s::STD_TUI_APPLICATION_ADD_CHILD {
        return None;
    }

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
        return Some(Ty::Unit);
    }

    let app_ty = c.check_expr(&args[0]);
    let parent_ty = c.check_expr(&args[1]);
    let child_ty = c.check_expr(&args[2]);

    let application = lookup_named_type(c, s::STD_TUI_APPLICATION);
    let dialog = lookup_named_type(c, s::STD_TUI_DIALOG);
    let window = lookup_named_type(c, s::STD_TUI_WINDOW);
    let button = lookup_named_type(c, s::STD_TUI_BUTTON);
    let static_text = lookup_named_type(c, s::STD_TUI_STATIC_TEXT);
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
        && child_ty != input_line
        && child_ty != list_box
        && child_ty != check_box
        && child_ty != radio_button
    {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.AddChild` child must be a button, static text, input line, list box, check box, or radio button handle"
                .to_string(),
            "Pass a handle from `Application.CreateButton`, `Application.CreateStaticText`, `Application.CreateInputLine`, `Application.CreateListBox`, `Application.CreateCheckBox`, or `Application.CreateRadioButton`.",
            span,
        );
    }

    Some(Ty::Unit)
}

/// Registers the polymorphic `Application.AddChild` builtin placeholder.
pub(crate) fn register_add_child_builtin(checker: &mut Checker) {
    super::super::define_builtin_std(
        checker,
        s::STD_TUI_APPLICATION_ADD_CHILD,
        Ty::Procedure(ProcedureTy {
            type_params: Vec::new(),
            params: Vec::new(),
            variadic: false,
        }),
    );
}

fn lookup_named_type(c: &Checker, name: &str) -> Ty {
    c.scopes
        .lookup(name)
        .map(|symbol| symbol.ty.clone())
        .unwrap_or(Ty::Error)
}
