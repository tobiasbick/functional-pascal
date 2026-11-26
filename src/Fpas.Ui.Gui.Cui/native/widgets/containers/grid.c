// [Documentation](../../../../../docs/ui/gui/feedback/grid.md)
#include "../../fpas_cui.h"

/** Opens the native CUI grid with common box options. */
int fpas_cui_grid_begin(cui_ctx *ui, int columns, float minimum, float padding, float gap,
                        int clip, const char *id, int scroll) {
    const cui_box options = { .dir = CUI_GRID, .columns = columns, .min_column_width = minimum,
        .pad = padding, .gap = gap, .clip = clip != 0, .id = id,
        .scroll_y = scroll > 0, .scroll_x = scroll == 2 };
    return fpas_open_box(ui, &options);
}
