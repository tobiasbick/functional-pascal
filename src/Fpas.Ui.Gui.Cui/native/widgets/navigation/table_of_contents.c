// [Documentation](../../../../../docs/ui/gui/reader/table-of-contents.md)
#include "../../fpas_cui.h"

typedef struct { cui_id id; float y; } fpas_toc_anchor_position;

static void fpas_toc_position_anchors(cui_ctx *ui, const fpas_toc_anchor_position *positions,
    int count, float offset) {
    for (int index = 0; index < count; ++index)
        cui__state_find(ui, positions[index].id)->rect.y = positions[index].y + offset;
}

int fpas_cui_table_of_contents(cui_ctx *ui, const char *scroll_id,
    const char *const *titles, const char *const *anchors, const int *levels,
    int count, int enabled, int focus, int *node, int *rows) {
    cui_toc_item items[64];
    if (count < 1 || count > 64) return -1;
    for (int index = 0; index < count; ++index)
        items[index] = (cui_toc_item){ titles[index], anchors[index], levels[index] };

    const cui_id scroll_key = cui_make_id(ui, scroll_id);
    const cui_id key = scroll_key ^ 0x70CCu;
    if (enabled && !cui_is_disabled(ui) && focus > 0 && focus <= count) {
        const int index = focus - 1;
        ui->focus_next_request = cui__hash(&index, sizeof index, key);
    }

    /* CUI-054: the previous anchor positions reflect the laid-out scroll offset.
       Project them to the offset used by CUI for jumps and active selection.
       See docs/future/cui-upstream-bugs.md#cui-054. */
    const cui_scroll_state *scroll = cui_state(ui, scroll_key, sizeof *scroll);
    const float offset = scroll->laid_y - scroll->y;
    const float target = scroll->ty;
    fpas_toc_anchor_position positions[64];
    int positioned = 0;
    if (offset != 0) {
        for (int index = 0; index < count; ++index) {
            const cui_id anchor_key = cui__anchor_id(items[index].anchor);
            const cui_state_entry *anchor = cui__state_find(ui, anchor_key);
            if (anchor && anchor->has_rect)
                positions[positioned++] = (fpas_toc_anchor_position){anchor_key, anchor->rect.y};
        }
        fpas_toc_position_anchors(ui, positions, positioned, offset);
    }
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int active = cui_toc(ui, scroll_id, items, count);
    cui_pop_disabled(ui);
    fpas_toc_position_anchors(ui, positions, positioned, 0);
    /* CUI-055: a reduced-motion scroll can finish without another animation frame.
       See docs/future/cui-upstream-bugs.md#cui-055. */
    if (offset != 0 || scroll->ty != target) cui_request_redraw(ui);
    if (rows) {
        for (int index = 0; index < count; ++index)
            rows[index] = fpas_find(ui, cui__hash(&index, sizeof index, key));
    }
    ui->last_node = *node;
    return active;
}
