static cui_text_fx_opts fpas_motion_options(const fpas_motion_text *o) {
    return (cui_text_fx_opts){ .font = o->font, .size = o->size,
        .color = fpas_chart_color(o->palette, o->rgba), .width = fpas_extent(o->width_kind, o->width),
        .nowrap = (o->flags & 1) != 0, .speed = o->speed, .delay = o->delay, .caret = (o->flags & 2) != 0 };
}

static void fpas_motion_node(cui_ctx *ui, int node, cui_id key) {
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id = key;
}

static void fpas_motion_replay(cui_ctx *ui, cui_id key, int revision) {
    const cui_id control = key ^ 0xF47E5u;
    int *saved = cui_state(ui, control, sizeof *saved);
    if (*saved != revision) {
        fpas_forget_state(ui, key);
        *saved = revision;
    }
    fpas_own(ui, key, control, 0, 0);
}
