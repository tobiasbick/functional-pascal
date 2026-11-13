#ifndef FPAS_CUI_VOICE_NOTE_H
#define FPAS_CUI_VOICE_NOTE_H
CUI_API int fpas_cui_voice_note(cui_ctx *ui, const char *id, const float *levels, int count, float duration,
    float *position, float *speed, int *playing, int width_kind, float width, int flags, int focus, int *nodes);
#endif
