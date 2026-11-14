#include "../../fpas_cui.h"

int fpas_cui_grid_begin(cui_ctx *ui, int columns, float minimum, float padding, float gap) {
    const cui_box options = { .dir = CUI_GRID, .columns = columns, .min_column_width = minimum,
        .pad = padding, .gap = gap };
    cui_box_begin(ui, &options);
    return cui__current(ui);
}
