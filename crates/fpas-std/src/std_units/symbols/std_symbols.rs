macro_rules! std_console {
    ($suffix:literal) => {
        concat!("Std.Console.", $suffix)
    };
}
macro_rules! std_args {
    ($suffix:literal) => {
        concat!("Std.Args.", $suffix)
    };
}
macro_rules! std_env {
    ($suffix:literal) => {
        concat!("Std.Env.", $suffix)
    };
}
macro_rules! std_proc {
    ($suffix:literal) => {
        concat!("Std.Proc.", $suffix)
    };
}
macro_rules! std_path {
    ($suffix:literal) => {
        concat!("Std.Path.", $suffix)
    };
}
macro_rules! std_fs {
    ($suffix:literal) => {
        concat!("Std.Fs.", $suffix)
    };
}
macro_rules! std_time {
    ($suffix:literal) => {
        concat!("Std.Time.", $suffix)
    };
}
macro_rules! std_tui {
    ($suffix:literal) => {
        concat!("Std.Tui.", $suffix)
    };
}

macro_rules! std_graph {
    ($suffix:literal) => {
        concat!("Std.Graph.", $suffix)
    };
}
macro_rules! std_str {
    ($suffix:literal) => {
        concat!("Std.Str.", $suffix)
    };
}
macro_rules! std_conv {
    ($suffix:literal) => {
        concat!("Std.Conv.", $suffix)
    };
}
macro_rules! std_parse {
    ($suffix:literal) => {
        concat!("Std.Parse.", $suffix)
    };
}
macro_rules! std_math {
    ($suffix:literal) => {
        concat!("Std.Math.", $suffix)
    };
}
macro_rules! std_random {
    ($suffix:literal) => {
        concat!("Std.Random.", $suffix)
    };
}
macro_rules! std_array {
    ($suffix:literal) => {
        concat!("Std.Array.", $suffix)
    };
}
macro_rules! std_result {
    ($suffix:literal) => {
        concat!("Std.Result.", $suffix)
    };
}
macro_rules! std_option {
    ($suffix:literal) => {
        concat!("Std.Option.", $suffix)
    };
}
macro_rules! std_task {
    ($suffix:literal) => {
        concat!("Std.Task.", $suffix)
    };
}
macro_rules! std_dict {
    ($suffix:literal) => {
        concat!("Std.Dict.", $suffix)
    };
}
macro_rules! std_json {
    ($suffix:literal) => {
        concat!("Std.Json.", $suffix)
    };
}
macro_rules! std_test {
    ($suffix:literal) => {
        concat!("Std.Test.", $suffix)
    };
}

