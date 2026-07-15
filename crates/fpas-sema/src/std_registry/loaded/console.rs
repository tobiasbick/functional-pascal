use super::super::{define_const, define_func, define_proc, define_proc_variadic, p};
use crate::check::Checker;
use crate::std_registry::loaded::type_registration;
use crate::types::Ty;
use fpas_std::key_event::KEY_KIND_VARIANTS;
use fpas_std::std_symbols as s;
use fpas_std::{
    CONSOLE_COLOR_KIND_VARIANTS, EVENT_KIND_VARIANTS, MOUSE_ACTION_VARIANTS, MOUSE_BUTTON_VARIANTS,
};

/// Registers cell, color, frame, and saved-region types and operations.
fn register_std_console_cell_api(checker: &mut Checker) {
    let color_kind = type_registration::register_enum_type(
        checker,
        s::STD_CONSOLE_COLOR_KIND,
        CONSOLE_COLOR_KIND_VARIANTS,
    );
    let color = type_registration::register_record_type(
        checker,
        s::STD_CONSOLE_COLOR,
        vec![
            ("kind".into(), color_kind),
            ("index".into(), Ty::Integer),
            ("red".into(), Ty::Integer),
            ("green".into(), Ty::Integer),
            ("blue".into(), Ty::Integer),
        ],
    );
    let cell = type_registration::register_record_type(
        checker,
        s::STD_CONSOLE_CELL,
        vec![
            ("glyph".into(), Ty::String),
            ("foreground".into(), color.clone()),
            ("background".into(), color.clone()),
        ],
    );
    let rect = type_registration::register_record_type(
        checker,
        s::STD_CONSOLE_RECT,
        vec![
            ("x".into(), Ty::Integer),
            ("y".into(), Ty::Integer),
            ("width".into(), Ty::Integer),
            ("height".into(), Ty::Integer),
        ],
    );
    let saved_region =
        type_registration::register_record_type(checker, s::STD_CONSOLE_SAVED_REGION, Vec::new());

    define_func(
        checker,
        s::STD_CONSOLE_CRT_COLOR,
        vec![p("Index", Ty::Integer, false)],
        color.clone(),
    );
    define_func(
        checker,
        s::STD_CONSOLE_ANSI_256_COLOR,
        vec![p("Index", Ty::Integer, false)],
        color.clone(),
    );
    define_func(
        checker,
        s::STD_CONSOLE_RGB_COLOR,
        vec![
            p("Red", Ty::Integer, false),
            p("Green", Ty::Integer, false),
            p("Blue", Ty::Integer, false),
        ],
        color,
    );
    define_proc(checker, s::STD_CONSOLE_BEGIN_FRAME, vec![]);
    define_proc(checker, s::STD_CONSOLE_PRESENT, vec![]);
    define_proc(
        checker,
        s::STD_CONSOLE_PUT_CELL,
        vec![
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Value", cell.clone(), false),
        ],
    );
    define_func(
        checker,
        s::STD_CONSOLE_GET_CELL,
        vec![p("X", Ty::Integer, false), p("Y", Ty::Integer, false)],
        Ty::Option(Box::new(cell.clone())),
    );
    define_proc(
        checker,
        s::STD_CONSOLE_FILL_RECT,
        vec![
            p("Bounds", rect.clone(), false),
            p("Value", cell.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_CONSOLE_WRITE_CELLS,
        vec![
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Values", Ty::Array(Box::new(cell)), false),
        ],
    );
    define_func(
        checker,
        s::STD_CONSOLE_SAVE_REGION,
        vec![p("Bounds", rect, false)],
        saved_region.clone(),
    );
    define_proc(
        checker,
        s::STD_CONSOLE_RESTORE_REGION,
        vec![p("Region", saved_region.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_CONSOLE_DISCARD_REGION,
        vec![p("Region", saved_region, false)],
    );
    define_func(
        checker,
        s::STD_CONSOLE_DISPLAY_WIDTH,
        vec![p("Text", Ty::String, false)],
        Ty::Integer,
    );
}

pub(super) fn register_std_console_key_api(checker: &mut Checker) {
    let key_kind_ty =
        type_registration::register_enum_type(checker, s::STD_CONSOLE_KEY_KIND, KEY_KIND_VARIANTS);

    let key_event_ty = type_registration::register_record_type(
        checker,
        s::STD_CONSOLE_KEY_EVENT,
        vec![
            ("kind".into(), key_kind_ty.clone()),
            ("ch".into(), Ty::String),
            ("shift".into(), Ty::Boolean),
            ("ctrl".into(), Ty::Boolean),
            ("alt".into(), Ty::Boolean),
            ("meta".into(), Ty::Boolean),
        ],
    );
    define_func(
        checker,
        s::STD_CONSOLE_READ_KEY_EVENT,
        vec![],
        key_event_ty.clone(),
    );

    let event_kind_ty = type_registration::register_enum_type(
        checker,
        s::STD_CONSOLE_EVENT_KIND,
        EVENT_KIND_VARIANTS,
    );
    let mouse_action_ty = type_registration::register_enum_type(
        checker,
        s::STD_CONSOLE_MOUSE_ACTION,
        MOUSE_ACTION_VARIANTS,
    );
    let mouse_button_ty = type_registration::register_enum_type(
        checker,
        s::STD_CONSOLE_MOUSE_BUTTON,
        MOUSE_BUTTON_VARIANTS,
    );

    let event_ty = type_registration::register_record_type(
        checker,
        s::STD_CONSOLE_EVENT,
        vec![
            ("kind".into(), event_kind_ty),
            ("key".into(), key_event_ty),
            ("mouse_action".into(), mouse_action_ty),
            ("mouse_button".into(), mouse_button_ty),
            ("mouse_x".into(), Ty::Integer),
            ("mouse_y".into(), Ty::Integer),
            ("width".into(), Ty::Integer),
            ("height".into(), Ty::Integer),
            ("text".into(), Ty::String),
            ("shift".into(), Ty::Boolean),
            ("ctrl".into(), Ty::Boolean),
            ("alt".into(), Ty::Boolean),
            ("meta".into(), Ty::Boolean),
        ],
    );
    define_func(checker, s::STD_CONSOLE_READ_EVENT, vec![], event_ty.clone());
    define_func(checker, s::STD_CONSOLE_EVENT_PENDING, vec![], Ty::Boolean);
    define_func(
        checker,
        s::STD_CONSOLE_READ_EVENT_TIMEOUT,
        vec![p("Milliseconds", Ty::Integer, false)],
        Ty::Option(Box::new(event_ty.clone())),
    );
    define_func(
        checker,
        s::STD_CONSOLE_POLL_EVENT,
        vec![],
        Ty::Option(Box::new(event_ty)),
    );
}

/// Registers the complete public `Std.Console` API.
pub(super) fn register_std_console(checker: &mut Checker) {
    register_std_console_key_api(checker);
    register_std_console_cell_api(checker);

    for color_name in [
        s::STD_CONSOLE_BLACK,
        s::STD_CONSOLE_BLUE,
        s::STD_CONSOLE_GREEN,
        s::STD_CONSOLE_CYAN,
        s::STD_CONSOLE_RED,
        s::STD_CONSOLE_MAGENTA,
        s::STD_CONSOLE_BROWN,
        s::STD_CONSOLE_LIGHT_GRAY,
        s::STD_CONSOLE_DARK_GRAY,
        s::STD_CONSOLE_LIGHT_BLUE,
        s::STD_CONSOLE_LIGHT_GREEN,
        s::STD_CONSOLE_LIGHT_CYAN,
        s::STD_CONSOLE_LIGHT_RED,
        s::STD_CONSOLE_LIGHT_MAGENTA,
        s::STD_CONSOLE_YELLOW,
        s::STD_CONSOLE_WHITE,
        s::STD_CONSOLE_BLINK,
        s::STD_CONSOLE_BW40,
        s::STD_CONSOLE_C40,
        s::STD_CONSOLE_BW80,
        s::STD_CONSOLE_C80,
        s::STD_CONSOLE_CO40,
        s::STD_CONSOLE_CO80,
        s::STD_CONSOLE_MONO,
        s::STD_CONSOLE_FONT_8X8,
    ] {
        define_const(checker, color_name, Ty::Integer);
    }

    define_proc_variadic(checker, s::STD_CONSOLE_WRITE_LN);
    define_proc_variadic(checker, s::STD_CONSOLE_WRITE);
    define_proc(checker, s::STD_CONSOLE_CLR_SCR, vec![]);
    define_proc(checker, s::STD_CONSOLE_CLR_EOL, vec![]);
    define_proc(
        checker,
        s::STD_CONSOLE_GOTO_XY,
        vec![p("X", Ty::Integer, false), p("Y", Ty::Integer, false)],
    );
    define_func(checker, s::STD_CONSOLE_WHERE_X, vec![], Ty::Integer);
    define_func(checker, s::STD_CONSOLE_WHERE_Y, vec![], Ty::Integer);
    define_func(checker, s::STD_CONSOLE_WIND_MIN, vec![], Ty::Integer);
    define_func(checker, s::STD_CONSOLE_WIND_MAX, vec![], Ty::Integer);
    define_proc(checker, s::STD_CONSOLE_DEL_LINE, vec![]);
    define_proc(checker, s::STD_CONSOLE_INS_LINE, vec![]);
    define_proc(
        checker,
        s::STD_CONSOLE_WINDOW,
        vec![
            p("X1", Ty::Integer, false),
            p("Y1", Ty::Integer, false),
            p("X2", Ty::Integer, false),
            p("Y2", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_CONSOLE_TEXT_COLOR,
        vec![p("Color", Ty::Integer, false)],
    );
    define_proc(
        checker,
        s::STD_CONSOLE_TEXT_BACKGROUND,
        vec![p("Color", Ty::Integer, false)],
    );
    define_proc(
        checker,
        s::STD_CONSOLE_TEXT_COLOR_RGB,
        vec![
            p("R", Ty::Integer, false),
            p("G", Ty::Integer, false),
            p("B", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_CONSOLE_TEXT_BACKGROUND_RGB,
        vec![
            p("R", Ty::Integer, false),
            p("G", Ty::Integer, false),
            p("B", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_CONSOLE_TEXT_COLOR_256,
        vec![p("Index", Ty::Integer, false)],
    );
    define_proc(
        checker,
        s::STD_CONSOLE_TEXT_BACKGROUND_256,
        vec![p("Index", Ty::Integer, false)],
    );
    define_proc(checker, s::STD_CONSOLE_HIGH_VIDEO, vec![]);
    define_proc(checker, s::STD_CONSOLE_LOW_VIDEO, vec![]);
    define_proc(checker, s::STD_CONSOLE_NORM_VIDEO, vec![]);
    define_func(checker, s::STD_CONSOLE_TEXT_ATTR, vec![], Ty::Integer);
    define_proc(
        checker,
        s::STD_CONSOLE_SET_TEXT_ATTR,
        vec![p("Attr", Ty::Integer, false)],
    );
    define_proc(
        checker,
        s::STD_CONSOLE_DELAY,
        vec![p("Milliseconds", Ty::Integer, false)],
    );
    define_proc(checker, s::STD_CONSOLE_CURSOR_ON, vec![]);
    define_proc(checker, s::STD_CONSOLE_CURSOR_OFF, vec![]);
    define_proc(checker, s::STD_CONSOLE_CURSOR_BIG, vec![]);
    define_proc(
        checker,
        s::STD_CONSOLE_TEXT_MODE,
        vec![p("Mode", Ty::Integer, false)],
    );
    define_func(checker, s::STD_CONSOLE_LAST_MODE, vec![], Ty::Integer);
    define_func(checker, s::STD_CONSOLE_SCREEN_WIDTH, vec![], Ty::Integer);
    define_func(checker, s::STD_CONSOLE_SCREEN_HEIGHT, vec![], Ty::Integer);
    define_proc(
        checker,
        s::STD_CONSOLE_SOUND,
        vec![p("Hz", Ty::Integer, false)],
    );
    define_proc(checker, s::STD_CONSOLE_NO_SOUND, vec![]);
    define_proc(checker, s::STD_CONSOLE_ASSIGN_CRT, vec![]);
    define_func(checker, s::STD_CONSOLE_READ_LN, vec![], Ty::String);
    define_func(checker, s::STD_CONSOLE_READ, vec![], Ty::String);
    define_func(checker, s::STD_CONSOLE_READ_KEY, vec![], Ty::String);
    define_func(checker, s::STD_CONSOLE_KEY_PRESSED, vec![], Ty::Boolean);
    define_proc(checker, s::STD_CONSOLE_ENABLE_RAW_MODE, vec![]);
    define_proc(checker, s::STD_CONSOLE_DISABLE_RAW_MODE, vec![]);
    define_proc(checker, s::STD_CONSOLE_ENTER_ALT_SCREEN, vec![]);
    define_proc(checker, s::STD_CONSOLE_LEAVE_ALT_SCREEN, vec![]);
    define_proc(checker, s::STD_CONSOLE_ENABLE_MOUSE, vec![]);
    define_proc(checker, s::STD_CONSOLE_DISABLE_MOUSE, vec![]);
    define_proc(checker, s::STD_CONSOLE_ENABLE_FOCUS, vec![]);
    define_proc(checker, s::STD_CONSOLE_DISABLE_FOCUS, vec![]);
    define_proc(checker, s::STD_CONSOLE_ENABLE_PASTE, vec![]);
    define_proc(checker, s::STD_CONSOLE_DISABLE_PASTE, vec![]);
}
