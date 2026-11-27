// [Documentation](../../../../../docs/ui/gui/overlays/guided-tour.md)
#include "../../fpas_cui.h"

typedef struct {
    cui_id previous, close, back, forward, target;
    bool active, visible;
} fpas_tour_focus;

/** Restores the preceding focus if the tour still owns it. */
static void fpas_tour_restore_focus(cui_ctx *ui, const fpas_tour_focus *state) {
    if (ui->focus && ui->focus != state->close && ui->focus != state->back &&
        ui->focus != state->forward && ui->focus != state->target) return;
    ui->focus = state->previous;
    ui->focus_visible = state->visible;
}

/** Restores focus when the model removes the overlay. */
static void fpas_tour_forget(cui_ctx *ui, cui_id key) {
    const fpas_tour_focus *state = cui__state_lazy(ui, key ^ 0x7A0Fu, sizeof *state, false);
    if (state && state->active) fpas_tour_restore_focus(ui, state);
}

/** Builds the native tour and tracks its explanation and buttons. */
int fpas_cui_tour(cui_ctx *ui, const char *id, const unsigned int *targets,
    const char *const *titles, const char *const *bodies, const int *placements,
    int count, int *step, float width, float padding, int block_target,
    const char *next_label, const char *back_label, const char *done_label, const char *count_format,
    int focus, int *node, unsigned int *key, int *parts) {
    *node = parts[0] = parts[1] = parts[2] = -1;
    *key = cui_make_id(ui, id);
    if (!step || *step < 0 || count <= 0) return CUI_TOUR_NONE;
    *step = CUI_MIN(*step, count - 1);
    const int saved_node = ui->last_node;
    const cui_id saved_id = ui->last_id;
    cui_tour_step *steps = cui__frame_alloc(ui, (size_t)count * sizeof *steps);
    for (int index = 0; index < count; ++index)
        steps[index] = (cui_tour_step){ targets[index],
            cui__frame_strdup(ui, titles[index], -1), cui__frame_strdup(ui, bodies[index], -1),
            (cui_placement)placements[index] };
    const char *back = back_label ? back_label : "Back";
    const char *forward = *step == count - 1 ? (done_label ? done_label : "Done") : (next_label ? next_label : "Next");
    /* Keep a label slot for literal captions that CUI would parse as IDs. */
    const char *back_id = *back ? "Back###fpas-tour-back" : "###fpas-tour-back";
    const char *forward_id = *forward ? "Next###fpas-tour-forward" : "###fpas-tour-forward";
    const cui_tour_opts options = { .width = width, .padding = padding, .block_target = block_target != 0,
        .next_label = forward_id, .back_label = back_id, .done_label = forward_id, .count_format = count_format };
    cui_push_id(ui, id);
    const cui_id back_key = cui_make_id(ui, back_id), forward_key = cui_make_id(ui, forward_id);
    const cui_id close_key = cui__hash_str(CUI_ICON_X, cui_make_id(ui, "End the tour"));
    cui_pop_id(ui);

    /* CUI-119 (docs/future/cui-upstream-bugs.md#cui-119): route Tab to the tour,
       and keep the active target's arrow keys available when it holds focus. */
    const cui_id focus_key = *key ^ 0x7A0Fu;
    fpas_tour_focus *state = cui_state(ui, focus_key, sizeof *state);
    if (!state->active) {
        *state = (fpas_tour_focus){ .previous = ui->focus, .close = close_key, .back = back_key,
            .forward = forward_key, .active = true, .visible = ui->focus_visible };
        ui->focus_next_request = forward_key;
    }
    state->target = block_target ? 0 : targets[*step];
    if (focus && (focus != 2 || *step > 0)) {
        ui->focus_next_request = focus == 1 ? close_key : focus == 2 ? back_key : forward_key;
        ui->focus_visible = true;
    } else if (!ui->focus ||
        (ui->focus != state->target && ui->focus != close_key && ui->focus != forward_key &&
         (*step == 0 || ui->focus != back_key))) {
        ui->focus_next_request = forward_key;
    }
    const bool target_focused = state->target && ui->focus == state->target;
    const uint8_t left = ui->kpressed[CUI_KEY_LEFT], right = ui->kpressed[CUI_KEY_RIGHT];
    if (target_focused) ui->kpressed[CUI_KEY_LEFT] = ui->kpressed[CUI_KEY_RIGHT] = 0;
    cui_push_focus_scope(ui, *key, true);
    const int event = cui_tour_ex(ui, id, steps, count, step, &options);
    cui_pop_focus_scope(ui);
    if (target_focused) {
        ui->kpressed[CUI_KEY_LEFT] = left;
        ui->kpressed[CUI_KEY_RIGHT] = right;
    }
    if (event == CUI_TOUR_DONE || event == CUI_TOUR_SKIPPED) fpas_tour_restore_focus(ui, state);
    fpas_own(ui, *key, focus_key, 0, 0);
    fpas_on_forget(ui, *key, fpas_tour_forget);
    *node = fpas_find(ui, *key ^ 0xCA2Du);
    /* CUI-120 (docs/future/cui-upstream-bugs.md#cui-120): stop wheel routing
       at the card and scrim bands. The spotlight hole still has no hit node. */
    if (*node >= 0) ui->nodes[*node].flags |= CUI_NF_BLOCKER;
    const int scrim = fpas_find(ui, *key);
    if (scrim >= 0)
        for (int band = ui->nodes[scrim].abs_first; band >= 0; band = ui->nodes[band].next)
            ui->nodes[band].flags |= CUI_NF_BLOCKER;
    parts[0] = fpas_find(ui, close_key);
    parts[1] = fpas_find(ui, back_key);
    parts[2] = fpas_find(ui, forward_key);
    if (parts[1] >= 0 && *back) fpas_text(ui, fpas_label_node(ui, parts[1]), back);
    if (parts[2] >= 0 && *forward) fpas_text(ui, fpas_label_node(ui, parts[2]), forward);
    ui->last_node = saved_node; ui->last_id = saved_id;
    return event;
}
