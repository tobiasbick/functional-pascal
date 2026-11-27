// [Documentation](../../../../../docs/ui/gui/canvas-and-bars/canvas-controls.md)
#include "../../fpas_cui.h"

/** Opens the rail with the native geometry and appearance. */
static void fpas_canvas_rail_begin(cui_ctx *ui, int anchor, cui_id key, cui__rail_state *state,
    int buttons, bool separator, bool start_side) {
    const float height = ui->theme.control_md;
    const float rail_w = height + 2 * CUI__RAIL_PAD + 2;
    const float rail_h = buttons * height + CUI__RAIL_READOUT_H + (separator ? CUI__RAIL_SEP_H : 0)
        + (buttons + separator) * CUI__RAIL_GAP + 2 * CUI__RAIL_PAD + 2;
    const float x = state->rtl != start_side ? CUI__RAIL_INSET : state->w - CUI__RAIL_INSET - rail_w;
    const float y = roundf((state->h - rail_h) * .5f);
    const int rail = cui__absolute(ui, key, CUI_MAX(0, x), CUI_MAX(0, y), rail_w, rail_h);
    cui_node *node = &ui->nodes[rail];
    node->abs_anchor = anchor;
    node->flags |= CUI_NF_OVERLAY | CUI_NF_HIT_OPAQUE;
    node->dir = CUI_COLUMN;
    node->align = CUI_ALIGN_CENTER;
    node->gap = CUI__RAIL_GAP;
    node->pad[0] = node->pad[1] = node->pad[2] = node->pad[3] = CUI__RAIL_PAD;
    node->radius = CUI_CORNERS(ui->theme.radius_panel);
    node->bg = CUI_C(ui, SURFACE);
    node->border = 1;
    node->border_color = CUI_C(ui, BORDER);
    node->elevation = CUI_ELEVATION_RAISED;
    node->elevation_alpha = 1;
    node->paint = cui__paint_rail_measure;
    node->paint_data = state;
    cui__open(ui, rail);
}