pub const STD_CONSOLE_WRITE_LN: &str = std_console!("WriteLn");
pub const STD_CONSOLE_WRITE: &str = std_console!("Write");
pub const STD_CONSOLE_CLR_SCR: &str = std_console!("ClrScr");
pub const STD_CONSOLE_CLR_EOL: &str = std_console!("ClrEol");
pub const STD_CONSOLE_GOTO_XY: &str = std_console!("GotoXY");
pub const STD_CONSOLE_WHERE_X: &str = std_console!("WhereX");
pub const STD_CONSOLE_WHERE_Y: &str = std_console!("WhereY");
pub const STD_CONSOLE_WIND_MIN: &str = std_console!("WindMin");
pub const STD_CONSOLE_WIND_MAX: &str = std_console!("WindMax");
pub const STD_CONSOLE_DEL_LINE: &str = std_console!("DelLine");
pub const STD_CONSOLE_INS_LINE: &str = std_console!("InsLine");
pub const STD_CONSOLE_WINDOW: &str = std_console!("Window");
pub const STD_CONSOLE_TEXT_COLOR: &str = std_console!("TextColor");
pub const STD_CONSOLE_TEXT_BACKGROUND: &str = std_console!("TextBackground");
pub const STD_CONSOLE_HIGH_VIDEO: &str = std_console!("HighVideo");
pub const STD_CONSOLE_LOW_VIDEO: &str = std_console!("LowVideo");
pub const STD_CONSOLE_NORM_VIDEO: &str = std_console!("NormVideo");
pub const STD_CONSOLE_TEXT_ATTR: &str = std_console!("TextAttr");
pub const STD_CONSOLE_SET_TEXT_ATTR: &str = std_console!("SetTextAttr");
pub const STD_CONSOLE_DELAY: &str = std_console!("Delay");
pub const STD_CONSOLE_CURSOR_ON: &str = std_console!("CursorOn");
pub const STD_CONSOLE_CURSOR_OFF: &str = std_console!("CursorOff");
pub const STD_CONSOLE_CURSOR_BIG: &str = std_console!("CursorBig");
pub const STD_CONSOLE_TEXT_MODE: &str = std_console!("TextMode");
pub const STD_CONSOLE_LAST_MODE: &str = std_console!("LastMode");
pub const STD_CONSOLE_SCREEN_WIDTH: &str = std_console!("ScreenWidth");
pub const STD_CONSOLE_SCREEN_HEIGHT: &str = std_console!("ScreenHeight");
pub const STD_CONSOLE_SOUND: &str = std_console!("Sound");
pub const STD_CONSOLE_NO_SOUND: &str = std_console!("NoSound");
pub const STD_CONSOLE_ASSIGN_CRT: &str = std_console!("AssignCrt");
pub const STD_CONSOLE_READ_LN: &str = std_console!("ReadLn");
pub const STD_CONSOLE_READ: &str = std_console!("Read");
pub const STD_CONSOLE_READ_KEY: &str = std_console!("ReadKey");
pub const STD_CONSOLE_KEY_PRESSED: &str = std_console!("KeyPressed");
pub const STD_CONSOLE_READ_KEY_EVENT: &str = std_console!("ReadKeyEvent");
pub const STD_CONSOLE_EVENT_PENDING: &str = std_console!("EventPending");
pub const STD_CONSOLE_READ_EVENT: &str = std_console!("ReadEvent");
pub const STD_CONSOLE_READ_EVENT_TIMEOUT: &str = std_console!("ReadEventTimeout");
pub const STD_CONSOLE_POLL_EVENT: &str = std_console!("PollEvent");
pub const STD_CONSOLE_KEY_EVENT: &str = std_console!("KeyEvent");
pub const STD_CONSOLE_KEY_KIND: &str = std_console!("KeyKind");
pub const STD_CONSOLE_EVENT: &str = std_console!("Event");
pub const STD_CONSOLE_EVENT_KIND: &str = std_console!("EventKind");
pub const STD_CONSOLE_MOUSE_ACTION: &str = std_console!("MouseAction");
pub const STD_CONSOLE_MOUSE_BUTTON: &str = std_console!("MouseButton");
pub const STD_CONSOLE_ENABLE_RAW_MODE: &str = std_console!("EnableRawMode");
pub const STD_CONSOLE_DISABLE_RAW_MODE: &str = std_console!("DisableRawMode");
pub const STD_CONSOLE_ENTER_ALT_SCREEN: &str = std_console!("EnterAltScreen");
pub const STD_CONSOLE_LEAVE_ALT_SCREEN: &str = std_console!("LeaveAltScreen");
pub const STD_CONSOLE_ENABLE_MOUSE: &str = std_console!("EnableMouse");
pub const STD_CONSOLE_DISABLE_MOUSE: &str = std_console!("DisableMouse");
pub const STD_CONSOLE_ENABLE_FOCUS: &str = std_console!("EnableFocus");
pub const STD_CONSOLE_DISABLE_FOCUS: &str = std_console!("DisableFocus");
pub const STD_CONSOLE_ENABLE_PASTE: &str = std_console!("EnablePaste");
pub const STD_CONSOLE_DISABLE_PASTE: &str = std_console!("DisablePaste");
pub const STD_CONSOLE_BLACK: &str = std_console!("Black");
pub const STD_CONSOLE_BLUE: &str = std_console!("Blue");
pub const STD_CONSOLE_GREEN: &str = std_console!("Green");
pub const STD_CONSOLE_CYAN: &str = std_console!("Cyan");
pub const STD_CONSOLE_RED: &str = std_console!("Red");
pub const STD_CONSOLE_MAGENTA: &str = std_console!("Magenta");
pub const STD_CONSOLE_BROWN: &str = std_console!("Brown");
pub const STD_CONSOLE_LIGHT_GRAY: &str = std_console!("LightGray");
pub const STD_CONSOLE_DARK_GRAY: &str = std_console!("DarkGray");
pub const STD_CONSOLE_LIGHT_BLUE: &str = std_console!("LightBlue");
pub const STD_CONSOLE_LIGHT_GREEN: &str = std_console!("LightGreen");
pub const STD_CONSOLE_LIGHT_CYAN: &str = std_console!("LightCyan");
pub const STD_CONSOLE_LIGHT_RED: &str = std_console!("LightRed");
pub const STD_CONSOLE_LIGHT_MAGENTA: &str = std_console!("LightMagenta");
pub const STD_CONSOLE_YELLOW: &str = std_console!("Yellow");
pub const STD_CONSOLE_WHITE: &str = std_console!("White");
pub const STD_CONSOLE_TEXT_COLOR_RGB: &str = std_console!("TextColorRGB");
pub const STD_CONSOLE_TEXT_BACKGROUND_RGB: &str = std_console!("TextBackgroundRGB");
pub const STD_CONSOLE_TEXT_COLOR_256: &str = std_console!("TextColor256");
pub const STD_CONSOLE_TEXT_BACKGROUND_256: &str = std_console!("TextBackground256");
pub const STD_CONSOLE_BLINK: &str = std_console!("Blink");
pub const STD_CONSOLE_BW40: &str = std_console!("BW40");
pub const STD_CONSOLE_C40: &str = std_console!("C40");
pub const STD_CONSOLE_BW80: &str = std_console!("BW80");
pub const STD_CONSOLE_C80: &str = std_console!("C80");
pub const STD_CONSOLE_CO40: &str = std_console!("CO40");
pub const STD_CONSOLE_CO80: &str = std_console!("CO80");
pub const STD_CONSOLE_MONO: &str = std_console!("Mono");
pub const STD_CONSOLE_FONT_8X8: &str = std_console!("Font8x8");

pub const STD_ARGS_PARAM_COUNT: &str = std_args!("ParamCount");
pub const STD_ARGS_PARAM_STR: &str = std_args!("ParamStr");

pub const STD_ENV_GET: &str = std_env!("Get");
pub const STD_ENV_EXISTS: &str = std_env!("Exists");

