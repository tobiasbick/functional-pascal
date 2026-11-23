# CUI gallery reference crops

These light-theme references come from the upstream component screenshots at [CUI `e0124ea`](https://github.com/yappologistic/CUI/tree/e0124eaef2a41f07c7d6f22aed9af48314f1b3f3/docs/images/components). To reproduce a reference, average each 2 × 2 block of the original image with a box filter, crop the logical rectangle below, and save as RGBA PNG. Crop bounds exclude the right and bottom edges.

| Reference | Upstream component | Logical crop (left, top, right, bottom) |
| --- | --- | --- |
| actions-bold | [toggle-group.png](https://github.com/yappologistic/CUI/blob/e0124eaef2a41f07c7d6f22aed9af48314f1b3f3/docs/images/components/toggle-group.png) | 28, 36, 63, 74 |
| selection-switch | [toggles.png](https://github.com/yappologistic/CUI/blob/e0124eaef2a41f07c7d6f22aed9af48314f1b3f3/docs/images/components/toggles.png) | 226, 72, 268, 98 |
| picker-ruler | [ruler-histogram.png](https://github.com/yappologistic/CUI/blob/e0124eaef2a41f07c7d6f22aed9af48314f1b3f3/docs/images/components/ruler-histogram.png) | 199, 122, 249, 154 |
| disclosure-check | [checklist.png](https://github.com/yappologistic/CUI/blob/e0124eaef2a41f07c7d6f22aed9af48314f1b3f3/docs/images/components/checklist.png) | 44, 137, 75, 166 |
| audio-cutoff | [mixer.png](https://github.com/yappologistic/CUI/blob/e0124eaef2a41f07c7d6f22aed9af48314f1b3f3/docs/images/components/mixer.png) | 48, 89, 105, 137 |
| input-search | [inputs.png](https://github.com/yappologistic/CUI/blob/e0124eaef2a41f07c7d6f22aed9af48314f1b3f3/docs/images/components/inputs.png) | 344, 147, 379, 189 |
| format-color | [format-bar.png](https://github.com/yappologistic/CUI/blob/e0124eaef2a41f07c7d6f22aed9af48314f1b3f3/docs/images/components/format-bar.png) | 49, 27, 114, 76 |
| canvas-tools | [canvas-controls.png](https://github.com/yappologistic/CUI/blob/e0124eaef2a41f07c7d6f22aed9af48314f1b3f3/docs/images/components/canvas-controls.png) | 670, 184, 724, 303 |
| ratio-label | [ratio-slider.png](https://github.com/yappologistic/CUI/blob/e0124eaef2a41f07c7d6f22aed9af48314f1b3f3/docs/images/components/ratio-slider.png) | 27, 30, 100, 85 |
| palette-boundary | [palette-strip.png](https://github.com/yappologistic/CUI/blob/e0124eaef2a41f07c7d6f22aed9af48314f1b3f3/docs/images/components/palette-strip.png) | 125, 67, 155, 110 |

The [gallery test](../../Fpas.Ui.Gui.Cui.Tests/Packaging/GalleryReferenceTests.cs) compares each crop at fixed coordinates in the corresponding FPAS viewport at scale 1. The error is the average of the largest channel difference per pixel, on a scale from 0 to 255. Each crop has an allowance for rasterization differences. On the Linux software and OpenGL captures, replacing the crop with solid white or translating the crop three logical pixels along either axis exceeds its allowance.

The original screenshots are distributed with CUI. [CUI license](../../../third-party/cui/LICENSE-CUI.txt)
