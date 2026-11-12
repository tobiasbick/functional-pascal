#ifndef FPAS_DROPZONE_H
#define FPAS_DROPZONE_H

/** Draws CUI's dropzone.
 * @param ui Current drawing context.
 * @param id Stable identity.
 * @param title Optional literal title.
 * @param description Optional literal description.
 * @param accept Optional semicolon-separated extensions.
 * @param icon Optional icon glyph.
 * @param height Tall variant height, or zero for the default.
 * @param compact Whether to draw the compact row.
 * @param enabled Whether the dropzone accepts input.
 * @param focus Whether to request focus.
 * @param node Receives the native root.
 * @return Accepted file count, -1 on browse, or zero. */
CUI_API int fpas_cui_dropzone(cui_ctx *ui, const char *id, const char *title, const char *description,
    const char *accept, const char *icon, float height, int compact, int enabled, int focus, int *node);
#endif
