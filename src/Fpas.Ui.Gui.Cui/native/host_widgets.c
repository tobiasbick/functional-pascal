#include "fpas_cui.h"

int fpas_cui_group_begin(cui_ctx *ui, int horizontal, float padding, float gap, int align, int framed, int wrap, int clip) {
    return fpas_cui_group_begin_ex(ui, horizontal, padding, gap, align, framed, wrap, clip, CUI_JUSTIFY_START);
}

int fpas_cui_group_begin_ex(cui_ctx *ui, int horizontal, float padding, float gap,
                            int align, int framed, int wrap, int clip, int justify) {
    const cui_box options = { .dir = (cui_dir)horizontal,
        .pad = padding, .gap = gap, .align = (cui_align)align, .justify = (cui_justify)justify,
        .border = framed ? 1 : 0, .clip = clip != 0, .wrap = wrap != 0 };
    cui_box_begin(ui, &options);
    return cui__current(ui);
}

void fpas_cui_group_end(cui_ctx *ui, int node) {
    cui_box_end(ui);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}

void fpas_cui_box_details(cui_ctx *ui, int node, float top, float end, float bottom, float start, int direction) {
    cui_node *n = &ui->nodes[node];
    n->pad[0] = top; n->pad[1] = end; n->pad[2] = bottom; n->pad[3] = start;
    n->direction = (uint8_t)direction;
}

void fpas_cui_node_size(cui_ctx *ui, int node, int horizontal, int kind, float basis,
                        float minimum, float maximum, float grow, int align) {
    if (node < 0 || node >= ui->node_count) return;
    cui_node *n = &ui->nodes[node];
    if (maximum < 0) maximum = 1e20f;
    /* Preserve intrinsic fixed leaf sizes for content; all other sizes use CUI directly. */
    if (horizontal) {
        if (kind == 2 || kind == 3) n->w = fpas_extent(kind, basis);
        else if (kind == 1 || grow > 0) n->w = CUI_GROW(grow > 0 ? grow : 1);
        n->min_w = minimum; n->max_w = maximum;
    } else {
        if (kind == 2 || kind == 3) n->h = fpas_extent(kind, basis);
        else if (kind == 1 || grow > 0) n->h = CUI_GROW(grow > 0 ? grow : 1);
        n->min_h = minimum; n->max_h = maximum;
    }
    if (align != CUI_ALIGN_DEFAULT) n->self_align = align;
    if (align == CUI_ALIGN_STRETCH) {
        if (horizontal && n->h.kind != CUI_SIZE_FIXED && n->h.kind != CUI_SIZE_PERCENT) n->h = CUI_FILL;
        if (!horizontal && n->w.kind != CUI_SIZE_FIXED && n->w.kind != CUI_SIZE_PERCENT) n->w = CUI_FILL;
    }
}

void fpas_cui_node_role(cui_ctx *ui, int node, int role) {
    if (node < 0 || node >= ui->node_count) return;
    if (role == 0) return;
    cui_node *n = &ui->nodes[node];
    if (role == 1 || role == 7 || role == 9 || role == 10) {
        n->bg = role == 7 ? CUI_C(ui, SURFACE_MUTED) : role == 9 ? CUI_C(ui, BORDER) : CUI_C(ui, SURFACE);
        if (role == 10) {
            n->radius = CUI_CORNERS(ui->theme.radius_surface);
            n->border = 1;
            n->border_color = CUI_C(ui, BORDER);
        }
        return;
    }
    cui_color color = role == 3 || role == 12 || role == 15 ? CUI_C(ui, TEXT_MUTED) : role == 6 ? CUI_C(ui, DANGER) :
        role == 4 ? CUI_C(ui, ACCENT) : role == 5 || role == 18 ? CUI_C(ui, TEXT_SECONDARY) : CUI_C(ui, FOREGROUND);
    int label = (ui->nodes[node].flags & CUI_NF_TEXT) ? node : fpas_label_node(ui, node);
    if (label >= 0) {
        ui->nodes[label].text_color = color;
        if (role == 2 || role == 8 || role == 16 || role == 17) {
            const float size = ui->theme.heading[role == 17 ? 0 : role == 8 ? 1 : role == 16 ? 2 : 3];
            ui->nodes[label].font_size = cui__fs(ui, size);
            ui->nodes[label].font = size >= 26 ? CUI_FONT_DISPLAY : CUI_FONT_MEDIUM;
            ui->nodes[label].line_height = size >= 26 ? 1.15f : 1.3f;
        } else if (role == 11) {
            ui->nodes[label].font = CUI_FONT_MEDIUM;
            ui->nodes[label].font_size = cui__fs(ui, ui->theme.text_body);
            ui->nodes[label].line_height = 1.35f;
        } else if (role == 12) {
            ui->nodes[label].font_size = cui__fs(ui, ui->theme.text_sm);
        } else if (role == 13) {
            ui->nodes[label].font = CUI_FONT_DISPLAY;
            ui->nodes[label].font_size = cui__fs(ui, 22);
            ui->nodes[label].flags |= CUI_NF_TABULAR;
        } else if (role == 14) {
            ui->nodes[label].font = CUI_FONT_MEDIUM;
        } else if (role == 18) {
            ui->nodes[label].font_size = cui__fs(ui, 15);
        } else if (role == 15) {
            ui->nodes[label].font = CUI_FONT_ICONS;
            ui->nodes[label].font_size = 12;
            ui->nodes[label].line_height = 1;
            ui->nodes[label].flags &= ~CUI_NF_WRAP;
            ui->nodes[label].self_align = CUI_ALIGN_CENTER;
        }
    }
    for (int child = n->first; child >= 0; child = ui->nodes[child].next)
        fpas_cui_node_role(ui, child, role);
}

