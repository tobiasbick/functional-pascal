#ifndef FPAS_CUI_TIMELINE_EDITOR_H
#define FPAS_CUI_TIMELINE_EDITOR_H
typedef struct {
    float time, start, end, scroll, in, out;
    int track, key, range;
} fpas_timeline_view;
typedef struct {
    float start, end, fps, height, row_height, track_width, width;
    int ruler, width_kind, flags;
} fpas_timeline_options;
CUI_API int fpas_cui_timeline_editor(cui_ctx *ui, const char *id, cui_timeline_track *tracks, int count, fpas_timeline_view *view, const fpas_timeline_options *settings);
#endif
