#include "fpas_cui.h"

#if defined(_WIN32)
#define FPAS_GL_CALL __stdcall
#else
#define FPAS_GL_CALL
#endif

typedef void (FPAS_GL_CALL *get_texture_level_fn)(unsigned int target, int level, unsigned int parameter, int *value);

enum { FPAS_GL_TEXTURE_2D = 0x0DE1, FPAS_GL_TEXTURE_WIDTH = 0x1000, FPAS_GL_TEXTURE_HEIGHT = 0x1001 };

cui_texture fpas_cui_image_create(cui_app *app, int width, int height, const void *rgba) {
    if (!app || !rgba || width <= 0 || height <= 0) return 0;
    cui_ctx *ui = cui_app_ui(app);
    if (cui_app_is_software(app)) return cui_image_create(ui, width, height, rgba);

    get_texture_level_fn get_level = (get_texture_level_fn)cui__plat_gl_proc("glGetTexLevelParameteriv");
    if (!get_level) return 0;
    cui_texture texture = cui_image_create(ui, width, height, rgba);
    if (!texture) return 0;
    /* CUI leaves the texture bound, and its slot ID alone does not prove that the upload succeeded. */
    int actual_width = 0, actual_height = 0;
    get_level(FPAS_GL_TEXTURE_2D, 0, FPAS_GL_TEXTURE_WIDTH, &actual_width);
    get_level(FPAS_GL_TEXTURE_2D, 0, FPAS_GL_TEXTURE_HEIGHT, &actual_height);
    if (actual_width == width && actual_height == height) return texture;
    cui_image_destroy(ui, texture);
    return 0;
}

void fpas_cui_image(cui_ctx *ui, cui_texture texture, float width, float height) {
    const cui_image_opts options = { .width = CUI_PX(width), .height = CUI_PX(height) };
    cui_image_ex(ui, texture, &options);
}
