// [Documentation](../../../../../docs/ui/gui/code/code-block.md)
#include "../../fpas_cui.h"
#include <limits.h>

/** Uses four spaces per tab for paint while preserving the original text for copy. */
static void fpas_code_expand_tabs(cui_ctx *ui, cui_node *canvas, cui__code_paint *paint) {
    const char *tab = strchr(paint->code, '\t');
    if (!tab) return;
    /* CUI-064: count each tab as four spaces inside a mixed whitespace run.
       See docs/future/cui-upstream-bugs.md#cui-064. */
    int tabs = 0;
    for (const char *at = tab; at; at = strchr(at + 1, '\t')) ++tabs;
    if (tabs > (INT_MAX - paint->len) / 3) return;
    const int length = paint->len + 3 * tabs;
    char *expanded = cui__frame_alloc(ui, (size_t)length + 1);
    char *write = expanded;
    for (const char *read = paint->code; *read; ++read) {
        if (*read == '\t') {
            memset(write, ' ', 4);
            write += 4;
        } else *write++ = *read;
    }
    *write = '\0';
    paint->code = expanded;
    paint->len = length;
    canvas->min_w = cui__text_width(ui, CUI_FONT_MONO, paint->size, expanded, length, true) +
        paint->gutter + 2 * paint->pad + 8;
}

/** Preserves indented lines in the native plain-text painter. */
static void fpas_paint_indented_plain_code(cui_ctx *ui, cui_node *node, void *data) {
    cui__paint_code(ui, node, data);
    /* CUI-063: CUI skips the complete plain token when the line begins with a space.
       See docs/future/cui-upstream-bugs.md#cui-063. */
    const cui__code_paint *paint = data;
    const char *end = paint->code + paint->len;
    const float origin_x = node->rect.x + paint->pad + paint->gutter;
    const float font_height = cui__font_line_height(ui, CUI_FONT_MONO, paint->size);
    float y = node->rect.y + paint->pad + (paint->lh - font_height) * 0.5f - 1;
    for (const char *line = paint->code; line < end; y += paint->lh) {
        const char *next = strchr(line, '\n');
        const char *limit = next ? next : end;
        if (*line == ' ')
            cui__draw_text_line(ui, CUI_FONT_MONO, paint->size, origin_x, y, paint->colors[TK_TEXT],
                line, (int)(limit - line), true);
        line = next ? next + 1 : end;
    }
}

/** Applies the same text paint rules to standalone and embedded code blocks. */
static void fpas_prepare_code_paint(cui_ctx *ui, cui_node *canvas) {
    cui__code_paint *paint = canvas->paint_data;
    fpas_code_expand_tabs(ui, canvas, paint);
    if (paint->lang.plain) canvas->paint = fpas_paint_indented_plain_code;
}

/** Builds a native code block and wires its scroll and copy controls. */
void fpas_cui_code_block(cui_ctx *ui, const char *id, const char *code, const char *language,
    const char *title, int first, int last, float height, float size, int flags, int *copy) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id scroll = key ^ 0x5C0Du;
    const bool enabled = (flags & 4) != 0, allowed = enabled && !cui_is_disabled(ui);
    const cui_code_opts options = { .lang = language, .title = title,
        .line_numbers = (flags & 1) != 0, .highlight_from = first, .highlight_to = last,
        .max_height = height, .size = size, .no_header = (flags & 2) != 0 };
    cui_push_id(ui, id);
    const cui_id state_key = cui_make_id(ui, "copy") ^ 0xC0B7u;
    cui_push_id(ui, "copy");
    const cui_id copy_key = cui_make_id(ui, "#copy-button");
    cui_pop_id(ui);
    cui_pop_id(ui);
    if ((title || language) && !options.no_header && allowed && (flags & 8))
        ui->focus_next_request = copy_key;
    fpas_document_scroll_input(ui, scroll, allowed);
    cui_push_disabled(ui, !enabled);
    cui_code_block_ex(ui, id, code, &options);
    cui_pop_disabled(ui);
    cui_node *canvas = &ui->nodes[ui->last_node];
    fpas_prepare_code_paint(ui, canvas);
    ui->nodes[canvas->parent].paint = allowed ? fpas_document_scroll_layout : fpas_disabled_document_scroll_layout;
    *copy = fpas_find(ui, copy_key);
    fpas_own(ui, key, scroll, state_key, 0);
    ui->last_node = node;
    ui->last_id = key;
}
