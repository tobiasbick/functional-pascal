#include "../../fpas_cui.h"

void fpas_cui_time_histogram(cui_ctx *ui, const char *id, double start, double bucket,
    int buckets, const cui_histogram_series *series, int series_count, double range_start,
    double range_end, float height, int utc, int width_kind, float width, int enabled,
    fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    const cui_time_histogram_opts options = {
        .width = fpas_extent(width_kind, width), .height = height, .utc = utc
    };
    *out = fpas_dev_empty();

    /* The outer widget includes the legend. Give the native plot its own geometry and drag state. */
    cui_push_id(ui, id);
    const cui_id plot = cui_make_id(ui, "plot");
    const cui_id drag = plot ^ 0x7415u;
    fpas_own(ui, key, plot, drag, 0);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    owner->hover = plot;
    cui_push_disabled(ui, !allowed);
    const cui_time_histogram_result result = cui_time_histogram_ex(ui, "plot", start, bucket,
        buckets, series, series_count, &range_start, &range_end, &options);
    cui_pop_disabled(ui);
    cui_pop_id(ui);
    fpas_creative_node(ui, root, key);
    out->flags = allowed && result.selection_changed;
    out->hover = result.hovered;
    out->x = range_start;
    out->y = range_end;
}
