#include "../fpas_cui.h"

static const uint32_t fpas_text_area_scroll_hit = 1u << 30;

static void fpas_paint_text_area(cui_ctx *ui, cui_node *area, void *data) {
    cui__edit_paint *paint = data;
    cui__paint_edit(ui, area, data);

    /* CUI-001 (docs/future/cui-upstream-bugs.md#cui-001):
     * Let the editor receive the wheel before an enclosing viewport. */
    cui_scroll_state *scroll = cui_state(ui, area->id, sizeof *scroll);
    scroll->view_h = area->rect.h;
    scroll->content_h = paint->line_count * paint->line_h;
    scroll->ty = paint->scroll_y;
    if (scroll->content_h <= scroll->view_h + 0.5f) return;
    cui_rect clip = ui->dl.clip_stack[ui->dl.clip_depth - 1];
    const float scale = ui->scale;
    clip = (cui_rect){clip.x / scale, clip.y / scale, clip.w / scale, clip.h / scale};
    const cui_rect box = cui__xform_rect(ui, ui->nodes[area->parent].rect);
    cui__paint_add_hit(ui, area->id, cui__rect_intersect(box, clip), CUI_NF_SCROLL_Y | fpas_text_area_scroll_hit);
}

static void fpas_route_text_area_wheel(cui_ctx *ui) {
    if (ui->scroll_dy == 0 || ui->scroll_dx != 0 || (ui->mods & CUI_MOD_SHIFT)) return;

    /* CUI checks the overflow axis. Skip editors that have reached the edge in this direction. */
    bool changed = false;
    for (int i = 0; i < ui->prev_hit_count; ++i) {
        cui_hit *hit = &ui->prev_hits[i];
        if (!(hit->flags & fpas_text_area_scroll_hit) || !cui__rect_contains(hit->rect, ui->mx, ui->my)) continue;
        const cui_scroll_state *scroll = cui_state(ui, hit->id, sizeof *scroll);
        const float remaining = ui->scroll_dy > 0 ? scroll->ty : scroll->content_h - scroll->view_h - scroll->ty;
        if (remaining > 0.5f) continue;
        hit->flags &= ~CUI_NF_SCROLL_Y;
        changed = true;
    }
    if (!changed) return;

    cui__compute_hot(ui);
    for (int i = 0; i < ui->prev_hit_count; ++i) {
        cui_hit *hit = &ui->prev_hits[i];
        if (hit->flags & fpas_text_area_scroll_hit) hit->flags |= CUI_NF_SCROLL_Y;
    }
}

int fpas_cui_text_area(cui_ctx *ui, const char *id, char *value, size_t capacity,
    const char *label, const char *placeholder, const char *icon, const char *hint, const char *error,
    const char *prefix, const char *suffix, int size, float height, int clearable,
    int read_only, int enabled, int focus, int *nodes) {
    const cui_input_opts options = {
        .label = label, .placeholder = placeholder, .icon = icon, .hint = hint, .error = error,
        .prefix = prefix, .suffix = suffix, .size = (cui_control_size)size,
        .height = height, .clearable = clearable != 0, .read_only = read_only != 0
    };
    int revealed;
    return fpas_draw_text_editor(ui, id, value, capacity, options, true, enabled, focus, nodes, &revealed, cui_textarea_ex);
}
