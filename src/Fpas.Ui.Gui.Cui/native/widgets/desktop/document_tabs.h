#ifndef FPAS_CUI_DOCUMENT_TABS_H
#define FPAS_CUI_DOCUMENT_TABS_H

/* All operations use ui, the current UI-thread context. id is a stable retained identity.
 * Returned node indices are valid only in the frame that produced them. */

/** Draws document tabs; titles/icons/tips/dirty have count entries. titles use native
 * title###stable-key syntax, with unique keys; the adapter copies their strings for this frame.
 * icons/tips use empty strings for absent values; dirty contains unsaved-state booleans.
 * count is the nonnegative number of tabs. selected is an input/output index (-1 for none). flags: 1 shortcuts, 2 no close,
 * 4 no reorder, 8 input enabled, 16 focus selected tab. close receives the tab to close;
 * from/to receive source/destination positions of a move, or -1 for no proposal; nodes[count+1] receives strip then tab nodes (-1 if absent).
 * All node indices are frame-local. Makes the strip the last layout target. */
CUI_API void fpas_cui_document_tabs(cui_ctx *ui, const char *id, const char *const *titles,
    const char *const *icons, const char *const *tips, const int *dirty, int count, int *selected,
    int flags, int *close, int *from, int *to, int *nodes);

#endif
