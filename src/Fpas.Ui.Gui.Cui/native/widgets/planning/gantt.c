#include "../../fpas_cui.h"

int fpas_cui_gantt(cui_ctx *ui, const char *id, int origin, int days, fpas_gantt_task *tasks, int count,
    int today, float day_width, float row_height, int height_kind, float height, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && focus >= 0 && focus < count)
        ui->focus_next_request = fpas_well_part(key ^ 0x6A7u, focus);
    cui_gantt_task *native = cui__frame_alloc(ui, sizeof *native * (size_t)CUI_MAX(count, 1));
    for (int i = 0; i < count; ++i) {
        const fpas_gantt_task *t = &tasks[i];
        native[i] = (cui_gantt_task){ cui__frame_strdup(ui, t->name, -1),
            t->owner ? cui__frame_strdup(ui, t->owner, -1) : NULL, t->start, t->finish,
            t->progress, t->tone, t->dependency, t->milestone != 0, t->state };
    }
    const cui_gantt_opts options = { .today = today, .day_width = day_width, .row_height = row_height,
        .height = fpas_extent(height_kind, height), .read_only = (flags & 2) != 0 || !enabled };
    cui__gantt_state *state = cui__state_lazy(ui, key, sizeof *state, false);
    if (state && !enabled) state->drag = 0;
    cui_push_disabled(ui, !enabled);
    int clicked = cui_gantt_ex(ui, id, native, count, fpas_date_from_number(origin), days, &options);
    cui_pop_disabled(ui);
    for (int i = 0; i < count; ++i) {
        tasks[i].start = native[i].start;
        tasks[i].finish = native[i].end;
        parts[i] = fpas_find(ui, fpas_well_part(key ^ 0x6A7u, i));
        /* CUI-032 (docs/future/cui-upstream-bugs.md#cui-032): keep keyboard
           activation available when schedule editing is off. */
        if (enabled && options.read_only && clicked < 0 && parts[i] >= node && ui->focus_visible &&
            fpas_well_keys(ui, ui->nodes[parts[i]].id) &&
            (cui_key_pressed(ui, CUI_KEY_ENTER) || cui_key_pressed(ui, CUI_KEY_SPACE))) clicked = i;
    }
    fpas_creative_node(ui, node, key);
    return enabled ? clicked : -1;
}
