// [Documentation](../../../../../docs/ui/gui/more-charts/README.md)
/* Frame ownership only: CUI retains these strings in deferred paint data. */
static const char *fpas_viz_text(cui_ctx *ui, const char *text) {
    return text && *text ? cui__frame_strdup(ui, text, -1) : NULL;
}

static const char **fpas_viz_labels(cui_ctx *ui, const char *const *labels, int count) {
    if (!labels || count <= 0) return NULL;
    const char **copy = cui__frame_alloc(ui, sizeof *copy * (size_t)count);
    for (int i = 0; i < count; ++i) copy[i] = fpas_viz_text(ui, labels[i]);
    return copy;
}

static bool fpas_viz_cell_gap(cui_ctx *ui, float x, float y, float cell, float gap) {
    const float step = cell + gap;
    const float dx = ui->mx - x, dy = ui->my - y;
    return dx >= 0 && dy >= 0 && (fmodf(dx, step) >= cell || fmodf(dy, step) >= cell);
}

static cui_viz_item *fpas_viz_items(cui_ctx *ui, const fpas_viz_item *items, int count) {
    cui_viz_item *copy = cui__frame_alloc(ui, sizeof *copy * (size_t)(count > 0 ? count : 1));
    for (int i = 0; i < count; ++i)
        copy[i] = (cui_viz_item){ fpas_viz_text(ui, items[i].label), items[i].value,
            fpas_chart_color(items[i].palette, items[i].rgba) };
    return copy;
}
