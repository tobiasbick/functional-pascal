#include "../fpas_cui.h"

int fpas_cui_hex_view(cui_ctx *ui, const char *id, uint8_t *bytes, int count, int *start, int *length, int *cursor, const fpas_hex_mark *marks, int mark_count, uint64_t address, int go_to, int bytes_per_row, int width_kind, float width, float height, float size, int flags, int *edit_offset, int *edit_length) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), focus = key ^ 0x4E5B0D1u;
    fpas_own(ui, key, key ^ CUI__HEX_SALT, 0, 0);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    owner->focus = focus;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && (flags & 16)) ui->focus_next_request = focus;
    cui_hex_selection selection = {(size_t)*start, (size_t)*length, (size_t)*cursor};
    cui_hex_range *ranges = mark_count > 0 ? cui__frame_alloc(ui, (size_t)mark_count * sizeof *ranges) : NULL;
    for (int i = 0; i < mark_count; ++i)
        ranges[i] = (cui_hex_range){(size_t)marks[i].start, (size_t)marks[i].length,
            (cui_tone)marks[i].tone, marks[i].label ? cui__frame_strdup(ui, marks[i].label, -1) : NULL};
    const cui_hex_view_opts options = { .width = fpas_extent(width_kind, width),
        .height = height, .size = size, .bytes_per_row = bytes_per_row, .base = address,
        .ranges = ranges, .range_count = mark_count, .edit = enabled && (flags & 2) ? bytes : NULL,
        .no_ascii = (flags & 4) != 0, .no_header = (flags & 8) != 0 };
    if (enabled && go_to >= 0) cui_hex_view_goto(ui, id, (size_t)go_to);
    cui_push_disabled(ui, !enabled);
    const cui_hex_result result = cui_hex_view_ex(ui, id, bytes, (size_t)count, &selection, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    *start = (int)selection.start;
    *length = (int)selection.length;
    *cursor = (int)selection.cursor;
    *edit_offset = (int)result.edit_offset;
    *edit_length = (int)result.edit_length;
    return enabled ? (result.selection_changed || go_to >= 0 ? 1 : 0) | (result.edited ? 2 : 0) : 0;
}
