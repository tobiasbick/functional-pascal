/* Only representation conversion; layout, animation and input remain native. */
static cui_color fpas_chart_color(int palette, uint32_t rgba) {
    if (palette > 0) return CUI_TOKEN_COLOR(CUI_TOKEN_SERIES_1 + palette - 1);
    if (palette == -1) return (cui_color){ (uint8_t)rgba, (uint8_t)(rgba >> 8), (uint8_t)(rgba >> 16), (uint8_t)(rgba >> 24) };
    return (cui_color){0};
}

static cui_chart_opts fpas_chart_options_of(cui_ctx *ui, const fpas_chart_options *o) {
    cui_chart_opts result = {
        .width = fpas_extent(o->width_kind, o->width), .height = o->height,
        .labels = o->label_count > 0 ? o->labels : NULL, .label_count = o->label_count,
        .format = o->format, .min = o->minimum, .max = o->maximum,
        .area = (o->flags & 1) != 0, .no_grid = (o->flags & 2) != 0, .no_legend = (o->flags & 4) != 0,
        .stacked = (o->flags & 8) != 0, .smooth = (o->flags & 16) != 0, .dither = (o->flags & 32) != 0
    };
    if (o->annotation_count > 0) {
        cui_chart_annotation *annotations = cui__frame_alloc(ui, sizeof *annotations * (size_t)o->annotation_count);
        for (int i = 0; i < o->annotation_count; ++i) {
            const fpas_chart_annotation *a = &o->annotations[i];
            annotations[i] = (cui_chart_annotation){ .kind = a->kind, .x = a->x, .y = a->y,
                .label = a->label && *a->label ? a->label : NULL,
                .detail = a->detail && *a->detail ? a->detail : NULL, .accent = a->accent != 0 };
        }
        result.annotations = annotations; result.annotation_count = o->annotation_count;
    }
    return result;
}

static cui_series *fpas_chart_series_of(cui_ctx *ui, const fpas_chart_series *series, int count) {
    cui_series *native = cui__frame_alloc(ui, sizeof *native * (size_t)(count > 0 ? count : 1));
    for (int i = 0; i < count; ++i)
        native[i] = (cui_series){ .name = series[i].name, .values = series[i].values, .count = series[i].count,
            .color = fpas_chart_color(series[i].palette, series[i].rgba) };
    return native;
}
