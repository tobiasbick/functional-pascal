//! `Std.Tui` control model and query-state registration.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use crate::check::Checker;
use crate::std_registry::loaded::type_registration;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Registered types used by retained control APIs.
pub(super) struct TuiControlTypes {
    pub(super) radio_option: Ty,
    pub(super) input_line_state: Ty,
    pub(super) check_box_state: Ty,
    pub(super) radio_group_state: Ty,
    pub(super) list_box_item: Ty,
    pub(super) list_box_state: Ty,
    pub(super) scroll_bar_state: Ty,
    pub(super) scroll_view_state: Ty,
    pub(super) memo_state: Ty,
    pub(super) frame_root_state: Ty,
    pub(super) frame_scroll_state: Ty,
    pub(super) frame_window_entry: Ty,
}

/// Register retained control input and state records.
pub(super) fn register(checker: &mut Checker) -> TuiControlTypes {
    let radio_option = type_registration::register_record_type(
        checker,
        s::STD_TUI_RADIO_OPTION,
        vec![
            ("label".into(), Ty::String),
            ("accelerator".into(), Ty::Option(Box::new(Ty::String))),
            ("commandId".into(), Ty::Option(Box::new(Ty::Integer))),
            ("enabled".into(), Ty::Boolean),
        ],
    );
    let input_line_state = type_registration::register_record_type(
        checker,
        s::STD_TUI_INPUT_LINE_STATE,
        vec![
            ("text".into(), Ty::String),
            ("cursor".into(), Ty::Integer),
            ("scrollOffset".into(), Ty::Integer),
        ],
    );
    let check_box_state = type_registration::register_record_type(
        checker,
        s::STD_TUI_CHECK_BOX_STATE,
        vec![("checked".into(), Ty::Boolean)],
    );
    let radio_group_state = type_registration::register_record_type(
        checker,
        s::STD_TUI_RADIO_GROUP_STATE,
        vec![
            ("selectedIndex".into(), Ty::Integer),
            ("focusedIndex".into(), Ty::Integer),
        ],
    );
    let list_box_item = type_registration::register_record_type(
        checker,
        s::STD_TUI_LIST_BOX_ITEM,
        vec![
            ("text".into(), Ty::String),
            ("commandId".into(), Ty::Option(Box::new(Ty::Integer))),
            ("enabled".into(), Ty::Boolean),
        ],
    );
    let list_box_state = type_registration::register_record_type(
        checker,
        s::STD_TUI_LIST_BOX_STATE,
        vec![
            ("selectedIndex".into(), Ty::Integer),
            ("scrollOffset".into(), Ty::Integer),
        ],
    );
    let scroll_bar_state = type_registration::register_record_type(
        checker,
        s::STD_TUI_SCROLL_BAR_STATE,
        vec![
            ("scrollOffset".into(), Ty::Integer),
            ("contentLength".into(), Ty::Integer),
            ("viewportLength".into(), Ty::Integer),
        ],
    );
    let scroll_view_state = type_registration::register_record_type(
        checker,
        s::STD_TUI_SCROLL_VIEW_STATE,
        vec![
            ("scrollOffset".into(), Ty::Integer),
            ("lineCount".into(), Ty::Integer),
        ],
    );
    let memo_state = type_registration::register_record_type(
        checker,
        s::STD_TUI_MEMO_STATE,
        vec![
            ("text".into(), Ty::String),
            ("cursorLine".into(), Ty::Integer),
            ("cursorColumn".into(), Ty::Integer),
            ("scrollOffset".into(), Ty::Integer),
            ("selectionAnchorLine".into(), Ty::Integer),
            ("selectionAnchorColumn".into(), Ty::Integer),
        ],
    );
    let frame_root_state = type_registration::register_record_type(
        checker,
        s::STD_TUI_FRAME_ROOT_STATE,
        vec![
            ("x".into(), Ty::Integer),
            ("y".into(), Ty::Integer),
            ("width".into(), Ty::Integer),
            ("height".into(), Ty::Integer),
            ("kind".into(), Ty::Integer),
            ("movable".into(), Ty::Boolean),
            ("resizable".into(), Ty::Boolean),
            ("zoomable".into(), Ty::Boolean),
            ("scrollable".into(), Ty::Boolean),
            ("closable".into(), Ty::Boolean),
            ("zoomed".into(), Ty::Boolean),
        ],
    );
    let frame_scroll_state = type_registration::register_record_type(
        checker,
        s::STD_TUI_FRAME_SCROLL_STATE,
        vec![
            ("offsetX".into(), Ty::Integer),
            ("offsetY".into(), Ty::Integer),
            ("contentWidth".into(), Ty::Integer),
            ("contentHeight".into(), Ty::Integer),
        ],
    );
    let frame_window_entry = type_registration::register_record_type(
        checker,
        s::STD_TUI_FRAME_WINDOW_ENTRY,
        vec![
            (
                "id".into(),
                type_registration::lookup_required_type(checker, s::STD_TUI_VIEW_ID, "ViewId"),
            ),
            ("title".into(), Ty::String),
            ("kind".into(), Ty::Integer),
            ("active".into(), Ty::Boolean),
            ("zIndex".into(), Ty::Integer),
        ],
    );
    TuiControlTypes {
        radio_option,
        input_line_state,
        check_box_state,
        radio_group_state,
        list_box_item,
        list_box_state,
        scroll_bar_state,
        scroll_view_state,
        memo_state,
        frame_root_state,
        frame_scroll_state,
        frame_window_entry,
    }
}
