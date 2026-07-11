use super::super::super::{define_builtin_std, define_func, define_proc, p};
use super::{TuiCallbackTypes, TuiTypes};
use crate::check::Checker;
use crate::types::{ProcedureTy, Ty};
use fpas_std::std_symbols as s;

/// Register try-2 `Std.Tui` view and application symbols.
///
/// **Documentation:** `docs/refactor-tui-try-2/target-api.md`
pub(super) fn register_try2_api(
    checker: &mut Checker,
    types: &TuiTypes,
    _callbacks: &TuiCallbackTypes,
) {
    for name in [
        s::STD_TUI_CM_OK,
        s::STD_TUI_CM_CANCEL,
        s::STD_TUI_CM_CLOSE,
        s::STD_TUI_CM_QUIT,
        s::STD_TUI_CM_ABOUT,
        s::STD_TUI_CM_OPEN,
        s::STD_TUI_CM_USER,
    ] {
        super::super::super::define_const(checker, name, Ty::Integer);
    }

    define_func(
        checker,
        s::STD_TUI_DIALOG_NEW_MODAL,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Title", Ty::String, false),
        ],
        types.dialog.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_BUTTON_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
            p("Command", Ty::Integer, false),
            p("IsDefault", Ty::Boolean, false),
        ],
        types.button.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_STATIC_TEXT_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
        ],
        types.static_text.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_CHECK_BOX_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
            p("Checked", Ty::Boolean, false),
        ],
        types.check_box.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_INPUT_LINE_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
            p("MaxLength", Ty::Integer, false),
        ],
        types.input_line.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_CHECK_BOX_CHECKED,
        vec![p("Cb", types.check_box.clone(), false)],
        Ty::Boolean,
    );
    define_proc(
        checker,
        s::STD_TUI_CHECK_BOX_SET_CHECKED,
        vec![
            p("Cb", types.check_box.clone(), false),
            p("Checked", Ty::Boolean, false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_INPUT_LINE_TEXT,
        vec![p("Line", types.input_line.clone(), false)],
        Ty::String,
    );
    define_proc(
        checker,
        s::STD_TUI_INPUT_LINE_SET_TEXT,
        vec![
            p("Line", types.input_line.clone(), false),
            p("Text", Ty::String, false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_OUTLINE_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p(
                "Roots",
                Ty::Array(Box::new(types.outline_node.clone())),
                false,
            ),
        ],
        types.outline.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_OUTLINE_SELECTION,
        vec![p("O", types.outline.clone(), false)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_TUI_OUTLINE_SELECTED_TEXT,
        vec![p("O", types.outline.clone(), false)],
        Ty::String,
    );
    define_proc(
        checker,
        s::STD_TUI_OUTLINE_SET_NODES,
        vec![
            p("O", types.outline.clone(), false),
            p(
                "Roots",
                Ty::Array(Box::new(types.outline_node.clone())),
                false,
            ),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_LIST_BOX_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Items", Ty::Array(Box::new(Ty::String)), false),
            p("Command", Ty::Integer, false),
        ],
        types.list_box.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_LIST_BOX_SELECTION,
        vec![p("Lb", types.list_box.clone(), false)],
        Ty::Integer,
    );
    define_proc(
        checker,
        s::STD_TUI_LIST_BOX_SET_ITEMS,
        vec![
            p("Lb", types.list_box.clone(), false),
            p("Items", Ty::Array(Box::new(Ty::String)), false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_RADIO_BUTTON_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
            p("GroupId", Ty::Integer, false),
            p("Selected", Ty::Boolean, false),
        ],
        types.radio_button.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_RADIO_BUTTON_SELECTED,
        vec![p("Rb", types.radio_button.clone(), false)],
        Ty::Boolean,
    );
    define_proc(
        checker,
        s::STD_TUI_RADIO_BUTTON_SET_SELECTED,
        vec![
            p("Rb", types.radio_button.clone(), false),
            p("Selected", Ty::Boolean, false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_MEMO_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
        ],
        types.memo.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_MEMO_SET_TEXT,
        vec![
            p("M", types.memo.clone(), false),
            p("Text", Ty::String, false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_TEXT_VIEWER_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
        ],
        types.text_viewer.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_TEXT_VIEWER_SET_TEXT,
        vec![
            p("V", types.text_viewer.clone(), false),
            p("Text", Ty::String, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_STATIC_TEXT_SET_TEXT,
        vec![
            p("Txt", types.static_text.clone(), false),
            p("Text", Ty::String, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_BUTTON_SET_TEXT,
        vec![
            p("Btn", types.button.clone(), false),
            p("Text", Ty::String, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_DIALOG_SET_TITLE,
        vec![
            p("Dlg", types.dialog.clone(), false),
            p("Title", Ty::String, false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_EXEC_VIEW,
        vec![
            p("App", types.application.clone(), false),
            p("Dialog", types.dialog.clone(), false),
        ],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_MESSAGE_BOX,
        vec![
            p("App", types.application.clone(), false),
            p("Message", Ty::String, false),
            p("Options", Ty::Integer, false),
        ],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_RUN_FILE_DIALOG,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Title", Ty::String, false),
            p("Wildcard", Ty::String, false),
            p("StartPath", Ty::Option(Box::new(Ty::String)), false),
        ],
        Ty::Option(Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_TUI_WINDOW_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Title", Ty::String, false),
        ],
        types.window.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_WINDOW_SET_TITLE,
        vec![
            p("Win", types.window.clone(), false),
            p("Title", Ty::String, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_DESKTOP_ADD,
        vec![
            p("App", types.application.clone(), false),
            p("Win", types.window.clone(), false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_MENU_BAR_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Menus", Ty::Array(Box::new(types.menu.clone())), false),
        ],
        types.menu_bar.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_STATUS_LINE_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p(
                "Items",
                Ty::Array(Box::new(types.status_item.clone())),
                false,
            ),
        ],
        types.status_line.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_MENU_BAR_SET_MENUS,
        vec![
            p("Bar", types.menu_bar.clone(), false),
            p("Menus", Ty::Array(Box::new(types.menu.clone())), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_STATUS_LINE_SET_ITEMS,
        vec![
            p("Line", types.status_line.clone(), false),
            p(
                "Items",
                Ty::Array(Box::new(types.status_item.clone())),
                false,
            ),
        ],
    );

    let builtin_placeholder = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: Vec::new(),
        variadic: false,
    });
    for name in [s::STD_TUI_DIALOG_ADD, s::STD_TUI_WINDOW_ADD] {
        define_builtin_std(checker, name, builtin_placeholder.clone());
    }
}
