//! `Std.Tui` symbol names and registry group.

pub const STD_TUI_APPLICATION: &str = std_tui!("Application");
pub const STD_TUI_DIALOG: &str = std_tui!("Dialog");
pub const STD_TUI_WINDOW: &str = std_tui!("Window");
pub const STD_TUI_BUTTON: &str = std_tui!("Button");
pub const STD_TUI_STATIC_TEXT: &str = std_tui!("StaticText");
pub const STD_TUI_MEMO: &str = std_tui!("Memo");
pub const STD_TUI_TEXT_VIEWER: &str = std_tui!("TextViewer");
pub const STD_TUI_INPUT_LINE: &str = std_tui!("InputLine");
pub const STD_TUI_LIST_BOX: &str = std_tui!("ListBox");
pub const STD_TUI_OUTLINE: &str = std_tui!("Outline");
pub const STD_TUI_OUTLINE_NODE: &str = std_tui!("OutlineNode");
pub const STD_TUI_CHECK_BOX: &str = std_tui!("CheckBox");
pub const STD_TUI_RADIO_BUTTON: &str = std_tui!("RadioButton");
pub const STD_TUI_MENU_BAR: &str = std_tui!("MenuBar");
pub const STD_TUI_MENU: &str = std_tui!("Menu");
pub const STD_TUI_MENU_ITEM: &str = std_tui!("MenuItem");
pub const STD_TUI_STATUS_LINE: &str = std_tui!("StatusLine");
pub const STD_TUI_STATUS_ITEM: &str = std_tui!("StatusItem");
pub const STD_TUI_RECT: &str = std_tui!("Rect");
pub const STD_TUI_POINT: &str = std_tui!("Point");
pub const STD_TUI_SIZE: &str = std_tui!("Size");
pub const STD_TUI_CM_OK: &str = std_tui!("CM_OK");
pub const STD_TUI_CM_CANCEL: &str = std_tui!("CM_CANCEL");
pub const STD_TUI_CM_CLOSE: &str = std_tui!("CM_CLOSE");
pub const STD_TUI_CM_QUIT: &str = std_tui!("CM_QUIT");
pub const STD_TUI_CM_ABOUT: &str = std_tui!("CM_ABOUT");
pub const STD_TUI_CM_OPEN: &str = std_tui!("CM_OPEN");
pub const STD_TUI_CM_USER: &str = std_tui!("CM_USER");
pub const STD_TUI_DIALOG_NEW_MODAL: &str = std_tui!("Dialog.NewModal");
pub const STD_TUI_DIALOG_SET_TITLE: &str = std_tui!("Dialog.SetTitle");
pub const STD_TUI_DIALOG_ADD: &str = std_tui!("Dialog.Add");
pub const STD_TUI_BUTTON_NEW: &str = std_tui!("Button.New");
pub const STD_TUI_BUTTON_SET_TEXT: &str = std_tui!("Button.SetText");
pub const STD_TUI_WINDOW_NEW: &str = std_tui!("Window.New");
pub const STD_TUI_WINDOW_SET_TITLE: &str = std_tui!("Window.SetTitle");
pub const STD_TUI_WINDOW_ADD: &str = std_tui!("Window.Add");
pub const STD_TUI_DESKTOP_ADD: &str = std_tui!("Desktop.Add");
pub const STD_TUI_STATIC_TEXT_NEW: &str = std_tui!("StaticText.New");
pub const STD_TUI_STATIC_TEXT_SET_TEXT: &str = std_tui!("StaticText.SetText");
pub const STD_TUI_CHECK_BOX_NEW: &str = std_tui!("CheckBox.New");
pub const STD_TUI_INPUT_LINE_NEW: &str = std_tui!("InputLine.New");
pub const STD_TUI_CHECK_BOX_CHECKED: &str = std_tui!("CheckBox.Checked");
pub const STD_TUI_CHECK_BOX_SET_CHECKED: &str = std_tui!("CheckBox.SetChecked");
pub const STD_TUI_INPUT_LINE_TEXT: &str = std_tui!("InputLine.Text");
pub const STD_TUI_INPUT_LINE_SET_TEXT: &str = std_tui!("InputLine.SetText");
pub const STD_TUI_OUTLINE_NEW: &str = std_tui!("Outline.New");
pub const STD_TUI_OUTLINE_SELECTION: &str = std_tui!("Outline.Selection");
pub const STD_TUI_OUTLINE_SELECTED_TEXT: &str = std_tui!("Outline.SelectedText");
pub const STD_TUI_OUTLINE_SET_NODES: &str = std_tui!("Outline.SetNodes");
pub const STD_TUI_LIST_BOX_NEW: &str = std_tui!("ListBox.New");
pub const STD_TUI_LIST_BOX_SELECTION: &str = std_tui!("ListBox.Selection");
pub const STD_TUI_LIST_BOX_SET_ITEMS: &str = std_tui!("ListBox.SetItems");
pub const STD_TUI_RADIO_BUTTON_NEW: &str = std_tui!("RadioButton.New");
pub const STD_TUI_RADIO_BUTTON_SELECTED: &str = std_tui!("RadioButton.Selected");
pub const STD_TUI_RADIO_BUTTON_SET_SELECTED: &str = std_tui!("RadioButton.SetSelected");
pub const STD_TUI_MEMO_NEW: &str = std_tui!("Memo.New");
pub const STD_TUI_MEMO_SET_TEXT: &str = std_tui!("Memo.SetText");
pub const STD_TUI_TEXT_VIEWER_NEW: &str = std_tui!("TextViewer.New");
pub const STD_TUI_TEXT_VIEWER_SET_TEXT: &str = std_tui!("TextViewer.SetText");
pub const STD_TUI_MENU_BAR_NEW: &str = std_tui!("MenuBar.New");
pub const STD_TUI_MENU_BAR_SET_MENUS: &str = std_tui!("MenuBar.SetMenus");
pub const STD_TUI_STATUS_LINE_NEW: &str = std_tui!("StatusLine.New");
pub const STD_TUI_STATUS_LINE_SET_ITEMS: &str = std_tui!("StatusLine.SetItems");
pub const STD_TUI_MESSAGE_BOX_OPTION_WARNING: &str = std_tui!("MessageBoxOption.Warning");
pub const STD_TUI_MESSAGE_BOX_OPTION_ERROR: &str = std_tui!("MessageBoxOption.Error");
pub const STD_TUI_MESSAGE_BOX_OPTION_INFORMATION: &str = std_tui!("MessageBoxOption.Information");
pub const STD_TUI_MESSAGE_BOX_OPTION_CONFIRMATION: &str = std_tui!("MessageBoxOption.Confirmation");
pub const STD_TUI_MESSAGE_BOX_OPTION_ABOUT: &str = std_tui!("MessageBoxOption.About");
pub const STD_TUI_MESSAGE_BOX_OPTION_YES_BUTTON: &str = std_tui!("MessageBoxOption.YesButton");
pub const STD_TUI_MESSAGE_BOX_OPTION_NO_BUTTON: &str = std_tui!("MessageBoxOption.NoButton");
pub const STD_TUI_MESSAGE_BOX_OPTION_OK_BUTTON: &str = std_tui!("MessageBoxOption.OkButton");
pub const STD_TUI_MESSAGE_BOX_OPTION_CANCEL_BUTTON: &str =
    std_tui!("MessageBoxOption.CancelButton");
