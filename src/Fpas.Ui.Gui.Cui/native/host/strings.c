#include "../fpas_cui.h"

typedef struct { cui_strings table; } fpas_strings;

/* The window owns this allocation: per-ID state can expire while captions remain in use. */
CUI_API void *fpas_cui_strings(cui_ctx *ui, void *previous, const char *const *values) {
    if (!values) {
        cui_set_strings(ui, NULL);
        cui__free(ui, previous);
        return NULL;
    }
    size_t bytes = 0;
    for (int i = 0; i < 46; i++) if (values[i]) bytes += strlen(values[i]) + 1;
    fpas_strings *owned = cui__alloc(ui, sizeof *owned + bytes);
    *owned = (fpas_strings){0};
    char *copy = (char *)(owned + 1);
    const char *entries[46];
    for (int i = 0; i < 46; i++) {
        entries[i] = values[i] ? copy : NULL;
        if (values[i]) { size_t n = strlen(values[i]) + 1; memcpy(copy, values[i], n); copy += n; }
    }
    for (int i = 0; i < 12; i++) { owned->table.month[i] = entries[i]; owned->table.month_short[i] = entries[12+i]; }
    for (int i = 0; i < 7; i++) {
        owned->table.weekday[i] = entries[24+i]; owned->table.weekday_short[i] = entries[31+i];
        owned->table.weekday_min[i] = entries[38+i];
    }
    owned->table.today = entries[45];
    cui_set_strings(ui, &owned->table);
    cui__free(ui, previous);
    return owned;
}
