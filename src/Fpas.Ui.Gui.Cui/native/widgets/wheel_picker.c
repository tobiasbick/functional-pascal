#include "../fpas_cui.h"

static cui_id fpas_wheel_scroll_key(cui_id key) { return key ^ 0x57484545u; }

static void fpas_paint_wheel_picker(cui_ctx *ui, cui_node *node, void *data) {
    cui__paint_wheel(ui, node, data);

    /* CUI-009 (docs/future/cui-upstream-bugs.md#cui-009): register the wheel as
       the vertical scroll target before an enclosing viewport consumes the event. */
    const cui_id key = fpas_wheel_scroll_key(node->id);
    cui_scroll_state *scroll = cui_state(ui, key, sizeof *scroll);
    scroll->view_h = node->rect.h;
    scroll->content_h = scroll->view_h + 1;
    const cui_rect pixels = ui->dl.clip_stack[ui->dl.clip_depth - 1];
    const float scale = ui->scale;
    const cui_rect clip = {pixels.x / scale, pixels.y / scale, pixels.w / scale, pixels.h / scale};
    const cui_rect bounds = cui__xform_rect(ui, node->rect);
    cui__paint_add_hit(ui, key, cui__rect_intersect(bounds, clip), CUI_NF_SCROLL_Y);
}

static void fpas_paint_focused_wheel_picker(cui_ctx *ui, cui_node *node, void *data) {
    fpas_paint_wheel_picker(ui, node, data);
    cui__draw_focus_ring(ui, node->rect, node->radius, 1.0f);
}

int fpas_cui_wheel_picker(cui_ctx *ui, const char *id, const char *const *items, int count,
    int index, float width, int rows, int loop, int join_left, int join_right, int enabled,
    int focus, int *node) {
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    if (!allowed) {
        cui__wheel_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        if (state) state->dragging = false;
    }

    /* CUI reads the captions during painting, after the managed call returns. */
    const char **captions = cui__frame_alloc(ui, (size_t)count * sizeof *captions);
    for (int i = 0; i < count; ++i) captions[i] = cui__frame_strdup(ui, items[i], -1);

    const cui_wheel_opts options = { .width = width > 0 ? CUI_PX(width) : CUI_FIT, .rows = rows,
        .loop = loop != 0, .join_left = join_left != 0, .join_right = join_right != 0 };
    int selected = index;
    cui_push_disabled(ui, !enabled);
    cui_wheel_picker_ex(ui, id, captions, count, &selected, &options);
    cui_pop_disabled(ui);
    *node = fpas_find(ui, key);
    ui->last_node = *node;
    cui_node *wheel = &ui->nodes[*node];
    if (wheel->paint == cui__paint_focus_after) {
        /* CUI-010 (docs/future/cui-upstream-bugs.md#cui-010): the focus ring
           replaced CUI's wheel paint data. Give both painters their inputs. */
        const cui__wheel_state *state = cui_state(ui, key, sizeof *state);
        const int visible_rows = rows | 1;
        cui__wheel_paint *paint = cui__frame_alloc(ui, sizeof *paint);
        *paint = (cui__wheel_paint){
            .items = captions, .count = count, .pos = state->pos.x, .rows = visible_rows,
            .row_h = wheel->h.value / (float)visible_rows, .loop = options.loop,
            .join_l = options.join_left, .join_r = options.join_right,
            .band = CUI_C(ui, ACTIVE), .fg = CUI_C(ui, FOREGROUND), .dim = CUI_C(ui, TEXT_MUTED),
            .font_size = cui__fs(ui, ui->theme.text_body + 2)
        };
        wheel->paint_data = paint;
        wheel->paint = fpas_paint_focused_wheel_picker;
    } else if (allowed) {
        wheel->paint = fpas_paint_wheel_picker;
    }
    fpas_own(ui, key, fpas_wheel_scroll_key(key), 0, 0);
    return allowed ? selected : index;
}
