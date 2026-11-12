# CUI upstream bugs

The `CUI-NNN` identifiers are local references used by the FPAS adapters.

<a id="cui-001"></a>

## CUI-001 — An enclosing scroll view consumes the text area's wheel input

Reproduced with direct CUI calls on Windows at
`5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5` (v1.4.0).

```c
#include <cui.h>
#include <stdio.h>

static cui_rect frame(cui_ctx *ui) {
    char body[160] = "One\nTwo\nThree\nFour\nFive\nSix\nSeven\nEight\nNine\nTen";
    cui_id editor = 0;
    cui_begin_frame(ui, 0.016f);
    cui_scroll(ui, .id = "page", .height = CUI_PX(180), .width = CUI_FILL) {
        cui_textarea(ui, "about", body, sizeof body, .height = 72);
        editor = cui_last_id(ui);
        cui_box(ui, .height = CUI_PX(400)) {}
    }
    cui_end_frame(ui);
    return cui_get_rect(ui, editor);
}

int main(void) {
    cui_sw_renderer *sw = cui_sw_create();
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(sw) });
    cui_set_viewport(ui, 400, 240, 1);
    cui_set_reduced_motion(ui, true);
    frame(ui);
    cui_rect before = frame(ui);
    cui_input_mouse_move(ui, before.x + before.w / 2, before.y + before.h / 2);
    cui_input_scroll(ui, 0, -1);
    cui_rect after = frame(ui);
    printf("Text area Y before wheel: %.1f\n", before.y);
    printf("Text area Y after wheel: %.1f\n", after.y);
    cui_destroy(ui);
    cui_sw_destroy(sw);
}
```

```text
Text area Y before wheel: 0.0
Text area Y after wheel: -44.0
```

The pointer is over the text area and the area contains more lines than its
height permits. The outer scroll view consumes the wheel before the editor
draws, shifting the entire field. The FPAS adapter registers the editor's
scroll range with CUI's wheel routing. The containing page can scroll after
the editor reaches its boundary.
[CUI editor implementation](https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/src/cui_textedit.c)

- [ ] Verify current upstream and cover the editor and the enclosing viewport
  at both scroll boundaries.

<a id="cui-002"></a>

## CUI-002 — Batched tag input can exceed the tag limit

Reproduced on Windows with direct CUI v1.4.0 calls at
`5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5`.

Render an empty `cui_tag_input` with `.max_tags = 1` and a 64-byte buffer.
Press Tab to focus the draft, then send `cui_input_text(ui, "one,two,")`
before the next frame. The buffer becomes `one, two`. The native handler
checks the count once before processing all separators. FPAS bounds the
returned tag list before publishing it.

- [ ] Verify current upstream and cover batched tag entry.

<a id="cui-003"></a>

## CUI-003 — Replacing a complete code skips the completion event

Reproduced on Windows with direct CUI v1.4.0 calls at
`5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5`.

Render an empty six-box `cui_otp_input`, press Tab, then enter `ABC123` with
`cui_input_text`. It returns true. Install a `cui_platform_iface.get_clipboard`
callback that returns `UVWXYZ` and send Primary+V while the field is still
focused. The buffer becomes `UVWXYZ` and the call returns false. CUI only
compares the completion booleans. FPAS compares the edited code to the incoming
model and sends completion after the change when all boxes are filled.

- [ ] Verify current upstream and cover replacing one complete code with another.

<a id="cui-004"></a>

## CUI-004 — Searchable select loses keyboard focus on dismissal

Reproduced on Windows with direct CUI v1.4.0 calls at
`5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5`. Render a searchable
`cui_select`, focus it with Tab and press Enter. After the search editor
receives focus, press Escape and then Enter. The list remains closed.
The trigger never regained focus after its search editor disappeared.
The FPAS adapter returns the search editor's focus to the trigger when
the popup closes.

- [ ] Verify current upstream and exercise Escape followed by keyboard reopening.

<a id="cui-005"></a>

## CUI-005 — Number field can retain a clamped draft and emit NaN

Reproduced on Windows with direct CUI v1.4.0 calls at
`5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5`. Render `cui_number` with value 3
and bounds 0 and 99. Focus it, select the value, enter `999` and press Enter.
The model becomes 99 while the editor still shows 999. Replace the draft
with `nan` and the native call writes NaN into the model. Comparisons against
the bounds do not reject NaN. The FPAS adapter only proposes complete
finite numbers and refreshes the displayed draft on commit or blur.

- [ ] Verify current upstream and exercise clamping and nonfinite input.
