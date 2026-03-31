use super::super::{define_const, define_func, define_proc, define_proc_variadic, p};
use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, RecordTy, Ty};
use fpas_std::key_event::KEY_KIND_VARIANTS;
use fpas_std::std_symbols as s;
use fpas_std::{EVENT_KIND_VARIANTS, MOUSE_ACTION_VARIANTS, MOUSE_BUTTON_VARIANTS};

fn register_enum_type(checker: &mut Checker, qualified_name: &str, variants: &[&str]) -> Ty {
    let variants: Vec<EnumVariantTy> = variants
        .iter()
        .map(|variant| EnumVariantTy {
            name: (*variant).to_string(),
            fields: vec![],
        })
        .collect();
    let member_names: Vec<String> = variants.iter().map(|v| v.name.clone()).collect();
    let enum_ty = Ty::Enum(EnumTy {
        name: qualified_name.into(),
        type_params: Vec::new(),
        variants,
    });
    checker.scopes.define(
        qualified_name,
        Symbol {
            ty: enum_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
        },
    );

    for member in &member_names {
        let qualified = format!("{qualified_name}.{member}");
        checker.scopes.define(
            &qualified,
            Symbol {
                ty: enum_ty.clone(),
                mutable: false,
                kind: SymbolKind::EnumMember,
            },
        );
    }

    enum_ty
}

fn register_std_console_key_api(checker: &mut Checker) {
    let key_kind_ty = register_enum_type(checker, s::STD_CONSOLE_KEY_KIND, KEY_KIND_VARIANTS);

    let key_event_ty = Ty::Record(RecordTy {
        name: s::STD_CONSOLE_KEY_EVENT.into(),
        type_params: Vec::new(),
        fields: vec![
            ("kind".into(), key_kind_ty.clone()),
            ("ch".into(), Ty::Char),
            ("shift".into(), Ty::Boolean),
            ("ctrl".into(), Ty::Boolean),
            ("alt".into(), Ty::Boolean),
            ("meta".into(), Ty::Boolean),
        ],
        methods: Vec::new(),
    });
    checker.scopes.define(
        s::STD_CONSOLE_KEY_EVENT,
        Symbol {
            ty: key_event_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
        },
    );
    define_func(
        checker,
        s::STD_CONSOLE_READ_KEY_EVENT,
        vec![],
        key_event_ty.clone(),
    );

    let event_kind_ty = register_enum_type(checker, s::STD_CONSOLE_EVENT_KIND, EVENT_KIND_VARIANTS);
    let mouse_action_ty =
        register_enum_type(checker, s::STD_CONSOLE_MOUSE_ACTION, MOUSE_ACTION_VARIANTS);
    let mouse_button_ty =
        register_enum_type(checker, s::STD_CONSOLE_MOUSE_BUTTON, MOUSE_BUTTON_VARIANTS);

    let event_ty = Ty::Record(RecordTy {
        name: s::STD_CONSOLE_EVENT.into(),
        type_params: Vec::new(),
        fields: vec![
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
        methods: Vec::new(),
    });
    checker.scopes.define(
        s::STD_CONSOLE_EVENT,
        Symbol {
            ty: event_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
        },
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

pub(super) fn register_std_console(checker: &mut Checker) {
    register_std_console_key_api(checker);

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
    define_func(checker, s::STD_CONSOLE_READ, vec![], Ty::Char);
    define_func(checker, s::STD_CONSOLE_READ_KEY, vec![], Ty::Char);
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
