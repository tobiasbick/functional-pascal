#include "../fpas_cui.h"

int fpas_cui_tour(cui_ctx *ui, const char *id, const unsigned int *targets,
    const char *const *titles, const char *const *bodies, const int *placements,
    int count, int *step, float width, float padding, int block_target,
    int focus, int *node, unsigned int *key, int *parts) {
    const int saved_node = ui->last_node, first = ui->node_count;
    const cui_id saved_id = ui->last_id;
    *key = cui_make_id(ui, id);
    cui_tour_step *steps = cui__frame_alloc(ui, (size_t)count * sizeof *steps);
    for (int index = 0; index < count; ++index)
        steps[index] = (cui_tour_step){ targets[index],
            cui__frame_strdup(ui, titles[index], -1), cui__frame_strdup(ui, bodies[index], -1),
            (cui_placement)placements[index] };
    const cui_tour_opts options = { .width = width, .padding = padding, .block_target = block_target != 0 };
    if (focus) {
        cui_push_id(ui, id);
        ui->focus_next_request = focus == 1 ? cui__hash_str(CUI_ICON_X, cui_make_id(ui, "End the tour")) :
            cui_make_id(ui, focus == 2 ? "Back" : *step == count - 1 ? "Done" : "Next");
        ui->focus_visible = true;
        cui_pop_id(ui);
    }
    const int event = cui_tour_ex(ui, id, steps, count, step, &options);
    *node = fpas_find(ui, *key ^ 0xCA2Du);
    parts[0] = parts[1] = parts[2] = -1;
    for (int index = first; index < ui->node_count; ++index) {
        if (!(ui->nodes[index].flags & CUI_NF_INTERACTIVE)) continue;
        const int label = fpas_label_node(ui, index);
        if (label < 0) {
            const int icon = ui->nodes[index].first;
            if (icon >= 0 && ui->nodes[icon].font == CUI_FONT_ICONS &&
                ui->nodes[icon].text && !strcmp(ui->nodes[icon].text, CUI_ICON_X)) parts[0] = index;
            continue;
        }
        if (!ui->nodes[label].text) continue;
        const char *text = ui->nodes[label].text;
        if (!strcmp(text, "Back")) parts[1] = index;
        else if (!strcmp(text, "Next") || !strcmp(text, "Done")) parts[2] = index;
    }
    ui->last_node = saved_node; ui->last_id = saved_id;
    return event;
}