pub const STD_TUI_MESSAGE_BOX_OPTION_YES_NO_CANCEL: &str = std_tui!("MessageBoxOption.YesNoCancel");
pub const STD_TUI_MESSAGE_BOX_OPTION_OK_CANCEL: &str = std_tui!("MessageBoxOption.OkCancel");
pub const STD_TUI_APPLICATION_OPEN: &str = std_tui!("Application.Open");
pub const STD_TUI_APPLICATION_NEW: &str = std_tui!("Application.New");
pub const STD_TUI_APPLICATION_CLOSE: &str = std_tui!("Application.Close");
/// Dispatch-mode hosted application loop; see `docs/pascal/std/tui/app/README.md`.
pub const STD_TUI_APPLICATION_RUN: &str = std_tui!("Application.Run");
pub const STD_TUI_APPLICATION_SIZE: &str = std_tui!("Application.Size");
/// Show a modal Turbo Vision file dialog and return the selected path, or `None` when canceled.
pub const STD_TUI_APPLICATION_RUN_FILE_DIALOG: &str = std_tui!("Application.RunFileDialog");
/// Queue the result returned by the next headless `Application.RunFileDialog` call.
pub const STD_TUI_APPLICATION_TEST_SET_FILE_DIALOG_RESULT: &str =
    std_tui!("Application.TestSetFileDialogResult");
