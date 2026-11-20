// [Documentation](../../../../../docs/ui/gui/cui.md#desktop-services)
#include "../../fpas_cui.h"

#ifdef _WIN32
#include "win32_file_dialog.c"
#endif

int fpas_cui_file_dialog(cui_app *app, int kind, const char *title, const char *filter_name,
    const char *accept, const char *default_name, char *path, size_t capacity) {
    const cui_file_dialog_opts options = {
        .kind = (cui_file_dialog_kind)kind, .title = title, .filter_name = filter_name,
        .filter = accept, .default_name = default_name
    };
#ifdef _WIN32
    if (app && (kind == CUI_FILE_SAVE || kind == CUI_FILE_FOLDER)) {
        bool selected = fpas__win32_file_dialog(cui__w(app)->hwnd, &options, path, capacity);
        /* A modal chooser runs its own message loop between application frames. */
        cui_input_focus(app->ui, false);
        cui_input_focus(app->ui, true);
        cui_input_mouse_leave(app->ui);
        app->last_time = cui__plat_now();
        cui_request_redraw(app->ui);
        return selected;
    }
#endif
    return cui_app_file_dialog(app, &options, path, capacity) &&
        (kind != CUI_FILE_OPEN || cui_path_has_extension(path, accept));
}
