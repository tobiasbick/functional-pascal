#include "../../fpas_cui.h"
#include "chrome_input.h"

void fpas_cui_document_tabs(cui_ctx *ui, const char *id, const char *const *titles,
    const char *const *icons, const char *const *tips, const int *dirty, int count, int *selected,
    int flags, int *close, int *from, int *to, int *nodes) {
    cui_doc_tab *tabs = count ? cui__frame_alloc(ui, (size_t)count * sizeof *tabs) : NULL;
    for (int i = 0; i < count; ++i)
        tabs[i] = (cui_doc_tab){ cui__frame_strdup(ui, titles[i], -1),
            icons[i][0] ? cui__frame_strdup(ui, icons[i], -1) : NULL,
            tips[i][0] ? cui__frame_strdup(ui, tips[i], -1) : NULL, dirty[i] != 0 };
    const bool enabled = (flags & 8) && !cui_is_disabled(ui);
    const cui_doc_tabs_opts opts = { .shortcuts = enabled && (flags & 1),
        .no_close = (flags & 2) != 0, .no_reorder = (flags & 4) != 0 };
    const cui_id key = cui_make_id(ui, id);
    if (enabled && (flags & 16) && *selected >= 0 && *selected < count) {
        cui_push_id(ui, id);
        ui->focus_next_request = cui_make_id(ui, tabs[*selected].title);
        cui_pop_id(ui);
    }
    fpas_chrome_input input;
    if (!enabled) {
        cui__dtabs_state *state = cui_state(ui, key, sizeof *state);
        state->pending = false;
        state->from = -1;
        fpas_chrome_pause(ui, &input);
    }
    cui_push_disabled(ui, !enabled);
    const cui_doc_tabs_result result = cui_doc_tabs_ex(ui, id, tabs, count, selected, &opts);
    cui_pop_disabled(ui);
    if (!enabled) fpas_chrome_resume(ui, &input);
    *close = enabled ? result.close : -1;
    *from = enabled ? result.move_from : -1;
    *to = enabled ? result.move_to : -1;
    nodes[0] = fpas_find(ui, key ^ 0xD0C7u);
    cui_push_id(ui, id);
    for (int i = 0; i < count; ++i) {
        const cui_id part = cui_make_id(ui, tabs[i].title);
        nodes[i + 1] = fpas_find(ui, part);
        if (nodes[i + 1] >= 0) fpas_own(ui, part, part ^ 0x1u, 0, 0);
    }
    cui_pop_id(ui);
    if (nodes[0] >= 0) fpas_own(ui, key ^ 0xD0C7u, key, 0, 0);
    ui->last_node = nodes[0];
}