int fpas_cui_focused(cui_ctx *ui, int node) { return node >= 0 && node < ui->node_count && fpas_has_focus(ui, ui->nodes[node].id); }
int fpas_cui_hovered(cui_ctx *ui, int node) { return node >= 0 && node < ui->node_count && ui->nodes[node].id == ui->hot; }

unsigned int fpas_cui_node_key(cui_ctx *ui, int node) { return ui->nodes[node].id; }

void fpas_cui_scroll_state(cui_ctx *ui, const char *id, int mode, float offset, float *current, float *maximum) {
    cui_scroll_state *state = cui_state(ui, cui_make_id(ui, id), sizeof *state);
    *maximum = CUI_MAX(0, state->content_h - state->view_h);
    *current = state->ty;
    if (mode >= 0) {
        state->ty = mode == 1 ? *maximum : cui__clampf(offset, 0, *maximum);
        state->y = state->ty;
    }
}

unsigned int fpas_cui_modal_key(cui_ctx *ui) { return ui->current_layer; }

int fpas_cui_focus_key(cui_ctx *ui, unsigned int key) { return fpas_has_focus(ui, key); }

void fpas_cui_host_begin(cui_ctx *ui) {
    ui->prev_modal_layer = 0;
}

static void fpas_hidden_scroll(cui_ctx *ui, cui_node *node, void *data) {
    cui_scroll_state *state = cui_state(ui, node->id, sizeof *state);
    state->content_w = node->content_w; state->content_h = node->content_h;
    state->view_w = node->rect.w; state->view_h = node->rect.h;
    cui_rect clip = ui->dl.clip_stack[ui->dl.clip_depth - 1];
    float scale = ui->scale;
    clip.x /= scale; clip.y /= scale; clip.w /= scale; clip.h /= scale;
    if (state->fade.a) {
        const float bar = state->bar_t;
        const int hit_count = ui->hit_count;
        state->bar_t = 0;
        cui__paint_scrollbars(ui, node, clip);
        state->bar_t = bar;
        ui->hit_count = hit_count;
    }
    /* Keep native scroll flags through measurement and placement. Only hide
       the automatic scrollbar paint after children and native edge fades. */
    node->flags &= ~(CUI_NF_SCROLL_X | CUI_NF_SCROLL_Y);
    cui__paint_add_hit(ui, node->id, cui__rect_intersect(node->rect, clip),
        CUI_NF_SCROLL_Y | (data ? CUI_NF_SCROLL_X : 0));
}

int fpas_cui_viewport(cui_ctx *ui, const char *id, int scrollbar, int enabled) {
    return fpas_cui_viewport_ex(ui, id, scrollbar, enabled, 0, 0);
}

int fpas_cui_viewport_ex(cui_ctx *ui, const char *id, int scrollbar, int enabled, int fade_edges, int horizontal) {
    const cui_id key = cui_make_id(ui, id);
    cui_scroll_state *state = cui_state(ui, key, sizeof *state);
    state->fade = fade_edges ? cui__backdrop(ui) : cui__rgba(0, 0, 0, 0);
    const cui_box options = { .width = CUI_FILL, .height = CUI_FILL };
    float wheel_x = ui->scroll_dx, wheel_y = ui->scroll_dy;
    /* CUI-007 (docs/future/cui-upstream-bugs.md#cui-007):
       A vertical viewport must leave horizontal wheel input for swipe rows. */
    if (!horizontal || !enabled) ui->scroll_dx = 0;
    if (!enabled) ui->scroll_dy = 0;
    cui_push_disabled(ui, !enabled);
    int node = cui__scroll_begin_ex(ui, key, &options, horizontal != 0);
    cui_pop_disabled(ui);
    ui->scroll_dx = wheel_x;
    if (!enabled) ui->scroll_dy = wheel_y;
    if (!scrollbar) {
        ui->nodes[node].flags |= CUI_NF_CLIP | CUI_NF_PAINT_AFTER;
        ui->nodes[node].paint = fpas_hidden_scroll;
        ui->nodes[node].paint_data = horizontal ? ui : NULL;
    }
    return node;
}

void fpas_cui_scroll_layout(cui_ctx *ui, int node, int mode, float offset) {
    cui_node *n = &ui->nodes[node];
    cui_scroll_state *state = cui_state(ui, n->id, sizeof *state);
    state->content_h = n->content_h; state->view_h = n->rect.h;
    float maximum = CUI_MAX(0, n->content_h - n->rect.h);
    state->y = state->ty = mode == 1 ? maximum : cui__clampf(offset, 0, maximum);
    n->scroll_y = state->y;
}

void fpas_cui_scroll_metrics(cui_ctx *ui, int node, float *offset, float *maximum) {
    cui_scroll_state *state = cui_state(ui, ui->nodes[node].id, sizeof *state);
    *offset = state->ty;
    *maximum = CUI_MAX(0, state->content_h - state->view_h);
}
