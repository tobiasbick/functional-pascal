#ifndef FPAS_PAGINATION_H
#define FPAS_PAGINATION_H

/** Draws CUI's pager with a controlled zero-based page.
 * @param ui Current UI-thread context.
 * @param id Stable pager identity.
 * @param page Current page index.
 * @param count Positive page count.
 * @param enabled Whether the pager accepts input.
 * @param focus Page index, -1 for Previous, -2 for Next, or -3 for none.
 * @param parts Receives up to nine pairs of button index and native node.
 * @param used Receives the number of reported pairs.
 * @return Proposed page index, or minus one when unchanged. */
CUI_API int fpas_cui_pagination(cui_ctx *ui, const char *id, int page, int count,
    int enabled, int focus, int *parts, int *used);

#endif