/** Draws the controls and applies user-requested zoom, fit and toggle changes. */
int fpas_cui_canvas_controls(cui_ctx *ui, const char *id, cui_node_view *view, cui_rect content,
    const float *steps, int step_count, const fpas_canvas_toggle *toggles, bool *values, int count, int flags) {
    const int anchor = ui->last_node;
    if (!view || anchor < 0 || anchor >= ui->node_count) return 0;
    const cui_id anchor_key = ui->last_id;
    const cui_id key = cui_make_id(ui, id);
    cui__rail_state *state = cui_state(ui, key, sizeof *state);
    if (!state->measured && ui->nodes[anchor].id && cui__has_prev_rect(ui, ui->nodes[anchor].id)) {
        const cui_rect previous = cui__prev_rect(ui, ui->nodes[anchor].id);
        state->w = previous.w;
        state->h = previous.h;
    }
    if (!steps || step_count <= 0) {
        steps = cui__zoom_steps;
        step_count = CUI_COUNTOF(cui__zoom_steps);
    }
    const float minimum = steps[0], maximum = steps[step_count - 1];
    const float zoom = view->zoom > 0 ? view->zoom : 1;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const bool fit = !(flags & 2);
    if (!state->init || !enabled || zoom != state->written) {
        state->target = state->cur = zoom;
        state->anim = state->fit = false;
        state->init = true;
    }

    // CUI-107 (docs/future/cui-upstream-bugs.md#cui-107): leave keyboard activation to the popup.
    const bool covered = enabled && cui__keys_covered(ui);
    const uint8_t enter = ui->kpressed[CUI_KEY_ENTER], space = ui->kpressed[CUI_KEY_SPACE];
    if (covered) ui->kpressed[CUI_KEY_ENTER] = ui->kpressed[CUI_KEY_SPACE] = 0;
    cui_push_disabled(ui, !enabled);
    cui_push_id(ui, id);
    const bool separator = fit || count > 0;
    fpas_canvas_rail_begin(ui, anchor, key, state, 2 + fit + count, separator, (flags & 4) != 0);

    // CUI-166 (docs/future/cui-upstream-bugs.md#cui-166): step through every distinct preset.
    float target = state->target;
    cui_button_opts button = { .variant = CUI_BUTTON_GHOST, .tooltip = "Zoom in", .disabled = target >= maximum };
    if (cui_icon_button_ex(ui, CUI_ICON_PLUS, &button)) {
        for (int i = 0; i < step_count; ++i) {
            if (steps[i] > target) { target = steps[i]; break; }
        }
    }
    // CUI-165 (docs/future/cui-upstream-bugs.md#cui-165): a preset range only limits steps and fit.
    if (cui__rail_readout(ui, cui_make_id(ui, "#zoom"), state, zoom)) target = 1;
    button.tooltip = "Zoom out";
    button.disabled = target <= minimum;
    if (cui_icon_button_ex(ui, CUI_ICON_MINUS, &button)) {
        for (int i = step_count - 1; i >= 0; --i) {
            if (steps[i] < target) { target = steps[i]; break; }
        }
    }

    if (separator) {
        const int divider = cui__node_new(ui, 0);
        ui->nodes[divider].pad[0] = ui->nodes[divider].pad[2] = 3;
        cui__open(ui, divider);
        const int line = cui__leaf(ui, 0, CUI_PX(20), CUI_PX(1));
        ui->nodes[line].bg = CUI_C(ui, BORDER);
        cui__close(ui);
    }
    int result = 0;
    button.disabled = false;
    if (fit) {
        button.tooltip = "Fit";
        if (cui_icon_button_ex(ui, "", &button)) {
            result |= CUI_CANVAS_FIT;
            if (content.w > 0 && content.h > 0 && state->w > 0 && state->h > 0) {
                const float scale = CUI_MIN(state->w * .9f / content.w, state->h * .9f / content.h);
                target = cui__clampf(scale, minimum, maximum);
                state->fit = state->anim = true;
                state->tpx = content.x + content.w * .5f;
                state->tpy = content.y + content.h * .5f;
                state->px = (state->w * .5f - view->x) / zoom;
                state->py = (state->h * .5f - view->y) / zoom;
                result |= CUI_CANVAS_ZOOMED;
            }
        }
        cui__cv_glyph_on(ui, ui->last_node, CUI__CV_FIT, 16, CUI_C(ui, FOREGROUND));
    }
    for (int i = 0; i < count; ++i) {
        button.tooltip = toggles[i].label;
        button.selected = values[i];
        cui_push_id_int(ui, i);
        // CUI-031 (docs/future/cui-upstream-bugs.md#cui-031): keep focus when the toggle changes its glyph or tooltip.
        const char *icon = toggles[i].icon ? toggles[i].icon : CUI_ICON_CIRCLE;
        if (cui__icon_button_id(ui, cui_make_id(ui, "#toggle"), icon, &button)) {
            values[i] = !values[i];
            result |= CUI_CANVAS_TOGGLED;
        }
        cui_pop_id(ui);
    }
    cui__close(ui);
    cui_pop_id(ui);
    cui_pop_disabled(ui);
    if (covered) {
        ui->kpressed[CUI_KEY_ENTER] = enter;
        ui->kpressed[CUI_KEY_SPACE] = space;
    }

    if (target != state->target) {
        state->target = target;
        state->anim = true;
        result |= CUI_CANVAS_ZOOMED;
    }
    if (state->anim && state->w > 0) cui__rail_ease(ui, state, view);
    else if (state->anim) {
        view->zoom = state->cur = state->target;
        state->anim = false;
    }
    state->written = view->zoom > 0 ? view->zoom : 1;
    ui->last_node = anchor;
    ui->last_id = anchor_key;
    return result;
}
