#include "../fpas_cui.h"

int fpas_cui_tabs(cui_ctx *ui, const char *id, const char *const *labels,
    const char *const *icons, int count, int selected, int variant, int full_width,
    int enabled, int focus, int *nodes) {
    const cui_id group = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    const bool scrolls = !(variant == CUI_TABS_PILL && full_width);
    const cui_id viewport = group ^ 0x7AB5u;
    const char *captions[32], *glyphs[32];
    bool literal = false;
    for (int index = 0; index < count; ++index) {
        const bool quoted = strstr(labels[index], "##") != NULL;
        captions[index] = quoted ? "x" : labels[index];
        glyphs[index] = icons[index][0] ? icons[index] : NULL;
        literal |= quoted;
    }
    fpas_indexed_focus(ui, group, count, allowed ? focus : 0);
    const cui_tabs_opts options = {
        .variant = (cui_tabs_variant)variant, .icons = glyphs, .full_width = full_width != 0
    };
    int choice = selected;
    const bool suppress_wheel = !allowed && scrolls && ui->scroll_target == viewport;
    const float wheel_x = ui->scroll_dx, wheel_y = ui->scroll_dy;
    if (suppress_wheel) ui->scroll_dx = ui->scroll_dy = 0;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_tabs_ex(ui, id, captions, count, &choice, &options);
    cui_pop_disabled(ui);
    if (suppress_wheel) { ui->scroll_dx = wheel_x; ui->scroll_dy = wheel_y; }

    nodes[0] = fpas_find(ui, group);
    for (int index = 0; index < count; ++index) {
        nodes[index + 1] = fpas_find(ui, cui__hash(&index, sizeof index, group));
        if (captions[index] != labels[index])
            fpas_literal(ui, nodes[index + 1], labels[index], CUI_C(ui, FOREGROUND));
    }
    if (literal) ui->nodes[nodes[0]].pref_hint = 0;
    /* Track the strip identity across both the scrolling and full-width pill layouts. */
    const int root = scrolls ? fpas_find(ui, viewport) : nodes[0];
    fpas_own(ui, group, scrolls ? viewport : 0, scrolls ? viewport ^ 1u : 0, 0);
    if (!enabled) ui->nodes[root].opacity = CUI__DISABLED;
    ui->last_node = root;
    ui->last_id = ui->nodes[root].id;
    return allowed && changed ? choice : -1;
}