pub const STD_PROC_RUN: &str = std_proc!("Run");

pub const STD_PATH_JOIN: &str = std_path!("Join");
pub const STD_PATH_BASE_NAME: &str = std_path!("BaseName");
pub const STD_PATH_DIR_NAME: &str = std_path!("DirName");
pub const STD_PATH_EXTENSION: &str = std_path!("Extension");
pub const STD_PATH_NORMALIZE: &str = std_path!("Normalize");

pub const STD_FS_READ_TEXT: &str = std_fs!("ReadText");
pub const STD_FS_WRITE_TEXT: &str = std_fs!("WriteText");
pub const STD_FS_EXISTS: &str = std_fs!("Exists");
pub const STD_FS_IS_FILE: &str = std_fs!("IsFile");
pub const STD_FS_IS_DIR: &str = std_fs!("IsDir");
pub const STD_FS_CREATE_DIR: &str = std_fs!("CreateDir");

pub const STD_TIME_TIMESTAMP_MILLIS: &str = std_time!("TimestampMillis");
pub const STD_TIME_MONOTONIC_MILLIS: &str = std_time!("MonotonicMillis");
pub const STD_TIME_ELAPSED_MILLIS: &str = std_time!("ElapsedMillis");
pub const STD_TIME_SLEEP: &str = std_time!("Sleep");

pub const STD_TUI_APPLICATION: &str = std_tui!("Application");
pub const STD_TUI_VIEW_ID: &str = std_tui!("ViewId");
pub const STD_TUI_DIALOG: &str = std_tui!("Dialog");
pub const STD_TUI_WINDOW: &str = std_tui!("Window");
pub const STD_TUI_BUTTON: &str = std_tui!("Button");
pub const STD_TUI_STATIC_TEXT: &str = std_tui!("StaticText");
pub const STD_TUI_MEMO: &str = std_tui!("Memo");
pub const STD_TUI_TEXT_VIEWER: &str = std_tui!("TextViewer");
pub const STD_TUI_INPUT_LINE: &str = std_tui!("InputLine");
pub const STD_TUI_LIST_BOX: &str = std_tui!("ListBox");
pub const STD_TUI_CHECK_BOX: &str = std_tui!("CheckBox");
pub const STD_TUI_RADIO_BUTTON: &str = std_tui!("RadioButton");
pub const STD_TUI_MENU_BAR: &str = std_tui!("MenuBar");
pub const STD_TUI_MENU_BAR_ITEM: &str = std_tui!("MenuBarItem");
pub const STD_TUI_STATUS_LINE: &str = std_tui!("StatusLine");
pub const STD_TUI_STATUS_ITEM: &str = std_tui!("StatusItem");
/// Hosted-dispatch handler bundle for `Std.Tui.Application.Configure`; see `docs/pascal/std/tui/app/README.md`.
pub const STD_TUI_APPLICATION_HANDLERS: &str = std_tui!("ApplicationHandlers");
pub const STD_TUI_RECT: &str = std_tui!("Rect");
pub const STD_TUI_POINT: &str = std_tui!("Point");
pub const STD_TUI_SIZE: &str = std_tui!("Size");
pub const STD_TUI_COMMAND_ACCEPT: &str = std_tui!("Command.Accept");
pub const STD_TUI_COMMAND_CANCEL: &str = std_tui!("Command.Cancel");
pub const STD_TUI_COMMAND_CLOSE: &str = std_tui!("Command.Close");
pub const STD_TUI_COMMAND_QUIT: &str = std_tui!("Command.Quit");
pub const STD_TUI_SCREEN_CELL: &str = std_tui!("ScreenCell");
pub const STD_TUI_EVENT: &str = std_tui!("TuiEvent");
pub const STD_TUI_EVENT_KIND: &str = std_tui!("EventKind");
pub const STD_TUI_EXIT_REASON: &str = std_tui!("ExitReason");
pub const STD_TUI_APPLICATION_OPEN: &str = std_tui!("Application.Open");
pub const STD_TUI_APPLICATION_CLOSE: &str = std_tui!("Application.Close");
/// Configure hosted-dispatch handlers from a single bundle; see `docs/pascal/std/tui/app/README.md`.
pub const STD_TUI_APPLICATION_CONFIGURE: &str = std_tui!("Application.Configure");
/// Dispatch-mode hosted application loop; see `docs/pascal/std/tui/app/README.md`.
pub const STD_TUI_APPLICATION_RUN: &str = std_tui!("Application.Run");
/// High-level modal helper rooted at a host-managed view subtree.
pub const STD_TUI_APPLICATION_SHOW_MODAL: &str = std_tui!("Application.ShowModal");
/// High-level dialog helper that creates a root host-managed view and shows it modally.
pub const STD_TUI_APPLICATION_SHOW_DIALOG: &str = std_tui!("Application.ShowDialog");
/// Close the active modal dialog shown via `Application.ShowModal`.
pub const STD_TUI_APPLICATION_CLOSE_MODAL: &str = std_tui!("Application.CloseModal");
pub const STD_TUI_APPLICATION_SIZE: &str = std_tui!("Application.Size");
pub const STD_TUI_APPLICATION_REQUEST_REDRAW: &str = std_tui!("Application.RequestRedraw");
pub const STD_TUI_APPLICATION_CREATE_DIALOG: &str = std_tui!("Application.CreateDialog");
pub const STD_TUI_APPLICATION_CREATE_WINDOW: &str = std_tui!("Application.CreateWindow");
pub const STD_TUI_APPLICATION_CREATE_BUTTON: &str = std_tui!("Application.CreateButton");
pub const STD_TUI_APPLICATION_CREATE_STATIC_TEXT: &str = std_tui!("Application.CreateStaticText");
pub const STD_TUI_APPLICATION_CREATE_MEMO: &str = std_tui!("Application.CreateMemo");
pub const STD_TUI_APPLICATION_CREATE_TEXT_VIEWER: &str = std_tui!("Application.CreateTextViewer");
pub const STD_TUI_APPLICATION_CREATE_INPUT_LINE: &str = std_tui!("Application.CreateInputLine");
pub const STD_TUI_APPLICATION_CREATE_LIST_BOX: &str = std_tui!("Application.CreateListBox");
pub const STD_TUI_APPLICATION_CREATE_CHECK_BOX: &str = std_tui!("Application.CreateCheckBox");
pub const STD_TUI_APPLICATION_CREATE_RADIO_BUTTON: &str = std_tui!("Application.CreateRadioButton");
/// Show a modal Turbo Vision file dialog and return the selected path, or `None` when canceled.
pub const STD_TUI_APPLICATION_RUN_FILE_DIALOG: &str = std_tui!("Application.RunFileDialog");
/// Queue the result returned by the next headless `Application.RunFileDialog` call.
pub const STD_TUI_APPLICATION_TEST_SET_FILE_DIALOG_RESULT: &str =
    std_tui!("Application.TestSetFileDialogResult");
