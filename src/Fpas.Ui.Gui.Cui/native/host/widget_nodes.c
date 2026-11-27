// [Documentation](../../../../docs/ui/gui/cui.md#layout-and-control-inspection)
#include "../fpas_cui.h"
#include <limits.h>

/* Shared bridge encoding: content, fill, logical pixels, percentage. */
static cui_size fpas_extent(int kind, float value) {
    switch (kind) {
        case 1: return CUI_FILL;
        case 2: return CUI_PX(value);
        case 3: return CUI_PCT(value);
        default: return CUI_FIT;
    }
}

static cui_button_variant fpas_button_variant(int variant) {
    switch (variant) {
        case 1: return CUI_BUTTON_PRIMARY;
        case 2: return CUI_BUTTON_SECONDARY;
        case 4: return CUI_BUTTON_DANGER;
        case 5: return CUI_BUTTON_OUTLINE;
        case 6: return CUI_BUTTON_GHOST;
        case 7: return CUI_BUTTON_LINK;
        default: return CUI_BUTTON_DEFAULT;
    }
}

static int fpas_find(cui_ctx *ui, cui_id id) {
    for (int i = ui->node_count - 1; i >= 0; --i) if (ui->nodes[i].id == id) return i;
    return -1;
}

/** Uses the previous painted hit geometry, including clipping and blockers. */
static bool fpas_interactive_at(cui_ctx *ui, cui_id key, float x, float y) {
    for (int i = ui->prev_hit_count - 1; i >= 0; --i) {
        const cui_hit *hit = &ui->prev_hits[i];
        if (!cui__rect_contains(hit->rect, x, y)) continue;
        if (hit->flags & CUI_NF_INTERACTIVE) return hit->id == key;
        if (hit->flags & (CUI_NF_BLOCKER | CUI_NF_HIT_OPAQUE)) return false;
    }
    return false;
}

/** CUI-100: take the position at release, even if there was no intervening frame. */
static bool fpas_drag_release(cui_ctx *ui, cui_id key, bool dragging, float *fraction) {
    if (!ui->window_focused || !ui->mreleased[CUI_MOUSE_LEFT] || ui->mdown[CUI_MOUSE_LEFT]) return false;
    if (!dragging && (!ui->mpressed[CUI_MOUSE_LEFT] || (ui->active && ui->active != key) ||
        !fpas_interactive_at(ui, key, ui->press_x, ui->press_y))) return false;
    const cui_rect bounds = cui__prev_rect(ui, key);
    if (bounds.w <= 0) return false;
    if (fraction) *fraction = cui__clampf((ui->release_x - bounds.x) / bounds.w, 0, 1);
    return true;
}

/** Reads CUI drag input without taking focus or consuming the interaction. */
static cui_interact fpas_probe_drag_interaction(cui_ctx *ui, cui_id key) {
    const int count = ui->focus_count;
    const cui_id focus = ui->focus, requested = ui->focus_next_request, active = ui->active;
    const cui_id tooltip = ui->hover_id_for_tooltip;
    const bool visible = ui->focus_visible;
    const cui_interact result = cui__interact(ui, key, CUI_IF_FOCUSABLE | CUI_IF_DRAG);
    ui->focus_count = count;
    ui->focus = focus;
    ui->focus_next_request = requested;
    ui->active = active;
    ui->hover_id_for_tooltip = tooltip;
    ui->focus_visible = visible;
    return result;
}

/** Determines CUI's intended focus style and leaves the input for the widget. */
static bool fpas_drag_focus_ring(cui_ctx *ui, cui_id key) {
    if (ui->focus != key && ui->focus_next_request != key &&
        !(ui->hot == key && ui->mpressed[CUI_MOUSE_LEFT])) return false;
    return fpas_probe_drag_interaction(ui, key).focus_visible;
}

typedef struct {
    int count;
    cui_popup first;
} fpas_focus_cover;

/** Keeps native keyboard focus while protecting a widget's only paint slot. */
static fpas_focus_cover fpas_begin_focus_cover(cui_ctx *ui) {
    const fpas_focus_cover saved = { ui->popup_count, ui->popups[0] };
    ui->popups[0] = (cui_popup){ .id = ui->current_layer ^ 1u, .open = true, .z = INT_MAX };
    ui->popup_count = 1;
    ui->focus_visible = false;
    return saved;
}

static void fpas_end_focus_cover(cui_ctx *ui, fpas_focus_cover saved) {
    ui->popups[0] = saved.first;
    ui->popup_count = saved.count;
    ui->focus_visible = true;
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
static void fpas_text(cui_ctx *ui, int node, const char *text) {
    const int length = (int)strlen(text);
    ui->nodes[node].text = cui__frame_strdup(ui, text, length);
    ui->nodes[node].text_len = length;
}

static void fpas_literal(cui_ctx *ui, int node, const char *text, cui_color color) {
    int label = fpas_label_node(ui, node);
    if (label < 0) {
        cui__open(ui, node);
        label = cui__text_node(ui, text, -1, CUI_FONT_MEDIUM, cui__fs(ui, ui->theme.text_body), color, false);
        cui__close(ui);
    } else {
        fpas_text(ui, label, text);
    }
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}

/** Makes the native document view clamp its offset after a content size change. */
static void fpas_document_scroll_layout(cui_ctx *ui, cui_node *node, void *data) {
    (void)data;
    /* CUI-065: CUI clamps against the new content bounds on the next build.
       See docs/future/cui-upstream-bugs.md#cui-065. */
    const cui_scroll_state *state = cui_state(ui, node->id, sizeof *state);
    const float width = CUI_MAX(0, node->content_w - node->rect.w);
    const float height = CUI_MAX(0, node->content_h - node->rect.h);
    if (state->tx > width || state->x > width || state->ty > height || state->y > height)
        cui_request_redraw(ui);
}

/** Gives wheel input to the enclosing viewport on the next frame. */
static void fpas_disabled_document_scroll_layout(cui_ctx *ui, cui_node *node, void *data) {
    fpas_document_scroll_layout(ui, node, data);
    if (ui->hit_count > 0 && ui->hits[ui->hit_count - 1].id == node->id)
        ui->hits[ui->hit_count - 1].flags &= ~(CUI_NF_SCROLL_X | CUI_NF_SCROLL_Y);
}

/** Drops input already routed to a document viewport that has become disabled. */
static void fpas_document_scroll_input(cui_ctx *ui, cui_id viewport, bool enabled) {
    if (enabled) return;
    if (ui->scroll_target == viewport) ui->scroll_target = 0;
    cui_scroll_state *state = cui__state_lazy(ui, viewport, sizeof *state, false);
    if (state) state->drag_x = state->drag_y = false;
}
