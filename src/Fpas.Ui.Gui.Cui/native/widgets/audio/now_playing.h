#ifndef FPAS_CUI_NOW_PLAYING_H
#define FPAS_CUI_NOW_PLAYING_H
CUI_API int fpas_cui_now_playing(cui_ctx *ui, const char *id, const char *title, const char *artist,
    const char *artwork, int palette, uint32_t rgba, float *position, float duration,
    int *playing, int width_kind, float width, int flags, int focus, int *parts);
#endif
