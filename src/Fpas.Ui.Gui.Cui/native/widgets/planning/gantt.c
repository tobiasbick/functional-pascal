// [Documentation](../../../../../docs/ui/gui/planning/gantt.md)
#include "../../fpas_cui.h"

/** Refreshes task-list density when its resolved layout width crosses the native breakpoint. */
static void fpas_gantt_layout(cui_ctx *ui, cui_node *chart, void *data) {
    (void)data;
    /* CUI-083 (docs/future/cui-upstream-bugs.md#cui-083): the task list
       uses the last measured width until the next frame. */
    const bool slim = chart->rect.w > 0 && chart->rect.w < 480;
    if (chart->first >= 0 && (ui->nodes[chart->first].w.value == 128) != slim)
        cui_request_redraw(ui);
}

/** Draws the native schedule with a shared vertical viewport when height is specified. */
int fpas_cui_gantt(cui_ctx *ui, const char *id, int origin, int days, fpas_gantt_task *tasks, int count,
    int today, float day_width, float row_height, int height_kind, float height, int flags, int focus, int *parts) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), timeline = key ^ 0x5C4u;
    const cui_id viewport = key ^ 0xB6A1u;
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
        .read_only = (flags & 2) != 0 || !enabled };
    /* CUI-079 (docs/future/cui-upstream-bugs.md#cui-079): native height
       does not take effect. Scroll the native labels and bars together. */
    if (height_kind != 0) {
        const cui_box box = { .width = CUI_FILL, .height = fpas_extent(height_kind, height) };
        fpas_document_scroll_input(ui, viewport, enabled);
        const int clip = cui__scroll_begin_ex(ui, viewport, &box, false);
        ui->nodes[clip].paint = enabled ? fpas_document_scroll_layout : fpas_disabled_document_scroll_layout;
    }
    const int node = ui->node_count;
    fpas_document_scroll_input(ui, timeline, enabled);
    cui_push_disabled(ui, !enabled);
    int clicked = cui_gantt_ex(ui, id, native, count, fpas_date_from_number(origin), days, &options);
    cui_pop_disabled(ui);
    ui->nodes[node].paint = fpas_gantt_layout;
    const int scroller = fpas_find(ui, timeline);
    if (scroller >= 0)
        ui->nodes[scroller].paint = enabled ? fpas_document_scroll_layout : fpas_disabled_document_scroll_layout;
    if (height_kind != 0) cui__scroll_end_ex(ui);
    for (int i = 0; i < count; ++i) {
        tasks[i].start = native[i].start;
        tasks[i].finish = native[i].end;
        parts[i] = fpas_find(ui, fpas_well_part(key ^ 0x6A7u, i));
    }
    if (height_kind != 0) fpas_own(ui, viewport, key, timeline, 0);
    else fpas_own(ui, key, timeline, 0, 0);
    fpas_creative_node(ui, root, height_kind != 0 ? viewport : key);
    return enabled ? clicked : -1;
}
