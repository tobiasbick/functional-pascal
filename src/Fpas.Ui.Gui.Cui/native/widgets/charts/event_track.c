#include "../../fpas_cui.h"

void fpas_cui_event_track(cui_ctx *ui, const char *id, double *times, int *count,
    int capacity, const fpas_event_options *o, int *result) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    int selected = result[0];
    const bool enabled = (o->flags & 8) != 0 && !cui_is_disabled(ui);
    if (enabled && (o->flags & 16)) ui->focus_next_request = key;
    if (!enabled) {
        cui__ev_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        if (state) state->drag = -1;
    }
    const cui_event_track_opts options = { .width = fpas_extent(o->width_kind, o->width),
        .step = o->step, .label_step = o->label_step, .units = o->units, .format = o->format,
        .selected = &selected, .no_add = (o->flags & 1) != 0, .no_remove = (o->flags & 2) != 0,
        .no_axis = (o->flags & 4) != 0 };
    cui_push_disabled(ui, !enabled);
    const cui_event_track_result native = cui_event_track_ex(ui, id, times, count, capacity,
        o->minimum, o->maximum, &options);
    cui_pop_disabled(ui);
    result[0] = native.selected;
    result[1] = native.added;
    result[2] = native.removed;
    result[3] = native.dragging;
    result[4] = enabled && native.changed;
    ui->last_node = node;
}
