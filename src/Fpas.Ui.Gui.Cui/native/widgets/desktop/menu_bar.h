// [Documentation](../../../../../docs/ui/gui/app-chrome/menu-bar.md)
#ifndef FPAS_CUI_MENU_BAR_H
#define FPAS_CUI_MENU_BAR_H

#include "../overlays/menu.h"

/* All operations use ui, the current UI-thread context. id is a stable retained identity.
 * Returned node indices are valid only in the frame that produced them. */

/** Begins a menu bar; pair with end in the same frame. border controls its bottom border;
 * enabled gates bar input and cancels its open menu when disabled. */
CUI_API void fpas_cui_menu_bar_begin(cui_ctx *ui, const char *id, int border, int enabled);
/** Closes the matching bar, retires an open menu no longer submitted, and makes its row
 * the last layout target. */
CUI_API void fpas_cui_menu_bar_end(cui_ctx *ui);
/** Submits one menu and its rows inside a bar. title uses native keyed-label syntax;
 * mnemonic is a native mnemonic character, width is popup width, enabled gates the menu.
 * keys/labels/icons/shortcuts/rows contain count entries; labels are literal captions.
 * nodes[count+1] receives trigger then visible command nodes (-1 for unavailable rows).
 * open reports acceptance by an open popup; highlighted receives row index or -1.
 * Returns chosen row index or -1. Nodes are frame-local; no separate popup end is required. */
CUI_API int fpas_cui_menu_bar_menu(cui_ctx *ui, const char *title, int mnemonic, float width,
    int enabled, const char *const *keys, const char *const *labels, const char *const *icons,
    const char *const *shortcuts, const fpas_cui_menu_row *rows, int count, int *nodes, int *open, int *highlighted);

#endif
