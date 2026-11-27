// [Documentation](../../../../../docs/ui/gui/social-and-media/counter.md)
#include "../../fpas_cui.h"

/** Combines native byte fragments into complete text glyphs before layout. */
static void fpas_counter_utf8(cui_ctx *ui, int row) {
    /* CUI-072 (docs/future/cui-upstream-bugs.md#cui-072): native counter builds
       one text node per byte, even when a format affix contains UTF-8. */
    for (int at = ui->nodes[row].first; at >= 0; at = ui->nodes[at].next) {
        cui_node *lead = &ui->nodes[at];
        if (!(lead->flags & CUI_NF_TEXT) || lead->text_len != 1) continue;
        const unsigned char byte = (unsigned char)lead->text[0];
        if (byte < 0xC2u || byte > 0xF4u) continue;
        const int expected = byte < 0xE0u ? 2 : byte < 0xF0u ? 3 : 4;
        char scalar[5] = { lead->text[0] };
        int size = 1, next = lead->next;
        while (size < expected && next >= 0) {
            const cui_node *part = &ui->nodes[next];
            if (!(part->flags & CUI_NF_TEXT) || part->text_len != 1
                || ((unsigned char)part->text[0] & 0xC0u) != 0x80u) break;
            scalar[size++] = part->text[0];
            next = part->next;
        }
        /* Drop any incomplete final scalar from CUI's fixed-size display buffer. */
        fpas_text(ui, at, size == expected ? scalar : "");
        ui->nodes[at].next = next;
        ui->nodes[row].child_count -= size - 1;
        if (next < 0) ui->nodes[row].last = at;
    }
}

/** Draws CUI's counter using an explicit font role and safe value format. */
void fpas_cui_counter(cui_ctx *ui, const char *id, double value, const char *format,
    float size, int palette, uint32_t rgba, int group, int font) {
    const int first = ui->node_count;
    const cui_counter_opts options = { .format = format, .size = size, .font = (cui_font)font,
        .color = fpas_chart_color(palette, rgba), .group_thousands = group != 0 };
    cui_counter_ex(ui, id, value, &options);
    if (format) fpas_counter_utf8(ui, first);
    fpas_creative_node(ui, first, cui_make_id(ui, id));
}
