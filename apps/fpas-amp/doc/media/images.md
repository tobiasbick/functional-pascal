# FpasAmp.Media.Images

Add a project reference to [FpasAmp.Media.Images.csproj](../../src/FpasAmp.Media.Images/FpasAmp.Media.Images.csproj)
and import `FpasAmp.Media.Images`. Call `CoverDecoder.Decode(encoded, edge, token)`
on a worker with JPEG or PNG bytes. The decoder detects the format from
the bytes. It normalizes orientation and color, scales
the picture to fit without cropping, and pads the square with transparency.

`Bitmap.Width` and `Bitmap.Height` equal `edge`. `Bitmap.Pixels` is a new
row-major byte array in straight-alpha RGBA order. Keep it unchanged while a
consumer displays it. Inputs are limited to 16 MiB and 64 million pixels.
The output edge must be between 1 and 8000 pixels. Cancellation is checked
before reading image metadata, before decoding and after copying pixels.
An edge outside the supported range raises `ArgumentOutOfRangeException`.
Unsupported artwork or input exceeding the size limits raises
`InvalidDataException`. The pixel array contains exactly `edge * edge * 4`
bytes, with no row padding.

MagicScaler and its libjpeg and libpng plugins supply managed code and native
codecs. Build and publish outputs carry the package licenses and native notices.
The [JPEG fit test](../../tests/FpasAmp.Media.Tests/Images.fpas) checks the decoded
color and transparent padding using an eight-by-two solid red fixture.
Cache storage, album identity, revision tracking and eviction belong to the
caller.

## See also

- [Using the media libraries from FPAS](README.md)
- [FPAS image decoding and cancellation tests](../../tests/FpasAmp.Media.Tests/Consumer.fpas)
- [PhotoSauce](https://github.com/saucecontrol/PhotoSauce)

## Source files

- [Bitmap.cs](../../src/FpasAmp.Media.Images/Bitmap.cs)
- [CoverDecoder.cs](../../src/FpasAmp.Media.Images/CoverDecoder.cs)
