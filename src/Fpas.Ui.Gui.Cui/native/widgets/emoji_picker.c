#include "../fpas_cui.h"

static int fpas_emoji_part(cui_ctx *ui, cui_id key, int first) {
    const int node = fpas_find(ui, key);
    return node < first ? -1 : node;
}

const char *fpas_cui_emoji_picker(cui_ctx *ui, const char *id, char *recent, size_t capacity,
    int *tone, float width, float height, float size, const char *placeholder, const char *empty_text,
    const char *hint, int search, int preview, int tone_selector, int enabled, int focus, int *nodes) {
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    cui_push_id(ui, id);
    const cui_id search_key = cui_make_id(ui, "#search"), grid = cui_make_id(ui, "#grid");
    const cui_id wheel = grid ^ 0x3E1Du;
    cui_pop_id(ui);
    fpas_own(ui, key, search_key, wheel, 0);
    if (allowed && focus) ui->focus_next_request = focus == 1 && search ? search_key : grid;
    if (allowed && !search && ui->focus == search_key) ui->focus = grid;
    if (!allowed || !tone_selector) {
        cui__em_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        if (state) {
            state->tone_open = false;
            if (!allowed) state->pressing = state->dragging = false;
        }
    }

    const cui_emoji_picker_opts options = {
        .width = width > 0 ? CUI_PX(width) : CUI_FIT, .height = height, .size = size,
        .recent = recent, .recent_cap = capacity, .tone = tone,
        .placeholder = placeholder, .empty_text = empty_text ? cui__frame_strdup(ui, empty_text, -1) : NULL,
        .hint = hint, .no_search = !search, .no_preview = !preview, .no_tone = !tone_selector
    };
    const int first = ui->node_count;
    const cui_id scroll = ui->scroll_target;
    if (!allowed) {
        if (scroll == wheel) ui->scroll_target = 0;
        if (ui->focus == grid || ui->focus == search_key) ui->focus = 0;
    }
    cui_push_disabled(ui, !enabled);
    const char *picked = cui_emoji_picker_ex(ui, id, &options);
    cui_pop_disabled(ui);
    if (!allowed) ui->scroll_target = scroll;
    if (allowed && !search && ui->focus == search_key) ui->focus = grid;

    ui->last_node = fpas_find(ui, key);
    ui->last_id = key;
    if (nodes) {
        nodes[0] = ui->last_node;
        nodes[1] = fpas_emoji_part(ui, search_key, first);
        nodes[2] = fpas_emoji_part(ui, grid, first);
        cui_push_id(ui, id);
        nodes[3] = fpas_emoji_part(ui, cui_make_id(ui, "#tones"), first);
        for (int i = 0; i < 10; ++i) {
            cui_push_id_int(ui, i);
            if (i < 6) nodes[4 + i] = fpas_emoji_part(ui, cui_make_id(ui, "#tone"), first);
            nodes[10 + i] = fpas_emoji_part(ui, cui_make_id(ui, "#tab"), first);
            cui_pop_id(ui);
        }
        cui_pop_id(ui);
    }
    return allowed ? picked : NULL;
}
