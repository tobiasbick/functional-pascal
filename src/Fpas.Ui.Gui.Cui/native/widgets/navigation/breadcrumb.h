#ifndef FPAS_BREADCRUMB_H
#define FPAS_BREADCRUMB_H

/** Draws CUI's breadcrumb trail with positional identities.
 * @param ui Current UI-thread context.
 * @param id Stable trail identity.
 * @param labels Borrowed literal path captions.
 * @param count Positive number of path items.
 * @param enabled Whether ancestor links accept input.
 * @param focus Zero or one plus the requested ancestor index.
 * @param nodes Receives the trail and path item node indices.
 * @return Clicked ancestor index, or minus one. */
CUI_API int fpas_cui_breadcrumb(cui_ctx *ui, const char *id, const char *const *labels,
    int count, int enabled, int focus, int *nodes);

#endif
