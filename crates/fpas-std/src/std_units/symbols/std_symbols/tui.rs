//! `Std.Tui` symbol names and registry group.

pub const STD_TUI_APPLICATION: &str = std_tui!("Application");
pub const STD_TUI_VIEW_ID: &str = std_tui!("ViewId");
pub const STD_TUI_DIALOG: &str = std_tui!("Dialog");
/// Result of a modal `Application.ExecDialog` call.
pub const STD_TUI_DIALOG_RESULT: &str = std_tui!("DialogResult");
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
pub const STD_TUI_MENU: &str = std_tui!("Menu");
pub const STD_TUI_MENU_ITEM: &str = std_tui!("MenuItem");
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
/// Run a dialog modally and return the closing command.
pub const STD_TUI_APPLICATION_EXEC_DIALOG: &str = std_tui!("Application.ExecDialog");
/// Show an upstream Turbo Vision message box and return the closing command id.
pub const STD_TUI_APPLICATION_MESSAGE_BOX: &str = std_tui!("Application.MessageBox");
/// Read the current text of an input line (valid after `Application.ExecDialog`).
pub const STD_TUI_APPLICATION_INPUT_TEXT: &str = std_tui!("Application.InputText");
/// Read the checked state of a check box (valid after `Application.ExecDialog`).
pub const STD_TUI_APPLICATION_CHECKED: &str = std_tui!("Application.Checked");
/// Read the selected state of a radio button (valid after `Application.ExecDialog`).
pub const STD_TUI_APPLICATION_SELECTED: &str = std_tui!("Application.Selected");
/// Read the selected index of a list box, or `-1` when no item is selected.
pub const STD_TUI_APPLICATION_LIST_SELECTION: &str = std_tui!("Application.ListSelection");
/// Queue the closing command returned by the next headless `Application.ExecDialog` call.
pub const STD_TUI_APPLICATION_TEST_SET_DIALOG_RESULT: &str =
    std_tui!("Application.TestSetDialogResult");
pub const STD_TUI_APPLICATION_CREATE_MENU_BAR: &str = std_tui!("Application.CreateMenuBar");
pub const STD_TUI_APPLICATION_SET_MENU_BAR: &str = std_tui!("Application.SetMenuBar");
pub const STD_TUI_APPLICATION_SET_MENUS: &str = std_tui!("Application.SetMenus");
pub const STD_TUI_APPLICATION_CREATE_STATUS_LINE: &str = std_tui!("Application.CreateStatusLine");
pub const STD_TUI_APPLICATION_SET_STATUS_LINE: &str = std_tui!("Application.SetStatusLine");
pub const STD_TUI_APPLICATION_SET_STATUS_ITEMS: &str = std_tui!("Application.SetStatusItems");
pub const STD_TUI_APPLICATION_ADD_CHILD: &str = std_tui!("Application.AddChild");
pub const STD_TUI_APPLICATION_SET_TEXT: &str = std_tui!("Application.SetText");
pub const STD_TUI_APPLICATION_SET_CHECKED: &str = std_tui!("Application.SetChecked");
pub const STD_TUI_APPLICATION_SET_ITEMS: &str = std_tui!("Application.SetItems");
pub const STD_TUI_APPLICATION_SET_TITLE: &str = std_tui!("Application.SetTitle");
pub const STD_TUI_APPLICATION_ADD_WINDOW: &str = std_tui!("Application.AddWindow");
pub const STD_TUI_APPLICATION_ON_COMMAND: &str = std_tui!("Application.OnCommand");
/// Register `function (Application, Std.Console.KeyEvent): boolean` for unhandled Turbo Vision keys.
pub const STD_TUI_APPLICATION_ON_KEY: &str = std_tui!("Application.OnKey");
/// Register `procedure (Application, Std.Console.Event)` for unhandled Turbo Vision mouse events.
pub const STD_TUI_APPLICATION_ON_MOUSE: &str = std_tui!("Application.OnMouse");
pub const STD_TUI_APPLICATION_PUMP: &str = std_tui!("Application.Pump");
pub const STD_TUI_APPLICATION_QUIT: &str = std_tui!("Application.Quit");
pub const STD_TUI_APPLICATION_TEST_CLICK_BUTTON: &str = std_tui!("Application.TestClickButton");
/// Queue a menu item command for headless tests (`MenuIndex` / `ItemIndex` into `CreateMenuBar` data).
pub const STD_TUI_APPLICATION_TEST_DISPATCH_MENU_COMMAND: &str =
    std_tui!("Application.TestDispatchMenuCommand");
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

