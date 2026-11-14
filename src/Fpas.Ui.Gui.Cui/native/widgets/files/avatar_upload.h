#ifndef FPAS_AVATAR_UPLOAD_H
#define FPAS_AVATAR_UPLOAD_H

/** Draws CUI's avatar upload.
 * @param ui Current drawing context.
 * @param id Stable identity.
 * @param name Display name for initials.
 * @param path Model-owned image path, or NULL for initials.
 * @param size Logical diameter, or zero for the default.
 * @param enabled Whether the avatar accepts input.
 * @param focus Whether to request focus.
 * @param node Receives the native root.
 * @param has_image Reports whether CUI is displaying an image.
 * @return One on upload, -1 on browse, or zero. */
CUI_API int fpas_cui_avatar_upload(cui_ctx *ui, const char *id, const char *name, const char *path, float size,
    int enabled, int focus, int *node, int *has_image);
#endif