pub const STD_TUI_APPLICATION_CREATE_MENU_BAR: &str = std_tui!("Application.CreateMenuBar");
pub const STD_TUI_APPLICATION_SET_MENU_BAR: &str = std_tui!("Application.SetMenuBar");
pub const STD_TUI_APPLICATION_CREATE_STATUS_LINE: &str = std_tui!("Application.CreateStatusLine");
pub const STD_TUI_APPLICATION_SET_STATUS_LINE: &str = std_tui!("Application.SetStatusLine");
pub const STD_TUI_APPLICATION_ADD_CHILD: &str = std_tui!("Application.AddChild");
pub const STD_TUI_APPLICATION_ADD_WINDOW: &str = std_tui!("Application.AddWindow");
pub const STD_TUI_APPLICATION_ON_COMMAND: &str = std_tui!("Application.OnCommand");
pub const STD_TUI_APPLICATION_PUMP: &str = std_tui!("Application.Pump");
pub const STD_TUI_APPLICATION_QUIT: &str = std_tui!("Application.Quit");
pub const STD_TUI_APPLICATION_TEST_CLICK_BUTTON: &str = std_tui!("Application.TestClickButton");
pub const STD_TUI_APPLICATION_OPEN_FOR_TEST: &str = std_tui!("Application.OpenForTest");
pub const STD_TUI_APPLICATION_TEST_PUMP: &str = std_tui!("Application.TestPump");
pub const STD_TUI_APPLICATION_TEST_PUMP_UNTIL_IDLE: &str =
    std_tui!("Application.TestPumpUntilIdle");
pub const STD_TUI_APPLICATION_CLOSE_FOR_TEST: &str = std_tui!("Application.CloseForTest");
pub const STD_TUI_APPLICATION_TEST_SEND_KEY: &str = std_tui!("Application.TestSendKey");
pub const STD_TUI_APPLICATION_TEST_SEND_MOUSE: &str = std_tui!("Application.TestSendMouse");
pub const STD_TUI_APPLICATION_TEST_MOVE_MOUSE: &str = std_tui!("Application.TestMoveMouse");
pub const STD_TUI_APPLICATION_TEST_CLICK_MOUSE: &str = std_tui!("Application.TestClickMouse");
pub const STD_TUI_APPLICATION_TEST_RESIZE: &str = std_tui!("Application.TestResize");
pub const STD_TUI_APPLICATION_TEST_PASTE: &str = std_tui!("Application.TestPaste");
pub const STD_TUI_APPLICATION_TEST_FOCUS: &str = std_tui!("Application.TestFocus");
pub const STD_TUI_APPLICATION_QUERY_SCREEN_SIZE: &str = std_tui!("Application.QueryScreenSize");
pub const STD_TUI_APPLICATION_QUERY_SCREEN_LINE: &str = std_tui!("Application.QueryScreenLine");
pub const STD_TUI_APPLICATION_QUERY_SCREEN_CELL: &str = std_tui!("Application.QueryScreenCell");
pub const STD_TUI_APPLICATION_QUERY_ROOT_VIEWS: &str = std_tui!("Application.QueryRootViews");
pub const STD_TUI_APPLICATION_QUERY_VIEW_RECT: &str = std_tui!("Application.QueryViewRect");
pub const STD_TUI_APPLICATION_QUERY_VIEW_PARENT: &str = std_tui!("Application.QueryViewParent");
pub const STD_TUI_APPLICATION_QUERY_VIEW_CHILDREN: &str = std_tui!("Application.QueryViewChildren");
pub const STD_TUI_APPLICATION_QUERY_MODAL_DEPTH: &str = std_tui!("Application.QueryModalDepth");
pub const STD_TUI_APPLICATION_QUERY_FOCUSED_VIEW_ID: &str =
    std_tui!("Application.QueryFocusedViewId");
