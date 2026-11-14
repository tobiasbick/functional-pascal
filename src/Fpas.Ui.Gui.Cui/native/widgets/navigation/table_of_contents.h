#ifndef FPAS_CUI_TABLE_OF_CONTENTS_H
#define FPAS_CUI_TABLE_OF_CONTENTS_H

/* Draws a native outline for a viewport in the current scope and returns its active index.
 * Arrays have count entries (1 through 64). Focus is one-based, with zero for no request.
 * Writes the native root and, when supplied, the node of each focusable row. */
CUI_API int fpas_cui_table_of_contents(cui_ctx *ui, const char *scroll_id,
    const char *const *titles, const char *const *anchors, const int *levels,
    int count, int enabled, int focus, int *node, int *rows);

#endif
