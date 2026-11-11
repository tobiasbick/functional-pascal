#ifndef FPAS_RATING_H
#define FPAS_RATING_H

/** Draws a native whole-star rating.
 * @param ui Current UI-thread context.
 * @param id Stable rating identity.
 * @param value Current model-owned rating.
 * @param maximum Number of stars.
 * @param enabled Whether the rating accepts pointer input.
 * @return Proposed rating, or minus one when unchanged. */
CUI_API int fpas_cui_rating(cui_ctx *ui, const char *id, int value, int maximum, int enabled);

#endif
