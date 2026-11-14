#include "../../fpas_cui.h"

int fpas_cui_pattern_meter(cui_ctx *ui, const char *id, const fpas_meter_part *parts, int count,
    const fpas_meter_options *options, int enabled, int focus, int *node, unsigned int *key) {
    *key = cui_make_id(ui, id);
    const cui_id bar = *key ^ 0x9E7E5u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (!allowed && ui->focus == bar) ui->focus = 0;
    if (allowed && focus) ui->focus_next_request = bar;
    const cui_pattern_meter_opts o = {
        .total = options->total, .label = options->label, .unit = options->unit, .format = options->format,
        .free_label = options->free_label, .grayscale = (options->flags & 1) != 0,
        .no_legend = (options->flags & 2) != 0, .height = options->height,
        .width = fpas_extent(options->width_kind, options->width)
    };
    cui_meter_part *native = cui__frame_alloc(ui, sizeof *native * (size_t)(count > 0 ? count : 1));
    for (int i = 0; i < count; ++i)
        native[i] = (cui_meter_part){ .name = parts[i].name, .value = parts[i].value,
            .color = fpas_chart_color(parts[i].palette, parts[i].rgba), .density = parts[i].density };
    *node = ui->node_count;
    cui_push_disabled(ui, !allowed);
    const int shown = cui_pattern_meter_ex(ui, id, native, count, &o);
    cui_pop_disabled(ui);
    fpas_own(ui, *key, 0, 0, 0);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(*key), sizeof *owner);
    owner->focus = bar;
    ui->last_node = *node; ui->last_id = *key;
    return shown;
}
