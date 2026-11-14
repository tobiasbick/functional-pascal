#include "../../fpas_cui.h"

int fpas_cui_sankey(cui_ctx *ui, const char *id, const fpas_sankey_node *nodes, int count,
    const cui_sankey_link *links, int link_count, int width_kind, float width, float height,
    float node_width, const char *format, int enabled, int *node) {
    const char **names = cui__frame_alloc(ui, sizeof *names * (size_t)(count > 0 ? count : 1));
    cui_color *colors = cui__frame_alloc(ui, sizeof *colors * (size_t)(count > 0 ? count : 1));
    for (int i = 0; i < count; ++i) {
        names[i] = fpas_viz_text(ui, nodes[i].label);
        colors[i] = fpas_chart_color(nodes[i].palette, nodes[i].rgba);
    }
    const cui_sankey_opts o = { .width = fpas_extent(width_kind, width), .height = height,
        .node_width = node_width, .format = fpas_viz_text(ui, format), .node_colors = colors };
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const cui_id key = cui_make_id(ui, id);
    const cui_interact interaction = cui__interact(ui, key, 0);
    int clicked = cui_sankey_ex(ui, id, names, count, links, link_count, &o);
    /* CUI-021 (docs/future/cui-upstream-bugs.md#cui-021): native return omits clicked links. */
    const cui__viz_state *state = cui_state(ui, key, sizeof *state);
    if (interaction.clicked && clicked < 0 && state->hover >= count) clicked = state->hover;
    cui_pop_disabled(ui); ui->last_node = *node;
    /* Native link indices omit zero flows; report the original input position. */
    if (clicked >= count) {
        int rank = clicked - count;
        for (int i = 0; i < link_count; ++i)
            if (links[i].value > 0 && --rank < 0) return count + i;
        return -1;
    }
    return clicked;
}
