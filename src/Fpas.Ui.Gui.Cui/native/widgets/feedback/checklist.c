#include "../../fpas_cui.h"

typedef struct { cui_id scope, key, ring; int count; } fpas_checklist_state;

static cui_id fpas_checklist_step(cui_id key, int index) {
    return cui__hash(&index, sizeof index, key);
}

static cui_id fpas_checklist_action(cui_id scope, int index) {
    return cui__hash_str("###action", cui__hash(&index, sizeof index, scope));
}

static void fpas_checklist_forget_steps(cui_ctx *ui, const fpas_checklist_state *state, int first) {
    for (int index = first; index < state->count; ++index) {
        const cui_id step = fpas_checklist_step(state->key, index);
        fpas_forget_state(ui, step);
        fpas_forget_state(ui, step ^ 0xC1u);
        fpas_forget_state(ui, fpas_checklist_action(state->scope, index));
    }
}

static void fpas_checklist_retire(cui_ctx *ui, cui_id key) {
    const fpas_checklist_state *state = cui__state_lazy(ui, key, sizeof *state, false);
    if (!state) return;
    fpas_checklist_forget_steps(ui, state, 0);
    fpas_forget_state(ui, state->ring);
}

static void fpas_paint_focused_checklist_circle(cui_ctx *ui, cui_node *node, void *data) {
    cui__paint_step_circle(ui, node, data);
    cui__draw_focus_ring(ui, node->rect, node->radius, 1.0f);
}

static void fpas_checklist_circle(cui_ctx *ui, int node, cui_id key) {
    cui_node *circle = &ui->nodes[node];
    if (circle->paint != cui__paint_focus_after) return;
    /* CUI-010 (docs/future/cui-upstream-bugs.md#cui-010): keep the circle
       painter when the generic keyboard focus ring replaces it. */
    const cui__step_state *state = cui_state(ui, key, sizeof *state);
    cui__circle_paint *paint = cui__frame_alloc(ui, sizeof *paint);
    *paint = (cui__circle_paint){
        .done = state->done, .hover = state->hover, .on = CUI_C(ui, SUCCESS), .ring = CUI_C(ui, BORDER_STRONG),
        .glyph = ui->mode == CUI_THEME_DARK ? cui__rgba(8, 26, 16, 255) : cui__rgba(255, 255, 255, 255)
    };
    circle->paint = fpas_paint_focused_checklist_circle;
    circle->paint_data = paint;
}

int fpas_cui_checklist(cui_ctx *ui, const char *id, const char *title, const char *description,
    const char *const *titles, const char *const *descriptions, const char *const *actions,
    int *completed, int count, int enabled, int focus, int *node, int *parts) {
    /* Native action buttons are keyed by index and caption within the caller's scope. */
    cui_push_id(ui, id);
    const cui_id scope = cui__seed(ui), key = cui_make_id(ui, id);
    fpas_checklist_state *owner = cui_state(ui, key, sizeof *owner);
    fpas_checklist_forget_steps(ui, owner, count);
    *owner = (fpas_checklist_state){
        scope, key, cui_make_id(ui, cui__frame_printf(ui, "##%s-ring", id)) ^ 0xA11A11u, count
    };
    fpas_on_forget(ui, key, fpas_checklist_retire);
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus > 0) {
        int slot = focus - 1, index = slot / 2;
        ui->focus_next_request = slot % 2 ? fpas_checklist_action(scope, index)
            : fpas_checklist_step(key, index) ^ 0xC1u;
    }

    cui_check_step *steps = cui__frame_alloc(ui, (size_t)count * sizeof *steps);
    for (int index = 0; index < count; ++index) {
        steps[index] = (cui_check_step){
            .title = cui__frame_strdup(ui, titles[index], -1),
            .description = *descriptions[index] ? cui__frame_strdup(ui, descriptions[index], -1) : NULL,
            .action = *actions[index] ? (cui__display_len(actions[index]) ?
                cui__frame_printf(ui, "%.*s###action", cui__display_len(actions[index]), actions[index])
                : "x###action") : NULL,
            .done = completed[index] != 0
        };
    }
    const cui_checklist_opts options = {
        .title = title ? cui__frame_strdup(ui, title, -1) : NULL,
        .description = description ? cui__frame_strdup(ui, description, -1) : NULL
    };
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int action = cui_checklist_ex(ui, id, steps, count, &options);
    cui_pop_disabled(ui);
    for (int index = 0; index < count; ++index) {
        const cui_id step = fpas_checklist_step(key, index);
        const int circle = fpas_find(ui, step ^ 0xC1u);
        const int button = *actions[index] ? fpas_find(ui, fpas_checklist_action(scope, index)) : -1;
        fpas_checklist_circle(ui, circle, step);
        if (button >= 0 && strstr(actions[index], "##")) fpas_text(ui, fpas_label_node(ui, button), actions[index]);
        if (parts) { parts[index * 2] = circle; parts[index * 2 + 1] = button; }
        if (allowed) completed[index] = steps[index].done;
    }
    cui_pop_id(ui);
    ui->last_node = *node;
    return allowed ? action : -1;
}
