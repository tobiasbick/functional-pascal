#include "fpas_cui.h"

static int fpas_find(cui_ctx *ui, cui_id id) {
    for (int i = ui->node_count - 1; i >= 0; --i) if (ui->nodes[i].id == id) return i;
    return -1;
}

static int fpas_label_node(cui_ctx *ui, int node) {
    for (int child = ui->nodes[node].first; child >= 0; child = ui->nodes[child].next) {
        if ((ui->nodes[child].flags & CUI_NF_TEXT) && ui->nodes[child].font != CUI_FONT_ICONS) return child;
        int found = fpas_label_node(ui, child);
        if (found >= 0) return found;
    }
    return -1;
}

/* Retain the upstream widget's interaction/paint but supply literal text separately from identity. */
static void fpas_literal(cui_ctx *ui, int node, const char *text, cui_color color) {
    int label = fpas_label_node(ui, node);
    if (label < 0) {
        cui__open(ui, node);
        label = cui__text_node(ui, text, -1, CUI_FONT_MEDIUM, cui__fs(ui, ui->theme.text_body), color, false);
        cui__close(ui);
    } else {
        ui->nodes[label].text = cui__frame_strdup(ui, text, (int)strlen(text));
        ui->nodes[label].text_len = (int)strlen(text);
    }
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}

int fpas_cui_control(cui_ctx *ui, const char *id, const char *text, int kind,
                    int enabled, int selected, int variant, int focus) {
    cui_id key = cui_make_id(ui, id);
    if (kind == 1) key ^= 0xC0EC0001u;
    if (kind == 2) key ^= 0x115E0008u;
    if (!enabled && ui->focus == key) ui->focus = 0;
    if (focus && enabled) ui->focus_next_request = key;
    cui_push_disabled(ui, !enabled);
    bool activated = false, value = selected != 0;
    cui_color color = CUI_C(ui, FOREGROUND);
    if (kind == 1) {
        activated = cui_checkbox_ex(ui, id, &value, NULL);
    } else if (kind == 2) {
        const cui_list_item_opts options = { .selected = value };
        activated = cui_list_item_ex(ui, id, &options);
    } else {
        cui_button_variant native_variant = variant == 1 ? CUI_BUTTON_PRIMARY :
            variant == 4 ? CUI_BUTTON_DANGER : variant == 2 ? CUI_BUTTON_SECONDARY : CUI_BUTTON_DEFAULT;
        const cui_button_opts options = { .variant = native_variant, .selected = value || variant == 3 };
        color = cui__button_colors(ui, native_variant == CUI_BUTTON_DEFAULT ? CUI_BUTTON_SECONDARY : native_variant, options.selected).fg;
        activated = cui_button_ex(ui, id, &options);
    }
    cui_pop_disabled(ui);
    int node = fpas_find(ui, key);
    if (node >= 0) fpas_literal(ui, node, text, color);
    return (activated ? 1 : 0) | (value ? 2 : 0);
}

