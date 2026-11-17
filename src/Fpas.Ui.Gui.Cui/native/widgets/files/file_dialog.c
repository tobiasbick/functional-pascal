#include "../../fpas_cui.h"

int fpas_cui_file_dialog(cui_app *app, int kind, const char *title, const char *filter_name,
    const char *accept, const char *default_name, char *path, size_t capacity) {
    const cui_file_dialog_opts options = {
        .kind = (cui_file_dialog_kind)kind, .title = title, .filter_name = filter_name,
        .filter = accept, .default_name = default_name
    };
    return cui_app_file_dialog(app, &options, path, capacity) &&
        (kind != CUI_FILE_OPEN || cui_path_has_extension(path, accept));
}
