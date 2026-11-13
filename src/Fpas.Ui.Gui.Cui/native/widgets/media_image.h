#ifndef FPAS_CUI_MEDIA_IMAGE_H
#define FPAS_CUI_MEDIA_IMAGE_H
typedef struct { const char *path; const uint8_t *pixels; int width, height; int64_t version; } fpas_media_source;
#endif
