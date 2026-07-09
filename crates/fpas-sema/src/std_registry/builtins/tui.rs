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
        s::STD_TUI_APPLICATION_ADD_CHILD => Some(check_add_child(c, args, span)),
        s::STD_TUI_APPLICATION_SET_TEXT => Some(check_set_text(c, args, span)),
        s::STD_TUI_APPLICATION_SET_CHECKED => Some(check_set_checked(c, args, span)),
        s::STD_TUI_APPLICATION_SET_ITEMS => Some(check_set_items(c, args, span)),
        s::STD_TUI_APPLICATION_SET_OUTLINE_NODES => Some(check_set_outline_nodes(c, args, span)),
        s::STD_TUI_APPLICATION_SET_TITLE => Some(check_set_title(c, args, span)),
        s::STD_TUI_APPLICATION_RUN => Some(check_application_run(c, args, span)),
        s::STD_TUI_DIALOG_ADD => Some(check_try2_dialog_add(c, args, span)),
        s::STD_TUI_WINDOW_ADD => Some(check_try2_window_add(c, args, span)),
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
    let outline = lookup_named_type(c, s::STD_TUI_OUTLINE);
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
        && child_ty != outline
        && child_ty != check_box
        && child_ty != radio_button
    {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.AddChild` child must be a button, static text, memo, text viewer, input line, list box, outline, check box, or radio button handle"
                .to_string(),
            "Pass a handle from `Application.CreateButton`, `Application.CreateStaticText`, `Application.CreateMemo`, `Application.CreateTextViewer`, `Application.CreateInputLine`, `Application.CreateListBox`, `Application.CreateOutline`, `Application.CreateCheckBox`, or `Application.CreateRadioButton`.",
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

/// Type-checks `Application.SetChecked(App, Control, Checked)`.
fn check_set_checked(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 3 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!(
                "`{}` expects 3 arguments, got {}",
                s::STD_TUI_APPLICATION_SET_CHECKED,
                args.len()
            ),
            "Example: Application.SetChecked(App, CheckBox, true).",
            span,
        );
        return Ty::Unit;
    }

    let app_ty = c.check_expr(&args[0]);
    let control_ty = c.check_expr(&args[1]);
    let checked_ty = c.check_expr(&args[2]);

    let application = lookup_named_type(c, s::STD_TUI_APPLICATION);
    let check_box = lookup_named_type(c, s::STD_TUI_CHECK_BOX);
    let radio_button = lookup_named_type(c, s::STD_TUI_RADIO_BUTTON);

    if app_ty != application {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.SetChecked` first argument must be an application handle".to_string(),
            "Pass the application handle returned by `Application.Open` or `OpenForTest`.",
            span,
        );
    }

    if control_ty != check_box && control_ty != radio_button {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.SetChecked` control must be a check box or radio button handle"
                .to_string(),
            "Pass a handle from `Application.CreateCheckBox` or `Application.CreateRadioButton`.",
            span,
        );
    }

    if checked_ty != Ty::Boolean {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`Application.SetChecked` checked must be boolean, got {checked_ty}"),
            "Pass `true` or `false`.",
            span,
        );
    }

    Ty::Unit
}

/// Type-checks `Application.SetItems(App, ListBox, Items)`.
fn check_set_items(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 3 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!(
                "`{}` expects 3 arguments, got {}",
                s::STD_TUI_APPLICATION_SET_ITEMS,
                args.len()
            ),
            "Example: Application.SetItems(App, ListBox, ['one', 'two']).",
            span,
        );
        return Ty::Unit;
    }

    let app_ty = c.check_expr(&args[0]);
    let list_ty = c.check_expr(&args[1]);
    let items_ty = c.check_expr(&args[2]);

    let application = lookup_named_type(c, s::STD_TUI_APPLICATION);
    let list_box = lookup_named_type(c, s::STD_TUI_LIST_BOX);

    if app_ty != application {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.SetItems` first argument must be an application handle".to_string(),
            "Pass the application handle returned by `Application.Open` or `OpenForTest`.",
            span,
        );
    }

    if list_ty != list_box {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.SetItems` list must be a list box handle".to_string(),
            "Pass a handle from `Application.CreateListBox`.",
            span,
        );
    }

    if items_ty != Ty::Array(Box::new(Ty::String)) {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`Application.SetItems` items must be array of string, got {items_ty}"),
            "Pass a string array, for example `['one', 'two']`.",
            span,
        );
    }

    Ty::Unit
}

/// Type-checks `Application.SetOutlineNodes(App, Outline, Roots)`.
fn check_set_outline_nodes(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 3 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!(
                "`{}` expects 3 arguments, got {}",
                s::STD_TUI_APPLICATION_SET_OUTLINE_NODES,
                args.len()
            ),
            "Example: Application.SetOutlineNodes(App, Outline, Roots).",
            span,
        );
        return Ty::Unit;
    }

    let app_ty = c.check_expr(&args[0]);
    let outline_ty = c.check_expr(&args[1]);
    let roots_ty = c.check_expr(&args[2]);

    let application = lookup_named_type(c, s::STD_TUI_APPLICATION);
    let outline = lookup_named_type(c, s::STD_TUI_OUTLINE_NODE);
    let outline_handle = lookup_named_type(c, s::STD_TUI_OUTLINE);

    if app_ty != application {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.SetOutlineNodes` first argument must be an application handle"
                .to_string(),
            "Pass the application handle returned by `Application.Open` or `OpenForTest`.",
            span,
        );
    }

    if outline_ty != outline_handle {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.SetOutlineNodes` outline must be an outline handle".to_string(),
            "Pass a handle from `Application.CreateOutline`.",
            span,
        );
    }

    if roots_ty != Ty::Array(Box::new(outline.clone())) {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!(
                "`Application.SetOutlineNodes` roots must be array of OutlineNode, got {roots_ty}"
            ),
            "Pass an array of Std.Tui.OutlineNode records.",
            span,
        );
    }

    Ty::Unit
}

/// Type-checks `Application.SetTitle(App, Root, Title)`.
fn check_set_title(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 3 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!(
                "`{}` expects 3 arguments, got {}",
                s::STD_TUI_APPLICATION_SET_TITLE,
                args.len()
            ),
            "Example: Application.SetTitle(App, Dialog, 'New title').",
            span,
        );
        return Ty::Unit;
    }

    let app_ty = c.check_expr(&args[0]);
    let root_ty = c.check_expr(&args[1]);
    let title_ty = c.check_expr(&args[2]);

    let application = lookup_named_type(c, s::STD_TUI_APPLICATION);
    let dialog = lookup_named_type(c, s::STD_TUI_DIALOG);
    let window = lookup_named_type(c, s::STD_TUI_WINDOW);

    if app_ty != application {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.SetTitle` first argument must be an application handle".to_string(),
            "Pass the application handle returned by `Application.Open` or `OpenForTest`.",
            span,
        );
    }

    if root_ty != dialog && root_ty != window {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`Application.SetTitle` root must be a dialog or window handle".to_string(),
            "Pass a handle from `Application.CreateDialog` or `Application.CreateWindow`.",
            span,
        );
    }

    if title_ty != Ty::String {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`Application.SetTitle` title must be a string, got {title_ty}"),
            "Pass a string value as the new title.",
            span,
        );
    }

    Ty::Unit
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

/// Registers polymorphic `Application.AddChild` and property-setter placeholders.
pub(crate) fn register_tui_builtins(checker: &mut Checker) {
    for name in [
        s::STD_TUI_APPLICATION_ADD_CHILD,
        s::STD_TUI_APPLICATION_SET_TEXT,
        s::STD_TUI_APPLICATION_SET_CHECKED,
        s::STD_TUI_APPLICATION_SET_ITEMS,
        s::STD_TUI_APPLICATION_SET_OUTLINE_NODES,
        s::STD_TUI_APPLICATION_SET_TITLE,
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