pub const STD_TUI_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED: &str =
    std_tui!("Application.HostRegisterOnKeyPressed");
pub const STD_TUI_APPLICATION_HOST_INVOKE_ON_KEY_PRESSED: &str =
    std_tui!("Application.HostInvokeOnKeyPressed");
pub const STD_TUI_APPLICATION_HOST_REGISTER_ON_RESIZE: &str =
    std_tui!("Application.HostRegisterOnResize");
pub const STD_TUI_APPLICATION_HOST_PROCESS_NEXT: &str = std_tui!("Application.HostProcessNext");
pub const STD_TUI_APPLICATION_HOST_REGISTER_ON_PAINT: &str =
    std_tui!("Application.HostRegisterOnPaint");
/// Register `procedure (Application)` plus an idle interval in milliseconds for hosted `OnIdle` callbacks.
pub const STD_TUI_APPLICATION_HOST_REGISTER_ON_IDLE: &str =
    std_tui!("Application.HostRegisterOnIdle");
pub const STD_TUI_APPLICATION_HOST_DISPATCH_REDRAW: &str =
    std_tui!("Application.HostDispatchRedraw");
pub const STD_TUI_APPLICATION_HOST_RUN_LOOP: &str = std_tui!("Application.HostRunLoop");
pub const STD_TUI_APPLICATION_HOST_REQUEST_QUIT: &str = std_tui!("Application.HostRequestQuit");
pub const STD_TUI_APPLICATION_HOST_REGISTER_ON_EXIT: &str =
    std_tui!("Application.HostRegisterOnExit");
/// Register `procedure (Application, Std.Console.Event)` for hosted mouse-event dispatch.
pub const STD_TUI_APPLICATION_HOST_REGISTER_ON_MOUSE: &str =
    std_tui!("Application.HostRegisterOnMouse");
/// Register `procedure (Application, Std.Console.Event)` for bracketed-paste dispatch.
pub const STD_TUI_APPLICATION_HOST_REGISTER_ON_PASTE: &str =
    std_tui!("Application.HostRegisterOnPaste");
/// Register `procedure (Application, Std.Console.Event)` for terminal focus-gained dispatch.
pub const STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_GAINED: &str =
    std_tui!("Application.HostRegisterOnFocusGained");
/// Register `procedure (Application, Std.Console.Event)` for terminal focus-lost dispatch.
pub const STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_LOST: &str =
    std_tui!("Application.HostRegisterOnFocusLost");
/// Register `procedure (Application)` for host-managed view focus-gained dispatch (Tab traversal).
pub const STD_TUI_APPLICATION_HOST_REGISTER_ON_ACTIVATE: &str =
    std_tui!("Application.HostRegisterOnActivate");
/// Register `procedure (Application)` for host-managed view focus-lost dispatch (Tab traversal).
pub const STD_TUI_APPLICATION_HOST_REGISTER_ON_DEACTIVATE: &str =
    std_tui!("Application.HostRegisterOnDeactivate");
/// Register `procedure (Application, integer)` for hosted command dispatch.
pub const STD_TUI_APPLICATION_HOST_REGISTER_ON_COMMAND: &str =
    std_tui!("Application.HostRegisterOnCommand");
/// Bind a `Std.Console.KeyEvent` shortcut to a hosted command id.
pub const STD_TUI_APPLICATION_HOST_BIND_COMMAND: &str = std_tui!("Application.HostBindCommand");
/// Bind a `Std.Console.KeyEvent` shortcut to a host-managed view subtree.
pub const STD_TUI_APPLICATION_HOST_BIND_COMMAND_TO_VIEW: &str =
    std_tui!("Application.HostBindCommandToView");
/// Bind a `Std.Console.KeyEvent` shortcut to the active modal frame.
pub const STD_TUI_APPLICATION_HOST_BIND_COMMAND_TO_ACTIVE_MODAL: &str =
    std_tui!("Application.HostBindCommandToActiveModal");
/// Push an application-defined modal id onto the hosted modal stack.
pub const STD_TUI_APPLICATION_HOST_ENTER_MODAL: &str = std_tui!("Application.HostEnterModal");
/// Pop the active hosted modal frame, if any.
pub const STD_TUI_APPLICATION_HOST_LEAVE_MODAL: &str = std_tui!("Application.HostLeaveModal");
/// Set and validate the active hosted modal result code.
pub const STD_TUI_APPLICATION_HOST_SET_ACTIVE_MODAL_RESULT: &str =
    std_tui!("Application.HostSetActiveModalResult");
/// Register a host-managed view and return its opaque `ViewId`.
pub const STD_TUI_APPLICATION_HOST_REGISTER_VIEW: &str = std_tui!("Application.HostRegisterView");
/// Remove a host-managed view by handle.
pub const STD_TUI_APPLICATION_HOST_UNREGISTER_VIEW: &str =
    std_tui!("Application.HostUnregisterView");
/// Append a host-managed view to the focus chain.
pub const STD_TUI_APPLICATION_HOST_PUSH_CHILD_VIEW: &str =
    std_tui!("Application.HostPushChildView");
