//! Public retained control API registration.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use super::super::super::{define_func, define_proc, p};
use super::TuiTypes;
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register control construction, mutation, and query calls.
pub(super) fn register(checker: &mut Checker, types: &TuiTypes) {
    let geometry = || {
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
        ]
    };
    let mut label = geometry();
    label.extend([
        p("Text", Ty::String, false),
        p("Accelerator", Ty::Option(Box::new(Ty::String)), false),
    ]);
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_CREATE_LABEL_VIEW,
        label,
        types.view_id.clone(),
    );
    let mut button = geometry();
    button.extend([
        p("Caption", Ty::String, false),
        p("CommandId", Ty::Option(Box::new(Ty::Integer)), false),
        p("IsDefault", Ty::Boolean, false),
    ]);
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_CREATE_BUTTON_VIEW,
        button,
        types.view_id.clone(),
    );
    let mut input = geometry();
    input.push(p("Text", Ty::String, false));
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_CREATE_INPUT_LINE_VIEW,
        input,
        types.view_id.clone(),
    );
    let mut check = geometry();
    check.extend([
        p("Label", Ty::String, false),
        p("Accelerator", Ty::Option(Box::new(Ty::String)), false),
        p("CommandId", Ty::Option(Box::new(Ty::Integer)), false),
        p("Checked", Ty::Boolean, false),
    ]);
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_CREATE_CHECK_BOX_VIEW,
        check,
        types.view_id.clone(),
    );
    let mut radio = geometry();
    radio.push(p(
        "Options",
        Ty::Array(Box::new(types.controls.radio_option.clone())),
        false,
    ));
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_CREATE_RADIO_GROUP_VIEW,
        radio,
        types.view_id.clone(),
    );
    let mut list = geometry();
    list.push(p(
        "Items",
        Ty::Array(Box::new(types.controls.list_box_item.clone())),
        false,
    ));
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_CREATE_LIST_BOX_VIEW,
        list,
        types.view_id.clone(),
    );

    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_SET_INPUT_LINE_TEXT,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
            p("Text", Ty::String, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_SET_LIST_BOX_ITEMS,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
            p(
                "Items",
                Ty::Array(Box::new(types.controls.list_box_item.clone())),
                false,
            ),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_SET_LIST_BOX_SELECTED,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
            p("SelectedIndex", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_SET_CHECK_BOX_CHECKED,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
            p("Checked", Ty::Boolean, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_SET_RADIO_GROUP_SELECTED,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
            p("SelectedIndex", Ty::Integer, false),
        ],
    );
    for (name, ty) in [
        (
            s::STD_TUI_APPLICATION_QUERY_INPUT_LINE_STATE,
            types.controls.input_line_state.clone(),
        ),
        (
            s::STD_TUI_APPLICATION_QUERY_CHECK_BOX_STATE,
            types.controls.check_box_state.clone(),
        ),
        (
            s::STD_TUI_APPLICATION_QUERY_RADIO_GROUP_STATE,
            types.controls.radio_group_state.clone(),
        ),
        (
            s::STD_TUI_APPLICATION_QUERY_LIST_BOX_STATE,
            types.controls.list_box_state.clone(),
        ),
    ] {
        define_func(
            checker,
            name,
            vec![
                p("App", types.application.clone(), false),
                p("ViewId", types.view_id.clone(), false),
            ],
            ty,
        );
    }
}