pub(in crate::std_units) const STD_TUI_SYMBOLS: &[&str] = &[
    STD_TUI_APPLICATION,
    STD_TUI_VIEW_ID,
    STD_TUI_DIALOG,
    STD_TUI_DIALOG_RESULT,
    STD_TUI_WINDOW,
    STD_TUI_BUTTON,
    STD_TUI_STATIC_TEXT,
    STD_TUI_MEMO,
    STD_TUI_TEXT_VIEWER,
    STD_TUI_INPUT_LINE,
    STD_TUI_LIST_BOX,
    STD_TUI_CHECK_BOX,
    STD_TUI_RADIO_BUTTON,
    STD_TUI_MENU_BAR,
    STD_TUI_MENU,
    STD_TUI_MENU_ITEM,
    STD_TUI_STATUS_LINE,
    STD_TUI_STATUS_ITEM,
    STD_TUI_APPLICATION_HANDLERS,
    STD_TUI_RECT,
    STD_TUI_POINT,
    STD_TUI_SIZE,
    STD_TUI_COMMAND_ACCEPT,
    STD_TUI_COMMAND_CANCEL,
    STD_TUI_COMMAND_CLOSE,
    STD_TUI_COMMAND_QUIT,
    STD_TUI_SCREEN_CELL,
    STD_TUI_EVENT,
    STD_TUI_EVENT_KIND,
    STD_TUI_EXIT_REASON,
    STD_TUI_APPLICATION_OPEN,
    STD_TUI_APPLICATION_CLOSE,
    STD_TUI_APPLICATION_CONFIGURE,
    STD_TUI_APPLICATION_RUN,
    STD_TUI_APPLICATION_SIZE,
    STD_TUI_APPLICATION_REQUEST_REDRAW,
    STD_TUI_APPLICATION_CREATE_DIALOG,
    STD_TUI_APPLICATION_CREATE_WINDOW,
    STD_TUI_APPLICATION_CREATE_BUTTON,
    STD_TUI_APPLICATION_CREATE_STATIC_TEXT,
    STD_TUI_APPLICATION_CREATE_MEMO,
    STD_TUI_APPLICATION_CREATE_TEXT_VIEWER,
    STD_TUI_APPLICATION_CREATE_INPUT_LINE,
    STD_TUI_APPLICATION_CREATE_LIST_BOX,
    STD_TUI_APPLICATION_CREATE_CHECK_BOX,
    STD_TUI_APPLICATION_CREATE_RADIO_BUTTON,
    STD_TUI_APPLICATION_RUN_FILE_DIALOG,
    STD_TUI_APPLICATION_TEST_SET_FILE_DIALOG_RESULT,
    STD_TUI_APPLICATION_EXEC_DIALOG,
    STD_TUI_APPLICATION_MESSAGE_BOX,
    STD_TUI_APPLICATION_INPUT_TEXT,
    STD_TUI_APPLICATION_CHECKED,
    STD_TUI_APPLICATION_SELECTED,
    STD_TUI_APPLICATION_LIST_SELECTION,
    STD_TUI_APPLICATION_TEST_SET_DIALOG_RESULT,
    STD_TUI_APPLICATION_CREATE_MENU_BAR,
    STD_TUI_APPLICATION_SET_MENU_BAR,
    STD_TUI_APPLICATION_SET_MENUS,
    STD_TUI_APPLICATION_CREATE_STATUS_LINE,
    STD_TUI_APPLICATION_SET_STATUS_LINE,
    STD_TUI_APPLICATION_SET_STATUS_ITEMS,
    STD_TUI_APPLICATION_ADD_CHILD,
    STD_TUI_APPLICATION_SET_TEXT,
    STD_TUI_APPLICATION_SET_CHECKED,
    STD_TUI_APPLICATION_SET_ITEMS,
    STD_TUI_APPLICATION_SET_TITLE,
    STD_TUI_APPLICATION_ADD_WINDOW,
    STD_TUI_APPLICATION_ON_COMMAND,
    STD_TUI_APPLICATION_PUMP,
    STD_TUI_APPLICATION_QUIT,
    STD_TUI_APPLICATION_TEST_CLICK_BUTTON,
    STD_TUI_APPLICATION_TEST_DISPATCH_MENU_COMMAND,
    STD_TUI_APPLICATION_OPEN_FOR_TEST,
    STD_TUI_APPLICATION_TEST_PUMP,
    STD_TUI_APPLICATION_TEST_PUMP_UNTIL_IDLE,
    STD_TUI_APPLICATION_CLOSE_FOR_TEST,
    STD_TUI_APPLICATION_TEST_SEND_KEY,
    STD_TUI_APPLICATION_TEST_SEND_MOUSE,
    STD_TUI_APPLICATION_TEST_MOVE_MOUSE,
    STD_TUI_APPLICATION_TEST_CLICK_MOUSE,
    STD_TUI_APPLICATION_TEST_RESIZE,
    STD_TUI_APPLICATION_TEST_PASTE,
    STD_TUI_APPLICATION_TEST_FOCUS,
    STD_TUI_APPLICATION_QUERY_SCREEN_SIZE,
    STD_TUI_APPLICATION_QUERY_SCREEN_LINE,
    STD_TUI_APPLICATION_QUERY_SCREEN_CELL,
    STD_TUI_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED,
    STD_TUI_APPLICATION_HOST_INVOKE_ON_KEY_PRESSED,
    STD_TUI_APPLICATION_HOST_REGISTER_ON_RESIZE,
    STD_TUI_APPLICATION_HOST_PROCESS_NEXT,
    STD_TUI_APPLICATION_HOST_REGISTER_ON_PAINT,
    STD_TUI_APPLICATION_HOST_REGISTER_ON_IDLE,
    STD_TUI_APPLICATION_HOST_DISPATCH_REDRAW,
    STD_TUI_APPLICATION_HOST_RUN_LOOP,
    STD_TUI_APPLICATION_HOST_REQUEST_QUIT,
    STD_TUI_APPLICATION_HOST_REGISTER_ON_EXIT,
    STD_TUI_APPLICATION_HOST_REGISTER_ON_MOUSE,
    STD_TUI_APPLICATION_HOST_REGISTER_ON_PASTE,
    STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_GAINED,
    STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_LOST,
    STD_TUI_APPLICATION_HOST_REGISTER_ON_ACTIVATE,
    STD_TUI_APPLICATION_HOST_REGISTER_ON_DEACTIVATE,
    STD_TUI_APPLICATION_HOST_REGISTER_ON_COMMAND,
    STD_TUI_APPLICATION_HOST_BIND_COMMAND,
];
