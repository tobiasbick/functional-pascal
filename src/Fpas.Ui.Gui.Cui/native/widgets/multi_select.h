#ifndef FPAS_MULTI_SELECT_H
#define FPAS_MULTI_SELECT_H

/** Draws a native multiple-selection picker and reports its proposal.
 * @param ui Current drawing context.
 * @param id Stable picker identity.
 * @param labels Ordered option captions.
 * @param icons Optional glyphs per option.
 * @param count Number of options.
 * @param selected Frame-local selection flags, updated by CUI.
 * @param label Optional field caption.
 * @param placeholder Optional empty selection caption.
 * @param max_chips Number of visible selected chips before the overflow badge.
 * @param searchable Whether to show the native search field.
 * @param enabled Whether the picker accepts input.
 * @param focus Request focus on the field.
 * @param nodes Output field and visible option nodes.
 * @param open Whether the popup accepts input.
 * @return Whether CUI proposed a new selection. */
CUI_API int fpas_cui_multi_select(cui_ctx *ui, const char *id, const char *const *labels,
    const char *const *icons, int count, int *selected, const char *label, const char *placeholder,
    int max_chips, int searchable, int enabled, int focus, int *nodes, int *open);
#endif
