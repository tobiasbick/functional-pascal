#ifndef FPAS_PASSWORD_STRENGTH_H
#define FPAS_PASSWORD_STRENGTH_H

/** Draws CUI's password strength meter.
 * @param ui Drawing context.
 * @param id Stable meter identity.
 * @param password Borrowed UTF-8 password.
 * @return CUI strength score from zero to four. */
CUI_API int fpas_cui_password_strength(cui_ctx *ui, const char *id, const char *password);

#endif
