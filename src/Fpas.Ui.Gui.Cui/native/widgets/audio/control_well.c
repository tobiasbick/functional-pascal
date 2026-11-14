static void fpas_creative_node(cui_ctx *ui, int node, cui_id key) {
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id = key;
}

static const char *fpas_well_label(cui_ctx *ui, const char *label, const char *id) {
    const char *caption = label[0] == '#' && label[1] == '#' ? "" :
        strstr(label, "##") ? "x" : label;
    return cui__frame_printf(ui, "%s###%s", caption, id);
}

static void fpas_well_literal(cui_ctx *ui, int node, const char *label) {
    if (strstr(label, "##") && !(label[0] == '#' && label[1] == '#'))
        fpas_text(ui, fpas_label_node(ui, node), label);
}

static uint32_t fpas_well_rgba(cui_color color) {
    return (uint32_t)color.r | (uint32_t)color.g << 8 | (uint32_t)color.b << 16 | (uint32_t)color.a << 24;
}

static cui_id fpas_well_part(cui_id row, int index) {
    return cui__hash(&index, sizeof index, row);
}

static void fpas_well_focus(cui_ctx *ui, bool enabled, cui_id row, cui_id key, int count, bool tool, int focus) {
    if (enabled && focus >= 0 && focus < count) ui->focus_next_request = fpas_well_part(row, focus);
    if (enabled && tool && focus == count) ui->focus_next_request = key ^ CUI__WELL_TOOL_SALT;
}

static bool fpas_well_keys(cui_ctx *ui, cui_id key) {
    if (ui->focus != key && ui->focus_next_request != key) return false;
    int layer_z = 0;
    for (int i = 0; i < ui->popup_count; ++i)
        if (ui->popups[i].id == ui->current_layer) layer_z = ui->popups[i].z;
    for (int i = 0; i < ui->popup_count; ++i)
        if (ui->popups[i].open && ui->popups[i].z > layer_z) return false;
    return true;
}
