# Social and media

The gallery shows conversation, presence, scrolling figures and a set of media
views. Image sources can be local files or tightly packed RGBA buffers.
Keep an RGBA buffer immutable while the view uses it and supply a new array
when its pixels change. These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Chat message](chat-message.md) | Show incoming or outgoing messages and receipts. |
| [Chat divider](chat-divider.md) | Separate days or groups in a conversation. |
| [Typing indicator](typing-indicator.md) | Show which participant is typing. |
| [Comment](comment.md) | Show threaded comments with Like and Reply actions. |
| [Presence](presence.md) | Show participants, away state and overflow. |
| [Counter](counter.md) | Animate a changing number. |
| [Ticker](ticker.md) | Scroll a strip of figures and trends. |
| [Carousel](carousel.md) | Navigate between composed slides. |
| [Image viewer](image-viewer.md) | Inspect an image with pan and zoom. |
| [Image cropper](image-cropper.md) | Choose a free, fixed-ratio or round crop. |
| [Image compare](image-compare.md) | Reveal before and after images with a divider. |
| [Card stack](card-stack.md) | Present cards that can be dismissed left or right. |

## See also

- [UI handbook](../README.md)
- [Media examples in the gallery](../../../../examples/ui/gui/gallery/Pages/Social/Media.fpas)
- [Voice](../voice/README.md)
- [Social and media in the gallery](../../../../examples/ui/gui/gallery/Pages/Social/View.fpas)

## Source files

- `examples/ui/gui/gallery/Pages/Social/`
  - [Scenes.fpas](../../../../examples/ui/gui/gallery/Pages/Social/Scenes.fpas)
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Social/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Social/Update.fpas)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [media.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/media.c)
  - [social.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/social.c)
- `tests/Fixtures/gui-host/`
  - [MediaContract.fpas](../../../../tests/Fixtures/gui-host/MediaContract.fpas)
