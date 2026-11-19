// [Documentation](../../../../../docs/ui/gui/social-and-media/presence.md)
#ifndef FPAS_CUI_PRESENCE_H
#define FPAS_CUI_PRESENCE_H

/** A collaborator shown in the presence strip. */
typedef struct {
    const char *name; /**< Display name for tooltips and initials. */
    const char *avatar; /**< Optional UTF-8 path to an avatar image. */
    int palette; /**< Ring color palette index, or -1 for a custom color. */
    uint32_t rgba; /**< Packed custom ring color. */
    int idle; /**< Nonzero when the collaborator is away. */
} fpas_presence_user;

/** Draws a presence strip and retains images for its visible avatars.
 * @param ui Drawing context.
 * @param id Stable widget identity.
 * @param users Borrowed collaborator data in display order.
 * @param count Number of collaborators.
 * @param maximum Maximum number of visible avatars.
 * @param size Avatar diameter in logical points, or zero for the native default. */
CUI_API void fpas_cui_presence(cui_ctx *ui, const char *id, const fpas_presence_user *users,
    int count, int maximum, float size);
#endif