/// Attach a host-managed view handle to the active modal scope.
pub const STD_TUI_APPLICATION_HOST_ATTACH_VIEW_TO_ACTIVE_MODAL: &str =
    std_tui!("Application.HostAttachViewToActiveModal");
/// Update the bounding rectangle for a host-managed view handle.
pub const STD_TUI_APPLICATION_HOST_SET_VIEW_RECT: &str = std_tui!("Application.HostSetViewRect");
/// Re-parent a host-managed view. Pass `None` as `Parent` to detach it back to the root list.
pub const STD_TUI_APPLICATION_HOST_SET_VIEW_PARENT: &str =
    std_tui!("Application.HostSetViewParent");
/// Register a local paint handler for a host-managed view.
pub const STD_TUI_APPLICATION_HOST_REGISTER_ON_VIEW_PAINT: &str =
    std_tui!("Application.HostRegisterOnViewPaint");
/// Register a host-managed solid-fill widget view and return its handle.
pub const STD_TUI_APPLICATION_HOST_CREATE_SOLID_FILL_VIEW: &str =
    std_tui!("Application.HostCreateSolidFillView");
/// Register a host-managed status bar widget from a Pascal segment model.
pub const STD_TUI_APPLICATION_HOST_CREATE_STATUS_BAR_VIEW: &str =
    std_tui!("Application.HostCreateStatusBarView");
/// Replace the segment model for an existing status bar widget view.
pub const STD_TUI_APPLICATION_HOST_SET_STATUS_BAR_SEGMENTS: &str =
    std_tui!("Application.HostSetStatusBarSegments");
pub const STD_TUI_STATUS_BAR_SEGMENT: &str = std_tui!("StatusBarSegment");
pub const STD_TUI_STATUS_BAR_STYLE: &str = std_tui!("StatusBarStyle");

pub const STD_GRAPH_APPLICATION: &str = std_graph!("Application");
/// Hosted-dispatch handler bundle for `Std.Graph.Application.Configure`; see `docs/pascal/std/graph/app/README.md`.
pub const STD_GRAPH_APPLICATION_HANDLERS: &str = std_graph!("ApplicationHandlers");
pub const STD_GRAPH_SIZE: &str = std_graph!("Size");
pub const STD_GRAPH_EVENT: &str = std_graph!("Event");
pub const STD_GRAPH_EVENT_KIND: &str = std_graph!("EventKind");
pub const STD_GRAPH_EXIT_REASON: &str = std_graph!("ExitReason");
pub const STD_GRAPH_APPLICATION_OPEN: &str = std_graph!("Application.Open");
pub const STD_GRAPH_APPLICATION_CLOSE: &str = std_graph!("Application.Close");
pub const STD_GRAPH_APPLICATION_CONFIGURE: &str = std_graph!("Application.Configure");
pub const STD_GRAPH_APPLICATION_RUN: &str = std_graph!("Application.Run");
pub const STD_GRAPH_APPLICATION_SIZE: &str = std_graph!("Application.Size");
pub const STD_GRAPH_APPLICATION_REQUEST_REDRAW: &str = std_graph!("Application.RequestRedraw");
pub const STD_GRAPH_APPLICATION_HOST_REQUEST_QUIT: &str = std_graph!("Application.HostRequestQuit");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED: &str =
    std_graph!("Application.HostRegisterOnKeyPressed");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_RESIZE: &str =
    std_graph!("Application.HostRegisterOnResize");
pub const STD_GRAPH_APPLICATION_HOST_PROCESS_NEXT: &str = std_graph!("Application.HostProcessNext");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_PAINT: &str =
    std_graph!("Application.HostRegisterOnPaint");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_IDLE: &str =
    std_graph!("Application.HostRegisterOnIdle");
pub const STD_GRAPH_APPLICATION_HOST_DISPATCH_REDRAW: &str =
    std_graph!("Application.HostDispatchRedraw");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_EXIT: &str =
    std_graph!("Application.HostRegisterOnExit");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_MOUSE: &str =
    std_graph!("Application.HostRegisterOnMouse");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_WHEEL: &str =
    std_graph!("Application.HostRegisterOnWheel");
pub const STD_GRAPH_APPLICATION_HOST_REGISTER_ON_CLOSE_REQUESTED: &str =
    std_graph!("Application.HostRegisterOnCloseRequested");
pub const STD_GRAPH_APPLICATION_UPLOAD_FRAME: &str = std_graph!("Application.UploadFrame");
pub const STD_GRAPH_APPLICATION_CLEAR: &str = std_graph!("Application.Clear");
pub const STD_GRAPH_APPLICATION_PUT_PIXEL: &str = std_graph!("Application.PutPixel");
pub const STD_GRAPH_APPLICATION_PRESENT: &str = std_graph!("Application.Present");
pub const STD_GRAPH_APPLICATION_DRAW_LINE: &str = std_graph!("Application.DrawLine");
pub const STD_GRAPH_APPLICATION_DRAW_RECT: &str = std_graph!("Application.DrawRect");
pub const STD_GRAPH_APPLICATION_FILL_RECT: &str = std_graph!("Application.FillRect");
pub const STD_GRAPH_APPLICATION_DRAW_CIRCLE: &str = std_graph!("Application.DrawCircle");
pub const STD_GRAPH_APPLICATION_DRAW_TEXT: &str = std_graph!("Application.DrawText");
pub const STD_GRAPH_APPLICATION_OPEN_FOR_TEST: &str = std_graph!("Application.OpenForTest");
pub const STD_GRAPH_APPLICATION_TEST_SEND_KEY: &str = std_graph!("Application.TestSendKey");

