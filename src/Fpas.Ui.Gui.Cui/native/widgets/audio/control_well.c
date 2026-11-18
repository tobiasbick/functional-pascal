static void fpas_creative_node(cui_ctx *ui, int node, cui_id key) {
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id = key;
}

/* A trailing hash would merge with CUI's identity delimiter. */
static bool fpas_well_needs_literal(const char *label) {
    const size_t length = strlen(label);
    return length && strncmp(label, "##", 2) && (label[length - 1] == '#' || strstr(label, "##"));
}

static const char *fpas_well_label(cui_ctx *ui, const char *label, const char *id) {
    const char *caption = strncmp(label, "##", 2) == 0 ? "" :
        fpas_well_needs_literal(label) ? "x" : label;
    return cui__frame_printf(ui, "%s###%s", caption, id);
}

static void fpas_well_literal(cui_ctx *ui, int node, const char *label) {
    if (fpas_well_needs_literal(label))
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
