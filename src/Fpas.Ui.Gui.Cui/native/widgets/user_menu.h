#ifndef FPAS_USER_MENU_H
#define FPAS_USER_MENU_H

/** Draws the CUI account trigger and its keyed command menu.
 * @param ui Current UI context.
 * @param id Stable FPAS trigger identity.
 * @param name Account name.
 * @param detail Optional line shown below the account name.
 * @param status Optional status dot color name.
 * @param keys Stable command keys.
 * @param labels Literal row captions.
 * @param icons Optional row glyphs.
 * @param shortcuts Optional display hints.
 * @param flags The row kind and item options used by fpas_cui_menu.
 * @param count Number of menu rows.
 * @param placement Native popup placement ordinal.
 * @param width Logical popup width, zero for the native default.
 * @param compact Use the avatar-only trigger.
 * @param enabled Whether input is accepted.
 * @param focus Request focus on the trigger.
 * @param nodes Output trigger and command nodes, or -1.
 * @param open Whether the menu accepts input.
 * @param highlighted Highlighted entry index, or -1.
 * @return Chosen entry index, or -1. */
CUI_API int fpas_cui_user_menu(cui_ctx *ui, const char *id, const char *name,
    const char *detail, const char *status, const char *const *keys,
    const char *const *labels, const char *const *icons, const char *const *shortcuts,
    const int *flags, int count, int placement, float width, int compact, int enabled,
    int focus, int *nodes, int *open, int *highlighted);
#endif