/// Run a modal dialog via try-2 `Application.ExecView` and return the closing command id.
pub const STD_TUI_APPLICATION_EXEC_VIEW: &str = std_tui!("Application.ExecView");
/// Headless test helper: queue a keyboard event.
pub const STD_TUI_TEST_INJECT_KEYBOARD: &str = std_tui!("Test.InjectKeyboard");
/// Headless test helper: queue a command for the next headless run-loop turn.
pub const STD_TUI_TEST_INJECT_COMMAND: &str = std_tui!("Test.InjectCommand");
/// Show an upstream Turbo Vision message box and return the closing command id.
pub const STD_TUI_APPLICATION_MESSAGE_BOX: &str = std_tui!("Application.MessageBox");
/// Queue the closing command returned by the next headless modal call.
pub const STD_TUI_APPLICATION_TEST_SET_DIALOG_RESULT: &str =
    std_tui!("Application.TestSetDialogResult");
pub const STD_TUI_APPLICATION_SET_MENU_BAR: &str = std_tui!("Application.SetMenuBar");
pub const STD_TUI_APPLICATION_SET_STATUS_LINE: &str = std_tui!("Application.SetStatusLine");
/// Register `function (Application, Std.Console.KeyEvent): boolean` for unhandled Turbo Vision keys.
pub const STD_TUI_APPLICATION_ON_KEY: &str = std_tui!("Application.OnKey");
/// Register `procedure (Application, Std.Console.Event)` for unhandled Turbo Vision mouse events.
pub const STD_TUI_APPLICATION_ON_MOUSE: &str = std_tui!("Application.OnMouse");
pub const STD_TUI_APPLICATION_QUIT: &str = std_tui!("Application.Quit");
/// Headless test helper: queue a button click at the button center.
pub const STD_TUI_TEST_CLICK: &str = std_tui!("Test.Click");
/// Headless test helper: dispatch a menu item command (`MenuIndex` / `ItemIndex` into menu bar data).
pub const STD_TUI_TEST_DISPATCH_MENU: &str = std_tui!("Test.DispatchMenu");
pub const STD_TUI_APPLICATION_OPEN_FOR_TEST: &str = std_tui!("Application.OpenForTest");
pub const STD_TUI_APPLICATION_CLOSE_FOR_TEST: &str = std_tui!("Application.CloseForTest");
pub const STD_TUI_APPLICATION_TEST_CLICK_MOUSE: &str = std_tui!("Application.TestClickMouse");