int fpas_cui_group_begin(cui_ctx *ui, int horizontal, float padding, float gap, int align, int framed) {
    const cui_box options = { .dir = horizontal ? CUI_ROW : CUI_COLUMN,
        .pad = padding, .gap = gap, .align = (cui_align)align,
        .border = framed ? 1 : 0, .clip = true };
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

void fpas_cui_preferred_width(cui_ctx *ui, int node, float width) { ui->nodes[node].pref_hint = width; }

void fpas_cui_node_role(cui_ctx *ui, int node, int role) {
    if (role == 0) return;
    cui_color color = role == 3 ? CUI_C(ui, TEXT_MUTED) : role == 6 ? CUI_C(ui, DANGER) :
        role == 4 ? CUI_C(ui, ACCENT) : role == 5 ? CUI_C(ui, TEXT_SECONDARY) : CUI_C(ui, FOREGROUND);
    int label = (ui->nodes[node].flags & CUI_NF_TEXT) ? node : fpas_label_node(ui, node);
    if (label >= 0) {
        ui->nodes[label].text_color = color;
        if (role == 2) ui->nodes[label].font_size = cui__fs(ui, ui->theme.text_lg);
    }
    if (role == 1) ui->nodes[node].bg = CUI_C(ui, SURFACE);
    for (int child = ui->nodes[node].first; role != 1 && child >= 0; child = ui->nodes[child].next)
        fpas_cui_node_role(ui, child, role);
}

int fpas_cui_focused(cui_ctx *ui, int node) { return node >= 0 && node < ui->node_count && ui->nodes[node].id == ui->focus; }
int fpas_cui_hovered(cui_ctx *ui, int node) { return node >= 0 && node < ui->node_count && ui->nodes[node].id == ui->hot; }

unsigned int fpas_cui_node_key(cui_ctx *ui, int node) { return ui->nodes[node].id; }

void fpas_cui_forget(cui_ctx *ui, unsigned int key) {
    if (!key) return;
    int bucket = (int)(key & (uint32_t)(ui->bucket_count - 1));
    int *link = &ui->state_buckets[bucket];
    while (*link >= 0) {
        int index = *link;
        cui_state_entry *entry = &ui->states[index];
        if (entry->id != key) { link = &entry->next; continue; }
        *link = entry->next;
        if (ui->edit == entry->data) { ui->edit = NULL; ui->edit_prev = false; }
        if (ui->focus == key) ui->focus = 0;
        if (ui->active == key) ui->active = 0;
        if (entry->dtor && entry->data) entry->dtor(ui, entry->data);
        cui__free(ui, entry->data);
        memset(entry, 0, sizeof *entry);
        entry->next = ui->state_free;
        ui->state_free = index;
        return;
    }
}

void fpas_cui_editor_options(cui_ctx *ui, const char *id, int enabled, int focus) {
    if (!enabled && ui->focus == cui_make_id(ui, id)) ui->focus = 0;
    if (focus && enabled) ui->focus_next_request = cui_make_id(ui, id);
    cui_push_disabled(ui, !enabled);
}

void fpas_cui_scroll_state(cui_ctx *ui, const char *id, int mode, float offset, float *current, float *maximum) {
    cui_scroll_state *state = cui_state(ui, cui_make_id(ui, id), sizeof *state);
    *maximum = CUI_MAX(0, state->content_h - state->view_h);
    *current = state->ty;
    if (mode >= 0) {
        state->ty = mode == 1 ? *maximum : cui__clampf(offset, 0, *maximum);
        state->y = state->ty;
    }
}

int fpas_cui_modal_begin(cui_ctx *ui, const char *id, int open) {
    bool visible = open != 0;
    const cui_dialog_opts options = { .persistent = true, .no_close_button = true };
    return cui_dialog_begin(ui, id, &visible, &options);
}

unsigned int fpas_cui_modal_key(cui_ctx *ui) { return ui->current_layer; }

typedef struct { int count; const fpas_paint *items; } fpas_canvas;
static void fpas_canvas_paint(cui_ctx *ui, cui_rect rect, void *user) {
    const fpas_canvas *canvas = user;
    cui_push_clip(ui, rect);
    for (int i = 0; i < canvas->count; ++i) {
        const fpas_paint *p = &canvas->items[i];
        cui_color color = cui__rgba(p->color & 255, (p->color >> 8) & 255, (p->color >> 16) & 255, p->color >> 24);
        if (p->kind == 0) cui_draw_rect(ui, (cui_rect){rect.x + p->a, rect.y + p->b, p->c, p->d}, CUI_CORNERS(0), color);
        else cui_draw_line(ui, (cui_vec2){rect.x + p->a, rect.y + p->b}, (cui_vec2){rect.x + p->c, rect.y + p->d}, p->thickness, color);
    }
    cui_pop_clip(ui);
}

void fpas_cui_canvas(cui_ctx *ui, const char *id, float width, float height, const fpas_paint *items, int count) {
    fpas_paint *copy = cui__frame_alloc(ui, sizeof *copy * (size_t)count);
    if (count) memcpy(copy, items, sizeof *copy * (size_t)count);
    const fpas_canvas canvas = { count, copy };
    cui_canvas(ui, id, CUI_PX(width), CUI_PX(height), fpas_canvas_paint, &canvas, sizeof canvas);
}

int fpas_cui_focus_key(cui_ctx *ui, unsigned int key) { return key && ui->focus == key; }
int fpas_cui_activate(cui_ctx *ui) { return cui_key_pressed(ui, CUI_KEY_ENTER) || ui->click_count >= 2; }
void fpas_cui_host_begin(cui_ctx *ui) { ui->prev_modal_layer = 0; }
int fpas_cui_choice_row(cui_ctx *ui, const char *id, int enabled, int selected, int focus, int *events) {
    *events = fpas_cui_control(ui, id, "", 2, enabled, selected, 0, focus);
    int node = ui->last_node;
    cui_node *n = &ui->nodes[node];
    n->first = n->last = -1; n->child_count = 0; n->dir = CUI_ROW;
    n->gap = 8;
    cui__open(ui, node);
    return node;
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

float fpas_cui_measure_text(cui_ctx *ui, const char *text) {
    return cui__text_width(ui, CUI_FONT_MEDIUM, cui__fs(ui, ui->theme.text_body), text, (int)strlen(text), false);
}
