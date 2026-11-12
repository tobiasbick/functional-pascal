#include "fpas_cui.h"

int fpas_cui_group_begin(cui_ctx *ui, int horizontal, float padding, float gap, int align, int framed, int wrap) {
    const cui_box options = { .dir = horizontal ? CUI_ROW : CUI_COLUMN,
        .pad = padding, .gap = gap, .align = (cui_align)align,
        .border = framed ? 1 : 0, .clip = !wrap, .wrap = wrap != 0 };
    cui_box_begin(ui, &options);
    return cui__current(ui);
}

void fpas_cui_group_end(cui_ctx *ui, int node) {
    cui_box_end(ui);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}

void fpas_cui_node_size(cui_ctx *ui, int node, int horizontal, int kind, float basis,
                        float minimum, float maximum, float grow, int align) {
    if (node < 0 || node >= ui->node_count) return;
    cui_node *n = &ui->nodes[node];
    if (maximum < 0) maximum = 1e20f;
    /* Preserve intrinsic fixed leaf sizes for content; all other sizes use CUI directly. */
    if (horizontal) {
        if (kind == 2) n->w = CUI_PX(basis);
        else if (kind == 1 || grow > 0) n->w = CUI_GROW(grow > 0 ? grow : 1);
        n->min_w = minimum; n->max_w = maximum;
    } else {
        if (kind == 2) n->h = CUI_PX(basis);
        else if (kind == 1 || grow > 0) n->h = CUI_GROW(grow > 0 ? grow : 1);
        n->min_h = minimum; n->max_h = maximum;
    }
    if (align != CUI_ALIGN_DEFAULT) n->self_align = align;
    if (align == CUI_ALIGN_STRETCH) {
        if (horizontal && n->h.kind != CUI_SIZE_FIXED) n->h = CUI_FILL;
        if (!horizontal && n->w.kind != CUI_SIZE_FIXED) n->w = CUI_FILL;
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
    cui_color color = role == 3 ? CUI_C(ui, TEXT_MUTED) : role == 6 ? CUI_C(ui, DANGER) :
        role == 4 ? CUI_C(ui, ACCENT) : role == 5 ? CUI_C(ui, TEXT_SECONDARY) : CUI_C(ui, FOREGROUND);
    int label = (ui->nodes[node].flags & CUI_NF_TEXT) ? node : fpas_label_node(ui, node);
    if (label >= 0) {
        ui->nodes[label].text_color = color;
        if (role == 2 || role == 8) {
            const float size = ui->theme.heading[role == 8 ? 1 : 3];
            ui->nodes[label].font_size = cui__fs(ui, size);
            ui->nodes[label].font = size >= 26 ? CUI_FONT_DISPLAY : CUI_FONT_MEDIUM;
            ui->nodes[label].line_height = size >= 26 ? 1.15f : 1.3f;
        } else if (role == 11) {
            ui->nodes[label].font = CUI_FONT_MEDIUM;
            ui->nodes[label].font_size = cui__fs(ui, ui->theme.text_body);
            ui->nodes[label].line_height = 1.35f;
        }
    }
    for (int child = n->first; child >= 0; child = ui->nodes[child].next)
        fpas_cui_node_role(ui, child, role);
}

int fpas_cui_focused(cui_ctx *ui, int node) { return node >= 0 && node < ui->node_count && ui->nodes[node].id == ui->focus; }
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

int fpas_cui_focus_key(cui_ctx *ui, unsigned int key) { return key && ui->focus == key; }

void fpas_cui_host_begin(cui_ctx *ui) {
    ui->prev_modal_layer = 0;
    fpas_route_text_area_wheel(ui);
}

static void fpas_hidden_scroll(cui_ctx *ui, cui_node *node, void *data) {
    (void)data;
    cui_scroll_state *state = cui_state(ui, node->id, sizeof *state);
    state->content_w = node->content_w; state->content_h = node->content_h;
    state->view_w = node->rect.w; state->view_h = node->rect.h;
    cui_rect clip = ui->dl.clip_stack[ui->dl.clip_depth - 1];
    float scale = ui->scale;
    clip.x /= scale; clip.y /= scale; clip.w /= scale; clip.h /= scale;
    cui__paint_add_hit(ui, node->id, cui__rect_intersect(node->rect, clip), CUI_NF_SCROLL_Y);
}

int fpas_cui_viewport(cui_ctx *ui, const char *id, int scrollbar, int enabled) {
    const cui_box options = { .width = CUI_FILL, .height = CUI_FILL };
    float wheel_x = ui->scroll_dx, wheel_y = ui->scroll_dy;
    if (!enabled) ui->scroll_dx = ui->scroll_dy = 0;
    cui_push_disabled(ui, !enabled);
    int node = cui__scroll_begin_ex(ui, cui_make_id(ui, id), &options, false);
    cui_pop_disabled(ui);
    if (!enabled) { ui->scroll_dx = wheel_x; ui->scroll_dy = wheel_y; }
    if (!scrollbar) {
        ui->nodes[node].flags &= ~CUI_NF_SCROLL_Y;
        ui->nodes[node].flags |= CUI_NF_CLIP;
        ui->nodes[node].paint = fpas_hidden_scroll;
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
