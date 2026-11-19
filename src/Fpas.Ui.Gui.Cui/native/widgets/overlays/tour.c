// [Documentation](../../../../../docs/ui/gui/overlays/guided-tour.md)
#include "../../fpas_cui.h"

int fpas_cui_tour(cui_ctx *ui, const char *id, const unsigned int *targets,
    const char *const *titles, const char *const *bodies, const int *placements,
    int count, int *step, float width, float padding, int block_target,
    const char *next_label, const char *back_label, const char *done_label, const char *count_format,
    int focus, int *node, unsigned int *key, int *parts) {
    const int saved_node = ui->last_node, first = ui->node_count;
    const cui_id saved_id = ui->last_id;
    *key = cui_make_id(ui, id);
    cui_tour_step *steps = cui__frame_alloc(ui, (size_t)count * sizeof *steps);
    for (int index = 0; index < count; ++index)
        steps[index] = (cui_tour_step){ targets[index],
            cui__frame_strdup(ui, titles[index], -1), cui__frame_strdup(ui, bodies[index], -1),
            (cui_placement)placements[index] };
    const char *back = back_label ? back_label : "Back";
    const char *forward = *step == count - 1 ? (done_label ? done_label : "Done") : (next_label ? next_label : "Next");
    /* Keep a nonempty native label slot even when a literal caption consists only of hashes. */
    const char *back_id = "Back###fpas-tour-back";
    const char *forward_id = *step == count - 1 ? "Done###fpas-tour-forward" : "Next###fpas-tour-forward";
    const cui_tour_opts options = { .width = width, .padding = padding, .block_target = block_target != 0,
        .next_label = forward_id, .back_label = back_id, .done_label = forward_id, .count_format = count_format };
    cui_push_id(ui, id);
    const cui_id back_key = cui_make_id(ui, back_id), forward_key = cui_make_id(ui, forward_id);
    cui_pop_id(ui);
    if (focus) {
        cui_push_id(ui, id);
        ui->focus_next_request = focus == 1 ? cui__hash_str(CUI_ICON_X, cui_make_id(ui, "End the tour")) :
            (focus == 2 ? back_key : forward_key);
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
        if (ui->nodes[index].id == back_key) { parts[1] = index; fpas_text(ui, label, back); }
        else if (ui->nodes[index].id == forward_key) { parts[2] = index; fpas_text(ui, label, forward); }
    }
    ui->last_node = saved_node; ui->last_id = saved_id;
    return event;
}
