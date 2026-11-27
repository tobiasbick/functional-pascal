// [Documentation](../../../../../docs/ui/gui/canvas-and-bars/format-bar.md)
#include "../../fpas_cui.h"

typedef struct {
    cui_id color, style, alignment, list, deletion, overflow;
} fpas_format_popups;

static fpas_format_popups fpas_format_popup_ids(cui_id key) {
    return (fpas_format_popups){
        .color = cui__hash_str("#color", key) ^ 0xC0105u,
        .style = cui__hash_str("#style", key) ^ 0x57713u,
        .alignment = cui__hash_str("", cui__hash_str("Alignment", key)) ^ 0xA119u,
        .list = cui__hash_str(CUI_ICON_LIST, cui__hash_str("List", key)) ^ 0x1157u,
        .deletion = cui__hash_str(CUI_ICON_TRASH_2, cui__hash_str("Delete", key)) ^ 0xDE1u,
        .overflow = key ^ 0x0F10Fu
    };
}

static void fpas_format_close_popup(cui_ctx *ui, cui_id id) {
    cui_popup *popup = cui__popup_get(ui, id, false);
    if (!popup || (!popup->open && popup->s.x <= 0)) return;
    popup->open = false;
    memset(&popup->s, 0, sizeof popup->s);
    for (int i = 0; i < ui->prev_focus_count; ++i) {
        const cui__focus_entry *entry = &ui->prev_focus_order[i];
        if (entry->layer == id && entry->id == ui->focus) ui->focus = 0;
    }
}

static void fpas_format_close_popups(cui_ctx *ui, cui_id key) {
    const fpas_format_popups popups = fpas_format_popup_ids(key);
    fpas_format_close_popup(ui, popups.color);
    fpas_format_close_popup(ui, popups.style);
    fpas_format_close_popup(ui, popups.alignment);
    fpas_format_close_popup(ui, popups.list);
    fpas_format_close_popup(ui, popups.deletion);
    fpas_format_close_popup(ui, popups.overflow);
}

/* CUI-162 (docs/future/cui-upstream-bugs.md#cui-162): keyboard opening needs a
   focused choice even if the current tone is outside the palette. */
static void fpas_format_color_focus(cui_ctx *ui, cui_id id, cui_id trigger) {
    const cui_popup *popup = cui__popup_get(ui, id, false);
    if (!popup || !popup->open || popup->opened_frame != ui->frame ||
        ui->focus != trigger || !ui->focus_visible) return;
    const int first = 0;
    ui->focus = cui__hash(&first, sizeof first, id);
    cui_request_redraw(ui);
}

/* CUI-164 (docs/future/cui-upstream-bugs.md#cui-164): the style trigger uses
   literal text, and the menu must display the same name before layout. */
static void fpas_format_style_labels(cui_ctx *ui, cui_id id, const cui_text_style *styles, int count) {
    if (!styles || count <= 0) return;
    const int menu = fpas_find(ui, id);
    if (menu < 0) return;
    int item = ui->nodes[menu].first;
    for (int i = 0; i < count && item >= 0; ++i, item = ui->nodes[item].next) {
        const char *name = styles[i].name;
        if (!name || !strstr(name, "##")) continue;
        const int label = ui->nodes[item].first;
        if (label >= 0) fpas_text(ui, label, name);
    }
}

int fpas_cui_format_bar(cui_ctx *ui, const char *id, cui_text_format *format, const cui_text_style *styles,
    int style_count, const cui_tone *colors, int color_count, const char *footer, const char *keys,
    const char *question, int flags) {
    if (!format) return 0;
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const fpas_format_popups popups = fpas_format_popup_ids(key);
    fpas_own(ui, key, key ^ 0xF0F0u, 0, 0);
    fpas_on_forget(ui, key, fpas_format_close_popups);
    const cui_format_bar_opts options = { .styles = styles, .style_count = style_count,
        .colors = colors, .color_count = color_count, .footer = footer, .footer_keys = keys,
        .delete_question = question, .no_delete = (flags & 2) != 0, .no_more = (flags & 4) != 0 };
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    /* CUI-163 (docs/future/cui-upstream-bugs.md#cui-163): the open Delete
       question handles Enter outside the native disabled check. */
    if (!enabled) fpas_format_close_popups(ui, key);
    else if (options.no_delete) fpas_format_close_popup(ui, popups.deletion);
    cui_text_format display = *format;
    cui_push_disabled(ui, !enabled);
    const int event = cui_format_bar_ex(ui, id, enabled ? format : &display, &options);
    cui_pop_disabled(ui);
    fpas_format_color_focus(ui, popups.color, popups.color ^ 0xC0105u);
    fpas_format_style_labels(ui, popups.style, styles, style_count);
    fpas_creative_node(ui, root, key);
    return enabled ? event : 0;
}