pub const STD_STR_LENGTH: &str = std_str!("Length");
pub const STD_STR_TO_UPPER: &str = std_str!("ToUpper");
pub const STD_STR_TO_LOWER: &str = std_str!("ToLower");
pub const STD_STR_TRIM: &str = std_str!("Trim");
pub const STD_STR_CONTAINS: &str = std_str!("Contains");
pub const STD_STR_STARTS_WITH: &str = std_str!("StartsWith");
pub const STD_STR_ENDS_WITH: &str = std_str!("EndsWith");
pub const STD_STR_SUBSTRING: &str = std_str!("Substring");
pub const STD_STR_INDEX_OF: &str = std_str!("IndexOf");
pub const STD_STR_REPLACE: &str = std_str!("Replace");
pub const STD_STR_SPLIT: &str = std_str!("Split");
pub const STD_STR_JOIN: &str = std_str!("Join");
pub const STD_STR_IS_NUMERIC: &str = std_str!("IsNumeric");
pub const STD_STR_REPEAT: &str = std_str!("RepeatStr");
pub const STD_STR_PAD_LEFT: &str = std_str!("PadLeft");
pub const STD_STR_PAD_RIGHT: &str = std_str!("PadRight");
pub const STD_STR_PAD_CENTER: &str = std_str!("PadCenter");
pub const STD_STR_FROM_CHAR: &str = std_str!("FromChar");
pub const STD_STR_CHAR_AT: &str = std_str!("CharAt");
pub const STD_STR_SET_CHAR_AT: &str = std_str!("SetCharAt");
pub const STD_STR_ORD: &str = std_str!("Ord");
pub const STD_STR_CHR: &str = std_str!("Chr");
pub const STD_STR_INSERT: &str = std_str!("Insert");
pub const STD_STR_DELETE: &str = std_str!("Delete");
pub const STD_STR_REVERSE: &str = std_str!("Reverse");
pub const STD_STR_TRIM_LEFT: &str = std_str!("TrimLeft");
pub const STD_STR_TRIM_RIGHT: &str = std_str!("TrimRight");
pub const STD_STR_LAST_INDEX_OF: &str = std_str!("LastIndexOf");
pub const STD_STR_FORMAT: &str = std_str!("Format");

pub const STD_CONV_INT_TO_STR: &str = std_conv!("IntToStr");
pub const STD_CONV_STR_TO_INT: &str = std_conv!("StrToInt");
pub const STD_CONV_REAL_TO_STR: &str = std_conv!("RealToStr");
pub const STD_CONV_STR_TO_REAL: &str = std_conv!("StrToReal");
pub const STD_CONV_INT_TO_REAL: &str = std_conv!("IntToReal");
pub const STD_CONV_BOOL_TO_STR: &str = std_conv!("BoolToStr");
pub const STD_CONV_STR_TO_BOOL: &str = std_conv!("StrToBool");
pub const STD_CONV_INT_TO_HEX: &str = std_conv!("IntToHex");
pub const STD_CONV_HEX_TO_INT: &str = std_conv!("HexToInt");

pub const STD_PARSE_TRY_INT: &str = std_parse!("TryInt");
pub const STD_PARSE_TRY_REAL: &str = std_parse!("TryReal");
pub const STD_PARSE_TRY_BOOL: &str = std_parse!("TryBool");

pub const STD_MATH_PI: &str = std_math!("Pi");
pub const STD_MATH_SQRT: &str = std_math!("Sqrt");
pub const STD_MATH_POW: &str = std_math!("Pow");
pub const STD_MATH_FLOOR: &str = std_math!("Floor");
pub const STD_MATH_CEIL: &str = std_math!("Ceil");
pub const STD_MATH_ROUND: &str = std_math!("Round");
pub const STD_MATH_SIN: &str = std_math!("Sin");
pub const STD_MATH_COS: &str = std_math!("Cos");
pub const STD_MATH_LOG: &str = std_math!("Log");
pub const STD_MATH_ABS: &str = std_math!("Abs");
pub const STD_MATH_MIN: &str = std_math!("Min");
pub const STD_MATH_MAX: &str = std_math!("Max");
pub const STD_MATH_TAN: &str = std_math!("Tan");
pub const STD_MATH_ARC_SIN: &str = std_math!("ArcSin");
pub const STD_MATH_ARC_COS: &str = std_math!("ArcCos");
pub const STD_MATH_ARC_TAN: &str = std_math!("ArcTan");
pub const STD_MATH_ARC_TAN2: &str = std_math!("ArcTan2");
pub const STD_MATH_EXP: &str = std_math!("Exp");
pub const STD_MATH_LOG10: &str = std_math!("Log10");
pub const STD_MATH_LOG2: &str = std_math!("Log2");
pub const STD_MATH_TRUNC: &str = std_math!("Trunc");
pub const STD_MATH_FRAC: &str = std_math!("Frac");
pub const STD_MATH_SIGN: &str = std_math!("Sign");
pub const STD_MATH_CLAMP: &str = std_math!("Clamp");

