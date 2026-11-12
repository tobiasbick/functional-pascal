#include "fpas_cui.h"

int fpas_cui_open_file(cui_app *app, const char *title, const char *filter_name,
    const char *accept, char *path, size_t capacity) {
    const cui_file_dialog_opts options = {
        .kind = CUI_FILE_OPEN, .title = title, .filter_name = filter_name, .filter = accept
    };
    return cui_app_file_dialog(app, &options, path, capacity) && cui_path_has_extension(path, accept);
}
