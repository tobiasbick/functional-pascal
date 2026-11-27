// [Documentation](../../../../../docs/ui/gui/code/diff-viewer.md)
#include "../../fpas_cui.h"

typedef struct { fpas_diff_observation summary; bool initialized; } fpas_diff_state;

/** Reads a page of cached diff rows from the last presented view. */
CUI_API int fpas_cui_diff_view_rows(cui_ctx *ui, const char *id, cui_diff_row *rows, int first, int capacity) {
    if (fpas_find(ui, cui_make_id(ui, id)) < 0) return 0;
    return cui_diff_view_rows(ui, id, rows, first, capacity);
}

int fpas_cui_diff_view(cui_ctx *ui, const char *id, const char *before, const char *after, int current, const char *language, const char *old_title, const char *new_title, int context, int width_kind, float width, float height, float size, float split_min_width, int flags, fpas_diff_observation *observation) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), slot = key ^ 0xD1FF08u;
    const cui_id focus = key ^ 0xD1FF11u;
    fpas_own(ui, key, slot, key ^ CUI__DIFF_SALT, 0);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    owner->focus = focus;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && (flags & 16)) ui->focus_next_request = focus;
    const cui_diff_opts options = { .mode = flags & 2 ? CUI_DIFF_INLINE : CUI_DIFF_SPLIT,
        .language = language, .old_title = old_title, .new_title = new_title, .current = &current,
        .context = context, .width = fpas_extent(width_kind, width), .height = height, .size = size,
        .split_min_width = split_min_width,
        .no_fold = (flags & 4) != 0, .ignore_eol = (flags & 8) != 0 };
    cui_push_disabled(ui, !enabled);
    const cui_diff_result result = cui_diff_view_ex(ui, id, before, after, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    fpas_diff_state *state = cui_state(ui, slot, sizeof *state);
    const fpas_diff_observation summary = {
        .current = result.current, .count = result.changes, .inline_mode = result.mode == CUI_DIFF_INLINE,
        .added = result.added, .removed = result.removed, .folds = result.folds, .hidden = result.hidden,
        .approximate = result.approximate
    };
    const bool changed = !state->initialized || memcmp(&state->summary, &summary, sizeof summary) != 0;
    *state = (fpas_diff_state){ summary, true };
    *observation = summary;
    observation->current_changed = result.current_changed;
    observation->recomputed = result.recomputed;
    return changed || result.current_changed || result.recomputed;
}
