#include "../fpas_cui.h"

int fpas_cui_radial_gauge(cui_ctx *ui, const char *id, float value, float minimum, float maximum,
    const fpas_radial_options *options) {
    cui_gauge_band *bands = cui__frame_alloc(ui, sizeof *bands * (size_t)(options->band_count > 0 ? options->band_count : 1));
    for (int i = 0; i < options->band_count; ++i)
        bands[i] = (cui_gauge_band){ options->bands[i].to, options->bands[i].tone, fpas_viz_text(ui, options->bands[i].name) };
    const cui_radial_gauge_opts o = { .label = options->label, .unit = options->unit, .format = options->format,
        .size = options->size, .sweep = options->sweep, .bands = bands, .band_count = options->band_count,
        .ticks = (options->flags & 1) ? 0 : -1, .target = options->target, .show_target = (options->flags & 2) != 0,
        .style = options->style, .detail = options->detail, .color = fpas_chart_color(options->palette, options->rgba) };
    const int node = ui->node_count;
    cui_radial_gauge_ex(ui, id, value, minimum, maximum, &o);
    ui->last_node = node; return node;
}
