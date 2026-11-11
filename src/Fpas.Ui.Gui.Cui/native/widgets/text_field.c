#include "../fpas_cui.h"

int fpas_cui_text_field(cui_ctx *ui, const char *id, char *value, size_t capacity,
    const char *label, const char *placeholder, const char *icon, const char *hint, const char *error,
    const char *prefix, const char *suffix, int size, int password, int clearable,
    int read_only, int enabled, int focus, int *nodes, int *revealed) {
    const cui_input_opts options = {
        .label = label, .placeholder = placeholder, .icon = icon, .hint = hint, .error = error,
        .prefix = prefix, .suffix = suffix, .size = (cui_control_size)size,
        .password = password != 0, .clearable = clearable != 0, .read_only = read_only != 0
    };
    return fpas_draw_text_editor(ui, id, value, capacity, options, false, enabled, focus, nodes, revealed, cui_input_ex);
}
