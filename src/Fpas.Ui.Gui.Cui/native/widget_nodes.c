#include "fpas_cui.h"

/* Shared bridge encoding: content, fill, logical pixels, percentage. */
static cui_size fpas_extent(int kind, float value) {
    switch (kind) {
        case 1: return CUI_FILL;
        case 2: return CUI_PX(value);
        case 3: return CUI_PCT(value);
        default: return CUI_FIT;
    }
}

static cui_button_variant fpas_button_variant(int variant) {
    switch (variant) {
        case 1: return CUI_BUTTON_PRIMARY;
        case 2: return CUI_BUTTON_SECONDARY;
        case 4: return CUI_BUTTON_DANGER;
        case 5: return CUI_BUTTON_OUTLINE;
        case 6: return CUI_BUTTON_GHOST;
        case 7: return CUI_BUTTON_LINK;
        default: return CUI_BUTTON_DEFAULT;
    }
}

static int fpas_find(cui_ctx *ui, cui_id id) {
    for (int i = ui->node_count - 1; i >= 0; --i) if (ui->nodes[i].id == id) return i;
    return -1;
}

static int fpas_label_node(cui_ctx *ui, int node) {
    for (int child = ui->nodes[node].first; child >= 0; child = ui->nodes[child].next) {
        if ((ui->nodes[child].flags & CUI_NF_TEXT) && ui->nodes[child].font != CUI_FONT_ICONS) return child;
        int found = fpas_label_node(ui, child);
        if (found >= 0) return found;
    }
    return -1;
}

/* Retain the upstream widget's interaction/paint but supply literal text separately from identity. */
static void fpas_text(cui_ctx *ui, int node, const char *text) {
    const int length = (int)strlen(text);
    ui->nodes[node].text = cui__frame_strdup(ui, text, length);
    ui->nodes[node].text_len = length;
}

static void fpas_literal(cui_ctx *ui, int node, const char *text, cui_color color) {
    int label = fpas_label_node(ui, node);
    if (label < 0) {
        cui__open(ui, node);
        label = cui__text_node(ui, text, -1, CUI_FONT_MEDIUM, cui__fs(ui, ui->theme.text_body), color, false);
        cui__close(ui);
    } else {
        fpas_text(ui, label, text);
    }
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}
