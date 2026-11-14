#include "../../fpas_cui.h"

int fpas_cui_search_field(cui_ctx *ui, const char *id, char *value, size_t capacity,
    const char *label, const char *placeholder, const char *icon, const char *hint, const char *error,
    const char *prefix, const char *suffix, int size, int read_only, int enabled, int focus, int *nodes) {
    const cui_input_opts options = {
        .label = label, .placeholder = placeholder, .icon = icon, .hint = hint, .error = error,
        .prefix = prefix, .suffix = suffix, .size = (cui_control_size)size,
        .clearable = true, .read_only = read_only != 0
    };
    int revealed;
    return fpas_draw_text_editor(ui, id, value, capacity, options, false, enabled, focus, nodes, &revealed, cui_search_ex);
}
