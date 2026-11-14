#ifndef FPAS_DISCLOSURE_H
#define FPAS_DISCLOSURE_H

/** Begins CUI's accordion (kind 0) or expandable card (kind 1).
 * id is stable; title, description, icon and badge are borrowed literal strings.
 * open is model-owned; enabled gates header input and focus requests use focus.
 * node receives the outer layout node, header receives the focusable header.
 * Returns draw-body bit 1, changed bit 2 and proposed-open bit 4.
 * A drawn body must be closed with fpas_cui_disclosure_end. */
CUI_API int fpas_cui_disclosure_begin(cui_ctx *ui, const char *id, const char *title,
    int kind, int open, int enabled, int focus, const char *description,
    const char *icon, const char *badge, int tone, int *node, int *header);

/** Ends the matching native disclosure body and restores its outer layout node. */
CUI_API void fpas_cui_disclosure_end(cui_ctx *ui, int kind, int node);

/** Draws native expandable text; lines/size/colors map directly to CUI options.
 * colors uses bit 1 for color and bit 2 for fade_to; RGBA starts in the low byte.
 * enabled gates input; focus requests the native expansion link if present.
 * toggle receives the link node or -1. Returns changed bit 1, expanded bit 2.
 * The last node is restored to the text block for host sizing and ownership. */
CUI_API int fpas_cui_expandable_text(cui_ctx *ui, const char *id, const char *text,
    int lines, float size, int colors, unsigned int color, unsigned int fade_to,
    int enabled, int focus, int *toggle);

#endif
