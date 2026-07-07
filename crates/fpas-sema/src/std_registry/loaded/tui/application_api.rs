use super::super::super::{define_func, define_proc, p};
use super::{TuiCallbackTypes, TuiTypes};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the user-facing `Std.Tui.Application` calls.
///
/// **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).
pub(super) fn register_application_api(
    checker: &mut Checker,
    types: &TuiTypes,
    callbacks: &TuiCallbackTypes,
) {
    define_func(
        checker,
        s::STD_TUI_APPLICATION_OPEN,
        vec![],
        types.application.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_NEW,
        vec![],
        types.application.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_CLOSE,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_RUN,
        vec![p("App", types.application.clone(), false)],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_SIZE,
        vec![p("App", types.application.clone(), false)],
        types.size.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_DIALOG,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Title", Ty::String, false),
        ],
        types.dialog.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_WINDOW,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Title", Ty::String, false),
        ],
        types.window.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_BUTTON,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
            p("CommandId", Ty::Integer, false),
        ],
        types.button.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_STATIC_TEXT,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
        ],
        types.static_text.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_MEMO,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
        ],
        types.memo.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_TEXT_VIEWER,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
        ],
        types.text_viewer.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_INPUT_LINE,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
            p("MaxLength", Ty::Integer, false),
        ],
        types.input_line.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_LIST_BOX,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Items", Ty::Array(Box::new(Ty::String)), false),
            p("CommandId", Ty::Integer, false),
        ],
        types.list_box.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_OUTLINE,
        vec![
            p("App", types.application.clone(), false),
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
        s::STD_TUI_APPLICATION_CREATE_CHECK_BOX,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
            p("Checked", Ty::Boolean, false),
        ],
        types.check_box.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_RADIO_BUTTON,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
            p("GroupId", Ty::Integer, false),
            p("Selected", Ty::Boolean, false),
        ],
        types.radio_button.clone(),
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
        s::STD_TUI_APPLICATION_EXEC_DIALOG,
        vec![
            p("App", types.application.clone(), false),
            p("Dialog", types.dialog.clone(), false),
        ],
        types.dialog_result.clone(),
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
        s::STD_TUI_APPLICATION_INPUT_TEXT,
        vec![
            p("App", types.application.clone(), false),
            p("Field", types.input_line.clone(), false),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CHECKED,
        vec![
            p("App", types.application.clone(), false),
            p("Field", types.check_box.clone(), false),
        ],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_SELECTED,
        vec![
            p("App", types.application.clone(), false),
            p("Field", types.radio_button.clone(), false),
        ],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_LIST_SELECTION,
        vec![
            p("App", types.application.clone(), false),
            p("ListBox", types.list_box.clone(), false),
        ],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_OUTLINE_SELECTION,
        vec![
            p("App", types.application.clone(), false),
            p("Outline", types.outline.clone(), false),
        ],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_OUTLINE_SELECTED_TEXT,
        vec![
            p("App", types.application.clone(), false),
            p("Outline", types.outline.clone(), false),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_MENU_BAR,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Menus", Ty::Array(Box::new(types.menu.clone())), false),
        ],
        types.menu_bar.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_SET_MENU_BAR,
        vec![
            p("App", types.application.clone(), false),
            p("MenuBar", types.menu_bar.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_SET_MENUS,
        vec![
            p("App", types.application.clone(), false),
            p("MenuBar", types.menu_bar.clone(), false),
            p("Menus", Ty::Array(Box::new(types.menu.clone())), false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_STATUS_LINE,
        vec![
            p("App", types.application.clone(), false),
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
        s::STD_TUI_APPLICATION_SET_STATUS_LINE,
        vec![
            p("App", types.application.clone(), false),
            p("StatusLine", types.status_line.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_SET_STATUS_ITEMS,
        vec![
            p("App", types.application.clone(), false),
            p("StatusLine", types.status_line.clone(), false),
            p(
                "Items",
                Ty::Array(Box::new(types.status_item.clone())),
                false,
            ),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_ADD_WINDOW,
        vec![
            p("App", types.application.clone(), false),
            p("Window", types.window.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_ON_COMMAND,
        vec![
            p("App", types.application.clone(), false),
            p("OnCommand", callbacks.on_command.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_ON_KEY,
        vec![
            p("App", types.application.clone(), false),
            p("OnKey", callbacks.on_key.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_ON_MOUSE,
        vec![
            p("App", types.application.clone(), false),
            p("OnMouse", callbacks.on_mouse.clone(), false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_PUMP,
        vec![p("App", types.application.clone(), false)],
        Ty::Integer,
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_QUIT,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_CLICK_BUTTON,
        vec![
            p("App", types.application.clone(), false),
            p("Button", types.button.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_CLICK_MOUSE,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_DISPATCH_MENU_COMMAND,
        vec![
            p("App", types.application.clone(), false),
            p("MenuBar", types.menu_bar.clone(), false),
            p("MenuIndex", Ty::Integer, false),
            p("ItemIndex", Ty::Integer, false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_OPEN_FOR_TEST,
        vec![
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
        ],
        types.application.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_CLOSE_FOR_TEST,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_SET_FILE_DIALOG_RESULT,
        vec![
            p("App", types.application.clone(), false),
            p("Result", Ty::Option(Box::new(Ty::String)), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_SET_DIALOG_RESULT,
        vec![
            p("App", types.application.clone(), false),
            p("Command", Ty::Integer, false),
        ],
    );
}