pub(in crate::std_units) const STD_TUI_SYMBOLS: &[&str] = &[
    STD_TUI_APPLICATION,
    STD_TUI_DIALOG,
    STD_TUI_WINDOW,
    STD_TUI_BUTTON,
    STD_TUI_STATIC_TEXT,
    STD_TUI_MEMO,
    STD_TUI_TEXT_VIEWER,
    STD_TUI_INPUT_LINE,
    STD_TUI_LIST_BOX,
    STD_TUI_OUTLINE,
    STD_TUI_OUTLINE_NODE,
    STD_TUI_CHECK_BOX,
    STD_TUI_RADIO_BUTTON,
    STD_TUI_MENU_BAR,
    STD_TUI_MENU,
    STD_TUI_MENU_ITEM,
    STD_TUI_STATUS_LINE,
    STD_TUI_STATUS_ITEM,
    STD_TUI_RECT,
    STD_TUI_POINT,
    STD_TUI_SIZE,
    STD_TUI_CM_OK,
    STD_TUI_CM_CANCEL,
    STD_TUI_CM_CLOSE,
    STD_TUI_CM_QUIT,
    STD_TUI_CM_ABOUT,
    STD_TUI_CM_OPEN,
    STD_TUI_CM_USER,
    STD_TUI_DIALOG_NEW_MODAL,
    STD_TUI_DIALOG_SET_TITLE,
    STD_TUI_DIALOG_ADD,
    STD_TUI_BUTTON_NEW,
    STD_TUI_BUTTON_SET_TEXT,
    STD_TUI_WINDOW_NEW,
    STD_TUI_WINDOW_SET_TITLE,
    STD_TUI_WINDOW_ADD,
    STD_TUI_DESKTOP_ADD,
    STD_TUI_STATIC_TEXT_NEW,
    STD_TUI_STATIC_TEXT_SET_TEXT,
    STD_TUI_CHECK_BOX_NEW,
    STD_TUI_INPUT_LINE_NEW,
    STD_TUI_CHECK_BOX_CHECKED,
    STD_TUI_CHECK_BOX_SET_CHECKED,
    STD_TUI_INPUT_LINE_TEXT,
    STD_TUI_INPUT_LINE_SET_TEXT,
    STD_TUI_OUTLINE_NEW,
    STD_TUI_OUTLINE_SELECTION,
    STD_TUI_OUTLINE_SELECTED_TEXT,
    STD_TUI_OUTLINE_SET_NODES,
    STD_TUI_LIST_BOX_NEW,
    STD_TUI_LIST_BOX_SELECTION,
    STD_TUI_LIST_BOX_SET_ITEMS,
    STD_TUI_RADIO_BUTTON_NEW,
    STD_TUI_RADIO_BUTTON_SELECTED,
    STD_TUI_RADIO_BUTTON_SET_SELECTED,
    STD_TUI_MEMO_NEW,
    STD_TUI_MEMO_SET_TEXT,
    STD_TUI_TEXT_VIEWER_NEW,
    STD_TUI_TEXT_VIEWER_SET_TEXT,
    STD_TUI_MENU_BAR_NEW,
    STD_TUI_MENU_BAR_SET_MENUS,
    STD_TUI_STATUS_LINE_NEW,
    STD_TUI_STATUS_LINE_SET_ITEMS,
    STD_TUI_MESSAGE_BOX_OPTION_WARNING,
    STD_TUI_MESSAGE_BOX_OPTION_ERROR,
    STD_TUI_MESSAGE_BOX_OPTION_INFORMATION,
    STD_TUI_MESSAGE_BOX_OPTION_CONFIRMATION,
    STD_TUI_MESSAGE_BOX_OPTION_ABOUT,
    STD_TUI_MESSAGE_BOX_OPTION_YES_BUTTON,
    STD_TUI_MESSAGE_BOX_OPTION_NO_BUTTON,
    STD_TUI_MESSAGE_BOX_OPTION_OK_BUTTON,
    STD_TUI_MESSAGE_BOX_OPTION_CANCEL_BUTTON,
    STD_TUI_MESSAGE_BOX_OPTION_YES_NO_CANCEL,
    STD_TUI_MESSAGE_BOX_OPTION_OK_CANCEL,
    STD_TUI_APPLICATION_OPEN,
    STD_TUI_APPLICATION_NEW,
    STD_TUI_APPLICATION_CLOSE,
    STD_TUI_APPLICATION_RUN,
    STD_TUI_APPLICATION_SIZE,
    STD_TUI_APPLICATION_RUN_FILE_DIALOG,
    STD_TUI_APPLICATION_TEST_SET_FILE_DIALOG_RESULT,
    STD_TUI_APPLICATION_EXEC_VIEW,
    STD_TUI_TEST_INJECT_KEYBOARD,
    STD_TUI_TEST_INJECT_COMMAND,
    STD_TUI_APPLICATION_MESSAGE_BOX,
    STD_TUI_APPLICATION_TEST_SET_DIALOG_RESULT,
    STD_TUI_APPLICATION_SET_MENU_BAR,
    STD_TUI_APPLICATION_SET_STATUS_LINE,
    STD_TUI_APPLICATION_ON_KEY,
    STD_TUI_APPLICATION_ON_MOUSE,
    STD_TUI_APPLICATION_QUIT,
    STD_TUI_TEST_CLICK,
    STD_TUI_TEST_DISPATCH_MENU,
    STD_TUI_APPLICATION_OPEN_FOR_TEST,
    STD_TUI_APPLICATION_CLOSE_FOR_TEST,
    STD_TUI_APPLICATION_TEST_CLICK_MOUSE,
];
