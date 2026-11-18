# Fpas.Media.Images

Add a project reference to [Fpas.Media.Images.csproj](Fpas.Media.Images.csproj)
and import `Fpas.Media.Images`. Call `CoverDecoder.Decode(encoded, edge, token)`
on a worker with JPEG or PNG bytes. It normalizes orientation and color, scales
the picture to fit without cropping, and pads the square with transparency.

`Bitmap.Width` and `Bitmap.Height` equal `edge`. `Bitmap.Pixels` is a new
row-major byte array in straight-alpha RGBA order. Keep it unchanged while a
consumer displays it. Inputs are limited to 16 MiB and 64 million pixels.
The output edge must be between 1 and 8000 pixels. Cancellation is checked
before reading the image, before decoding and after copying pixels.

MagicScaler and its libjpeg and libpng plugins supply managed code and native
codecs. Build and publish outputs carry the package licenses and native notices.
Cache storage, album identity, revision tracking and eviction belong to the
caller. See the [FPAS media guide](../../docs/pascal/libraries/media.md) and
[PhotoSauce](https://github.com/saucecontrol/PhotoSauce).
