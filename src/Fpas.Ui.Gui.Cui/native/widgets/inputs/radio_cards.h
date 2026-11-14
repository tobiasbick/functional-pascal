#ifndef FPAS_RADIO_CARDS_H
#define FPAS_RADIO_CARDS_H

/** Draws CUI's native radio-card grid.
 * @param ui Current UI-thread context.
 * @param id Stable group identity.
 * @param titles Ordered literal titles.
 * @param descriptions Ordered secondary captions, with empty entries for none.
 * @param icons Ordered glyphs, with empty entries for none.
 * @param badges Ordered badge captions, with empty entries for none.
 * @param count Number of cards.
 * @param selected Model-owned index, or -1.
 * @param columns Maximum columns, or zero for CUI's automatic choice.
 * @param enabled Whether the cards accept input.
 * @param focus Zero or one plus the requested focus index.
 * @param nodes Receives the grid followed by the card nodes.
 * @return Proposed index, or -1 when the selection is unchanged. */
CUI_API int fpas_cui_radio_cards(cui_ctx *ui, const char *id, const char *const *titles,
    const char *const *descriptions, const char *const *icons, const char *const *badges,
    int count, int selected, int columns, int enabled, int focus, int *nodes);

#endif
