#ifndef FPAS_AVATAR_H
#define FPAS_AVATAR_H

/** Draws the native avatar.
 * @param ui Current drawing context.
 * @param id Stable identity.
 * @param name Name used for initials and their color.
 * @param path Optional image file path.
 * @param status Optional presence name.
 * @param size Logical diameter, or zero for CUI's default.
 * @param square Whether to draw a rounded square.
 * @param node Receives the native avatar root.
 * @return Whether the native avatar has a decoded image. */
CUI_API int fpas_cui_avatar(cui_ctx *ui, const char *id, const char *name, const char *path,
    const char *status, float size, int square, int *node);
#endif
