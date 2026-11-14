#include "../../fpas_cui.h"

void fpas_cui_time_histogram(cui_ctx *ui, const char *id, double start, double bucket, int buckets, const cui_histogram_series * series, int series_count, double range_start, double range_end, float height, int utc, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    const cui_time_histogram_opts o = { .width = fpas_extent(width_kind, width), .height = height, .utc = utc };
    fpas_own(ui, key, key ^ 0x7415u, 0, 0);
    cui_push_disabled(ui, !enabled);
    cui_time_histogram_result r = cui_time_histogram_ex(ui,id,start,bucket,buckets,series,series_count,&range_start,&range_end,&o);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    ui->last_node = fpas_find(ui, key);
    ui->last_id = key;
    out->flags=r.selection_changed;
    out->hover=r.hovered;
    out->x=range_start;
    out->y=range_end;
}