pub const STD_RANDOM_RANDOM: &str = std_random!("Random");
pub const STD_RANDOM_RANDOM_INT: &str = std_random!("RandomInt");
pub const STD_RANDOM_RANDOMIZE: &str = std_random!("Randomize");

pub const STD_ARRAY_LENGTH: &str = std_array!("Length");
pub const STD_ARRAY_SORT: &str = std_array!("Sort");
pub const STD_ARRAY_REVERSE: &str = std_array!("Reverse");
pub const STD_ARRAY_CONTAINS: &str = std_array!("Contains");
pub const STD_ARRAY_INDEX_OF: &str = std_array!("IndexOf");
pub const STD_ARRAY_SLICE: &str = std_array!("Slice");
pub const STD_ARRAY_PUSH: &str = std_array!("Push");
pub const STD_ARRAY_POP: &str = std_array!("Pop");
pub const STD_ARRAY_MAP: &str = std_array!("Map");
pub const STD_ARRAY_FILTER: &str = std_array!("Filter");
pub const STD_ARRAY_REDUCE: &str = std_array!("Reduce");
pub const STD_ARRAY_CONCAT: &str = std_array!("Concat");
pub const STD_ARRAY_FILL: &str = std_array!("Fill");
pub const STD_ARRAY_FIND: &str = std_array!("Find");
pub const STD_ARRAY_FIND_INDEX: &str = std_array!("FindIndex");
pub const STD_ARRAY_ANY: &str = std_array!("Any");
pub const STD_ARRAY_ALL: &str = std_array!("All");
pub const STD_ARRAY_FLAT_MAP: &str = std_array!("FlatMap");
pub const STD_ARRAY_FOR_EACH: &str = std_array!("ForEach");

pub const STD_RESULT_UNWRAP: &str = std_result!("Unwrap");
pub const STD_RESULT_UNWRAP_OR: &str = std_result!("UnwrapOr");
pub const STD_RESULT_IS_OK: &str = std_result!("IsOk");
pub const STD_RESULT_IS_ERR: &str = std_result!("IsError");
pub const STD_RESULT_MAP: &str = std_result!("Map");
pub const STD_RESULT_AND_THEN: &str = std_result!("AndThen");
pub const STD_RESULT_OR_ELSE: &str = std_result!("OrElse");

pub const STD_OPTION_UNWRAP: &str = std_option!("Unwrap");
pub const STD_OPTION_UNWRAP_OR: &str = std_option!("UnwrapOr");
pub const STD_OPTION_IS_SOME: &str = std_option!("IsSome");
pub const STD_OPTION_IS_NONE: &str = std_option!("IsNone");
pub const STD_OPTION_MAP: &str = std_option!("Map");
pub const STD_OPTION_AND_THEN: &str = std_option!("AndThen");
pub const STD_OPTION_OR_ELSE: &str = std_option!("OrElse");

pub const STD_TASK_WAIT: &str = std_task!("Wait");
pub const STD_TASK_WAIT_ALL: &str = std_task!("WaitAll");

pub const STD_DICT_LENGTH: &str = std_dict!("Length");
pub const STD_DICT_CONTAINS_KEY: &str = std_dict!("ContainsKey");
pub const STD_DICT_KEYS: &str = std_dict!("Keys");
pub const STD_DICT_VALUES: &str = std_dict!("Values");
pub const STD_DICT_REMOVE: &str = std_dict!("Remove");
pub const STD_DICT_GET: &str = std_dict!("Get");
pub const STD_DICT_MERGE: &str = std_dict!("Merge");
pub const STD_DICT_MAP: &str = std_dict!("Map");
pub const STD_DICT_FILTER: &str = std_dict!("Filter");

pub const STD_JSON_VALUE: &str = std_json!("JsonValue");
pub const STD_JSON_VALUE_NULL: &str = std_json!("JsonValue.Null");
pub const STD_JSON_VALUE_BOOL: &str = std_json!("JsonValue.Bool");
pub const STD_JSON_VALUE_NUMBER: &str = std_json!("JsonValue.Number");
pub const STD_JSON_VALUE_STRING: &str = std_json!("JsonValue.String");
pub const STD_JSON_VALUE_ARRAY: &str = std_json!("JsonValue.Array");
pub const STD_JSON_VALUE_OBJECT: &str = std_json!("JsonValue.Object");
pub const STD_JSON_PARSE: &str = std_json!("Parse");
pub const STD_JSON_STRINGIFY: &str = std_json!("Stringify");

pub const STD_TEST_ASSERT_TRUE: &str = std_test!("AssertTrue");
pub const STD_TEST_ASSERT_FALSE: &str = std_test!("AssertFalse");
pub const STD_TEST_ASSERT_EQUALS: &str = std_test!("AssertEquals");
pub const STD_TEST_FAIL: &str = std_test!("Fail");
pub const STD_TEST_SKIP: &str = std_test!("Skip");
pub const STD_TEST_ASSERT_SCREEN_LINE: &str = std_test!("AssertScreenLine");
pub const STD_TEST_ASSERT_SCREEN_CELL: &str = std_test!("AssertScreenCell");
pub const STD_TEST_PUSH_READLN: &str = std_test!("PushReadLn");
