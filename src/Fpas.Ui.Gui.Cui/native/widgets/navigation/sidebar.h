#ifndef FPAS_SIDEBAR_H
#define FPAS_SIDEBAR_H

/** Clears the retained highlight when the sidebar has no active destination.
 * @param ui Owner context.
 * @param group Stable sidebar identity. */
CUI_API void fpas_cui_sidebar_clear(cui_ctx *ui, const char *group);
/** Draws a native sidebar destination with a literal caption.
 * @param ui Owner context.
 * @param group Stable identity of the sidebar's highlight.
 * @param id Stable item identity.
 * @param text Literal caption.
 * @param icon Optional icon glyph, or NULL.
 * @param badge Optional literal badge, or NULL.
 * @param enabled Input permission.
 * @param selected Whether the model marks the destination active.
 * @param focus Requests focus before drawing.
 * @return Nonzero on activation. */
CUI_API int fpas_cui_sidebar_item(cui_ctx *ui, const char *group, const char *id, const char *text,
                                 const char *icon, const char *badge, int enabled, int selected, int focus);

#endif
